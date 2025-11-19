use async_trait::async_trait;
use redis::AsyncCommands;
use tokio::select;
use tracing::{error, info};
use uuid::Uuid;

use crate::IdGen;
use crate::abstract_fa_agent::AbstractFAAgent;
use crate::domain::{AgentReqEvent, AgentRespEvent, Card, DestroyTarget, MagicianSkill, Oper, Role, RoleSet};
use crate::obs::Obs;

pub struct RedisProxyFAAgent {
  id_gen: IdGen,
  _fallback: Box<dyn AbstractFAAgent>,
  redis_conn: redis::aio::MultiplexedConnection, // TODO: use redis::aio::MultiplexedConnection
  req_redis_key: String,
  resp_redis_key: String,
}

impl RedisProxyFAAgent {
  pub fn new(
    agent_uuid: Uuid, id_gen: IdGen, redis_conn: redis::aio::MultiplexedConnection,
    fallback: Box<dyn AbstractFAAgent>,
  ) -> Self {
    let room_uuid = "";
    let req_redis_key = format!("room{room_uuid}:agent{agent_uuid}");
    let resp_redis_key = format!("agent{agent_uuid}_to_room{room_uuid}");
    Self {
      id_gen,
      _fallback: fallback,
      redis_conn,
      req_redis_key,
      resp_redis_key,
    }
  }

  async fn block_on_req(&mut self, event: &AgentReqEvent) -> AgentRespEvent {
    let json = serde_json::to_string(&event).unwrap();
    self
      .redis_conn
      .lpush::<String, String, usize>(self.req_redis_key.clone(), json.clone())
      .await
      .unwrap();

    const TIMEOUT: f64 = 1.0; // in seconds

    loop {
      select! {
        resp = self.redis_conn.brpop::<String, Option<[String; 2]>>(self.resp_redis_key.clone(), TIMEOUT) => {
            match resp {
              Ok(Some(items)) => {
                let payload = items[1].clone();
                let event: AgentRespEvent = serde_json::from_str(&payload).unwrap();
                return event;
              }
              Ok(None) => {
                info!("RedisProxyFAAgent block on resp timeout");
              }
              Err(e) => {
                error!("RedisProxyFAAgent brpop error: {:?}", e);
              }
            }
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
          info!("RedisProxyFAAgent block on req timeout");
        }
      }
    }
  }
}

#[async_trait]
impl AbstractFAAgent for RedisProxyFAAgent {
  fn name(&self) -> &str {
    "RedisProxyFAAgent"
  }

  async fn wait_for_ready(&mut self) {
    let id = self.id_gen.gen_next();
    let event = AgentReqEvent::WaitForReady { id };
    loop {
      let resp_event = self.block_on_req(&event).await;
      match resp_event {
        AgentRespEvent::WaitForReady { id: resp_id } => {
          if resp_id == id {
            return;
          }
          error!("wrong id")
        },
        _ => {
          error!("unexpected event");
        },
      }
    }
  }

  async fn choose_init_card(&mut self, obs: &Obs, c0: Card, c1: Card) -> Card {
    let id = self.id_gen.gen_next();
    let event = AgentReqEvent::ChooseInitCard {
      id,
      obs: obs.clone(),
      c0,
      c1,
    };
    loop {
      let resp_event = self.block_on_req(&event).await;
      match resp_event {
        AgentRespEvent::InitCard { id: resp_id, chosen } => {
          if resp_id == id {
            return chosen;
          }
          error!("wrong id")
        },
        _ => {
          error!("unexpected event");
        },
      }
    }

    // TODO: fallback
    // self.fallback.choose_init_card(obs, c1, c2).await
  }

  async fn choose_role(&mut self, obs: &Obs, roles: RoleSet) -> Role {
    let id = self.id_gen.gen_next();
    let event = AgentReqEvent::ChooseRole {
      id,
      obs: obs.clone(),
      roles,
    };
    loop {
      let resp_event = self.block_on_req(&event).await;
      match resp_event {
        AgentRespEvent::Role { id: resp_id, chosen } => {
          if resp_id == id {
            return chosen;
          }
          error!("wrong id");
        },
        _ => {
          error!("unexpected event");
        },
      }
    }

    // TODO: fallback
    // self.fallback.choose_role(obs, roles).await
  }

  async fn choose_kill_target(&mut self, obs: &Obs, choices: RoleSet) -> Role {
    let id = self.id_gen.gen_next();
    let event = AgentReqEvent::ChooseKillTarget {
      id,
      obs: obs.clone(),
      choices,
    };
    loop {
      let resp_event = self.block_on_req(&event).await;
      match resp_event {
        AgentRespEvent::KillTarget { id: resp_id, chosen } => {
          if resp_id == id {
            return chosen;
          }
          error!("wrong id");
        },
        _ => {
          error!("unexpected event");
        },
      }
    }

    // TODO: fallback
    // self.fallback.choose_kill_target(obs, choices).await
  }

