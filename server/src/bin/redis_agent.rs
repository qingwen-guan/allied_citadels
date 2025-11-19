use redis::AsyncCommands;
use server::{AbstractFAAgent, Config, V2FAAgent, domain, init_log};

async fn handle_event(
  event: domain::AgentReqEvent, agent: &mut V2FAAgent, con: &mut redis::aio::MultiplexedConnection,
  resp_redis_key: &str,
) -> anyhow::Result<()> {
  match event {
    domain::AgentReqEvent::WaitForReady { id } => {
      // TODO: remvoe domain::xxx
      let event = domain::AgentRespEvent::WaitForReady { id };
      let json = serde_json::to_string(&event).unwrap();
      println!("> {}", json);
      let _: usize = con.lpush(resp_redis_key, json.clone()).await.unwrap();
      Ok(())
    },
    domain::AgentReqEvent::ChooseInitCard { id, obs, c0, c1 } => {
      let chosen = agent.choose_init_card(&obs, c0, c1).await;
      let event = domain::AgentRespEvent::InitCard { id, chosen };
      let json = serde_json::to_string(&event).unwrap();
      println!("> {}", json);
      let _: usize = con.lpush(resp_redis_key, json.clone()).await.unwrap();
      Ok(())
    },
    domain::AgentReqEvent::ChooseRole { id, obs, roles } => {
      let chosen = agent.choose_role(&obs, roles).await;
      let event = domain::AgentRespEvent::Role { id, chosen };
      let json = serde_json::to_string(&event).unwrap();
      println!("> {}", json);
      let _: usize = con.lpush(resp_redis_key, json.clone()).await.unwrap();
      Ok(())
    },
    domain::AgentReqEvent::ChooseKillTarget { id, obs, choices } => {
      let chosen = agent.choose_kill_target(&obs, choices).await;
      let event = domain::AgentRespEvent::KillTarget { id, chosen };
      let json = serde_json::to_string(&event).unwrap();
      println!("> {}", json);
      let _: usize = con.lpush(resp_redis_key, json.clone()).await.unwrap();
      Ok(())
    },
    domain::AgentReqEvent::ChooseStealTarget { id, obs, choices } => {
      let chosen = agent.choose_steal_target(&obs, choices).await;
      let event = domain::AgentRespEvent::StealTarget { id, chosen };
      let json = serde_json::to_string(&event).unwrap();
      println!("> {}", json);
      let _: usize = con.lpush(resp_redis_key, json.clone()).await.unwrap();
      Ok(())
    },
    domain::AgentReqEvent::ChooseMagicTarget { id, obs } => {
      let chosen = agent.choose_swap_target(&obs).await;
      let event = domain::AgentRespEvent::MagicTarget { id, chosen };
      let json = serde_json::to_string(&event).unwrap();
      println!("> {}", json);
      let _: usize = con.lpush(resp_redis_key, json.clone()).await.unwrap();
      Ok(())
    },
    domain::AgentReqEvent::ChooseDestoryTarget { id, obs, choices } => {
      let chosen = agent.choose_destory_target(&obs, &choices).await;
      let event = domain::AgentRespEvent::DestoryTarget { id, chosen };
      let json = serde_json::to_string(&event).unwrap();
      println!("> {}", json);
      let _: usize = con.lpush(resp_redis_key, json.clone()).await.unwrap();
      Ok(())
    },
    domain::AgentReqEvent::ChooseTomb { id, obs, c } => {
      let chosen = agent.choose_tomb(&obs, c).await;
      let event = domain::AgentRespEvent::Tomb { id, chosen };
      let json = serde_json::to_string(&event).unwrap();
      println!("> {}", json);
      let _: usize = con.lpush(resp_redis_key, json.clone()).await.unwrap();
      Ok(())
    },
    domain::AgentReqEvent::ChooseOper { id, obs, choices } => {
      let chosen = agent.choose_oper(&obs, &choices).await;
      let event = domain::AgentRespEvent::Oper { id, chosen };
      let json = serde_json::to_string(&event).unwrap();
      println!("> {}", json);
      let _: usize = con.lpush(resp_redis_key, json.clone()).await.unwrap();
      Ok(())
    },
    domain::AgentReqEvent::ChooseFrom2 { id, obs, c0, c1 } => {
      let chosen = agent.choose_from_2(&obs, c0, c1).await;
      let event = domain::AgentRespEvent::From2 { id, chosen };
      let json = serde_json::to_string(&event).unwrap();
      println!("> {}", json);
      let _: usize = con.lpush(resp_redis_key, json.clone()).await.unwrap();
      Ok(())
    },
    domain::AgentReqEvent::ChooseFrom3 { id, obs, c0, c1, c2 } => {
      let chosen = agent.choose_from_3(&obs, c0, c1, c2).await;
      let event = domain::AgentRespEvent::From3 { id, chosen };
      let json = serde_json::to_string(&event).unwrap();
      println!("> {}", json);
      let _: usize = con.lpush(resp_redis_key, json.clone()).await.unwrap();
      Ok(())
    },
  } // match event
}

async fn work() -> anyhow::Result<()> {
  let config = Config::load("config.toml")?;

  let mut agent = V2FAAgent::new();
  let redis_conn = redis::Client::open("redis://127.0.0.1:6379")?;
  let mut con = redis_conn.get_multiplexed_async_connection().await?;

  let room_uuid = "";
  let agent_uuid = config.ws_agent_uuid;

  let req_redis_key = format!("room{room_uuid}:agent{agent_uuid}");
  let resp_redis_key = format!("agent{agent_uuid}_to_room{room_uuid}");

  println!("req_redis_key: {}", req_redis_key);

  loop {
    match con
      .brpop::<String, Option<(String, String)>>(req_redis_key.clone(), 1.0)
      .await
    {
      Ok(Some(items)) => {
        let text = items.1.clone();
        let event: domain::AgentReqEvent = serde_json::from_str(&text).unwrap();
        handle_event(event, &mut agent, &mut con, &resp_redis_key).await?;
      },
      Ok(None) => {
        println!("RedisAgent timeout");
      },
      Err(e) => {
        println!("RedisAgent error: {}", e);
      },
    }
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let _guard = init_log("ws_agent");

  work().await?;

  Ok(())
}