  async fn choose_steal_target(&mut self, obs: &Obs, choices: RoleSet) -> Role {
    let id = self.id_gen.gen_next();
    let event = AgentReqEvent::ChooseStealTarget {
      id,
      obs: obs.clone(),
      choices,
    };
    loop {
      let resp_event = self.block_on_req(&event).await;
      match resp_event {
        AgentRespEvent::StealTarget { id: resp_id, chosen } => {
          if resp_id == id {
            return chosen;
          }
          error!("wrong id");
        },
        _ => {
          error!("unexpected event");
        },
      }
    }

    // TODO: fallback
    // self.fallback.choose_steal_target(obs, choices).await
  }

  async fn choose_swap_target(&mut self, obs: &Obs) -> MagicianSkill {
    let id = self.id_gen.gen_next();
    let event = AgentReqEvent::ChooseMagicTarget { id, obs: obs.clone() };
    loop {
      let resp_event = self.block_on_req(&event).await;
      match resp_event {
        AgentRespEvent::MagicTarget { id: resp_id, chosen } => {
          if resp_id == id {
            return chosen;
          }
          error!("wrong id");
        },
        _ => {
          error!("unexpected event");
        },
      }
    }

    // TODO: fallback
    // self.fallback.choose_swap_target(obs).await
  }

  async fn choose_destory_target(&mut self, obs: &Obs, choices: &[DestroyTarget]) -> Option<DestroyTarget> {
    let id = self.id_gen.gen_next();
    let event = AgentReqEvent::ChooseDestoryTarget {
      id,
      obs: obs.clone(),
      choices: choices.to_vec(),
    };
    loop {
      let resp_event = self.block_on_req(&event).await;
      match resp_event {
        AgentRespEvent::DestoryTarget { id: resp_id, chosen } => {
          if resp_id == id {
            return chosen;
          }
          error!("wrong id");
        },
        _ => {
          error!("unexpected event");
        },
      }
    }

    // TODO: fallback
    // self.fallback.choose_destory_target(obs, choices).await
  }

  async fn choose_tomb(&mut self, obs: &Obs, c: Card) -> bool {
    let id = self.id_gen.gen_next();
    let event = AgentReqEvent::ChooseTomb {
      id,
      obs: obs.clone(),
      c,
    };
    loop {
      let resp_event = self.block_on_req(&event).await;
      match resp_event {
        AgentRespEvent::Tomb { id: resp_id, chosen } => {
          if resp_id == id {
            return chosen;
          }
          error!("wrong id");
        },
        _ => {
          error!("unexpected event");
        },
      }
    }

    // TODO: fallback
    // self.fallback.choose_tomb(obs, c).await
  }

  async fn choose_oper(&mut self, obs: &Obs, choices: &[Oper]) -> Oper {
    let id = self.id_gen.gen_next();
    let event = AgentReqEvent::ChooseOper {
      id,
      obs: obs.clone(),
      choices: choices.to_vec(),
    };
    loop {
      let resp_event = self.block_on_req(&event).await;
      match resp_event {
        AgentRespEvent::Oper { id: resp_id, chosen } => {
          if resp_id == id {
            return chosen;
          }
          error!("wrong id");
        },
        _ => {
          error!("unexpected event");
        },
      }
    }

    // TODO: fallback
    // self.fallback.choose_oper(obs, choices).await
  }

  async fn choose_from_2(&mut self, obs: &Obs, c0: Card, c1: Card) -> Card {
    let id = self.id_gen.gen_next();
    let event = AgentReqEvent::ChooseFrom2 {
      id,
      obs: obs.clone(),
      c0,
      c1,
    };
    loop {
      let resp_event = self.block_on_req(&event).await;
      match resp_event {
        AgentRespEvent::From2 { id: resp_id, chosen } => {
          if resp_id == id {
            return chosen;
          }
          error!("wrong id");
        },
        _ => {
          error!("unexpectyed event");
        },
      }
    }

    // TODO: fallback
    // self.fallback.choose_from_2(obs, c0, c1).await
  }

  async fn choose_from_3(&mut self, obs: &Obs, c0: Card, c1: Card, c2: Card) -> Card {
    let id = self.id_gen.gen_next();
    let event = AgentReqEvent::ChooseFrom3 {
      id,
      obs: obs.clone(),
      c0,
      c1,
      c2,
    };
    loop {
      let resp_event = self.block_on_req(&event).await;
      match resp_event {
        AgentRespEvent::From3 { id: resp_id, chosen } => {
          if resp_id == id {
            return chosen;
          }
          error!("wrong id");
        },
        _ => {
          error!("unexpected event");
        },
      }
    }

    // TODO: fallback
    // self.fallback.choose_from_3(obs, c0, c1, c2).await
  }
}
