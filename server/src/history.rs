// 用于游戏复盘

// TODO: send to message queue

use std::sync::atomic::{AtomicU32, Ordering};

use serde::{Deserialize, Serialize};
use tokio::select;
use tokio::sync::mpsc;
use tracing::info;
use valuable::Valuable;

use crate::deck::Deck;
use crate::domain::{Card, DestroyTarget, MagicianSkill, Oper, PlayerIndex, Role, RoleSet};
use crate::obs::Obs;

const EVENT_START_GAME: &str = "StartGame";
const EVENT_INIT_GOLD: &str = "InitGold";
const EVENT_INIT_CARD_REQ: &str = "InitCardReq";
const EVENT_INIT_CARD_RESP: &str = "InitCardResp";
const EVENT_START_ROUND: &str = "StartRound";
const EVENT_PUBLIC_DROP_ROLES: &str = "PublicDropRoles";
const EVENT_SECRET_FIRST_DROP_ROLE: &str = "SecretFirstDropRole";
const EVENT_SECRET_LAST_DROP_ROLE: &str = "SecretLastDropRole";
const EVENT_CHOOSE_ROLE_REQ: &str = "ChooseRoleReq";
const EVENT_CHOOSE_ROLE_RESP: &str = "ChooseRoleResp";
const EVENT_KILL_REQ: &str = "KillReq";
const EVENT_KILL_RESP: &str = "KillResp";
const EVENT_STEAL_REQ: &str = "StealReq";
const EVENT_STEAL_RESP: &str = "StealResp";
const EVENT_MAGIC_REQ: &str = "MagicReq";
const EVENT_MAGIC_RESP: &str = "MagicResp";
const EVENT_MERCHANT: &str = "Merchant";
const EVENT_ARCHITECT_DRAW_2_CARDS: &str = "ArchitectDraw2Cards";
const EVENT_DESTROY_REQ: &str = "DestroyReq";
const EVENT_DESTROY_RESP: &str = "DestroyResp";
const EVENT_TOMB_REQ: &str = "TombReq";
const EVENT_TOMB_RESP: &str = "TombResp";
const EVENT_OPER_REQ: &str = "OperReq";
const EVENT_OPER_RESP: &str = "OperResp";
const EVENT_DRAW_2_CARDS: &str = "Draw2Cards";
const EVENT_DRAW_3_CARDS: &str = "Draw3Cards";
const EVENT_PEEK_2_CARDS: &str = "Peek2Cards";
const EVENT_PEEK_3_CARDS: &str = "Peek3Cards";
const EVENT_CHOOSE_FROM_1: &str = "ChooseFrom1";
const EVENT_CHOOSE_FROM_2_REQ: &str = "ChooseFrom2Req";
const EVENT_CHOOSE_FROM_2_RESP: &str = "ChooseFrom2Resp";
const EVENT_CHOOSE_FROM_3_REQ: &str = "ChooseFrom3Req";
const EVENT_CHOOSE_FROM_3_RESP: &str = "ChooseFrom3Resp";
const EVENT_GOLD: &str = "GetGold";
const EVENT_BUILD: &str = "Build";
const EVENT_FIRST_8_BUILDINGS: &str = "First8Buildings";
const EVENT_NONFIRST_8_BUILDINGS: &str = "Nonfirst8Buildings";
const EVENT_SELL_CARD: &str = "SellCard";
const EVENT_SHUFFLE_DECK: &str = "ShuffleDeck";
const EVENT_REVEAL_ROLE: &str = "RevealRole";
const EVENT_MOVE_CROWN: &str = "MoveCrown";
const EVENT_SKIP_KILLED_TURN: &str = "SkipKilledTurn";
const EVENT_STEAL_GOLD: &str = "StealGold";
const EVENT_SWAP_CARDS: &str = "SwapCards";
const EVENT_REPLACE_CARDS: &str = "ReplaceCards";
const EVENT_FHINSH_GAME: &str = "FinishGame";

#[derive(Serialize, Deserialize, Debug)]
pub enum HistoryReqEvent {
  WaitForReady {
    id: u32,
  },
  StartGame {
    id: u32,
    init_crown: PlayerIndex,
  },
  InitGold {
    id: u32,
    actor: PlayerIndex,
    gold: u32,
  },
  InitCardReq {
    id: u32,
    actor: PlayerIndex,
    obs: Obs,
    c0: Card,
    c1: Card,
  },
  InitCardResp {
    id: u32,
    req_id: u32,
    chosen: Card,
    drop: Card,
  },
  StartRound {
    id: u32,
    round: u32,
    crown: PlayerIndex,
  },
  PublicDropRoles {
    id: u32,
    round: u32,
    roles: RoleSet,
  },
  SecretFirstDropRole {
    id: u32,
    round: u32,
    role: Role,
  },
  SecretLastDropRole {
    id: u32,
    round: u32,
    role: Role,
  },
  ChooseRoleReq {
    id: u32,
    actor: PlayerIndex,
    obs: Obs,
    choices: RoleSet,
  },
  ChooseRoleResp {
    id: u32,
    actor: PlayerIndex,
    obs: Obs,
    choices: RoleSet,
    chosen: Role,
  },
  KillReq {
    id: u32,
    actor: PlayerIndex,
    obs: Obs,
    choices: RoleSet,
  },
  KillResp {
    id: u32,
    req_id: u32,
    chosen: Role,
  },
  StealReq {
    id: u32,
    actor: PlayerIndex,
    obs: Obs,
    choices: RoleSet,
  },
  StealResp {
    id: u32,
    req_id: u32,
    chosen: Role,
  },
  MagicReq {
    id: u32,
    actor: PlayerIndex,
    obs: Obs,
  },
  MagicResp {
    id: u32,
    req_id: u32,
    chosen: MagicianSkill,
  },
  Merchant {
    id: u32,
    actor: PlayerIndex,
    round: u32,
  },
  ArchitectDraw2Cards {
    id: u32,
    actor: PlayerIndex,
    round: u32,
    c0: Option<Card>,
    c1: Option<Card>,
  },
  DestroyReq {
    id: u32,
    actor: PlayerIndex,
    obs: Obs,
    choices: Vec<DestroyTarget>,
  },
  DestroyResp {
    id: u32,
    req_id: u32,
    chosen_index: Option<PlayerIndex>,
    chosen_card: Option<Card>,
  },
  TombReq {
    id: u32,
    actor: PlayerIndex,
    obs: Obs,
    card: Card,
  },
  TombResp {
    id: u32,
    req_id: u32,
    chosen: bool,
  },
  OperReq {
    id: u32,
    actor: PlayerIndex,
    obs: Obs,
    choices: Vec<Oper>,
  },
  OperResp {
    id: u32,
    req_id: u32,
    chosen: Oper,
  },
  Draw2Cards {
    id: u32,
    actor: PlayerIndex,
    round: u32,
    c0: Option<Card>,
    c1: Option<Card>,
  },
  Draw3Cards {
    id: u32,
    actor: PlayerIndex,
    round: u32,
    c0: Option<Card>,
    c1: Option<Card>,
    c2: Option<Card>,
  },
  Peek2Cards {
    id: u32,
    actor: PlayerIndex,
    round: u32,
    c0: Option<Card>,
    c1: Option<Card>,
  },
  Peek3Cards {
    id: u32,
    actor: PlayerIndex,
    round: u32,
    c0: Option<Card>,
    c1: Option<Card>,
    c2: Option<Card>,
  },
  ChooseFrom1 {
    id: u32,
    actor: PlayerIndex,
    round: u32,
    c: Card,
  },
  ChooseFrom2Req {
    id: u32,
    actor: PlayerIndex,
    obs: Obs,
    c0: Card,
    c1: Card,
  },
  ChooseFrom2Resp {
    id: u32,
    req_id: u32,
    chosen: Card,
    drop: Card,
  },
  ChooseFrom3Req {
    id: u32,
    actor: PlayerIndex,
    obs: Obs,
    c0: Card,
    c1: Card,
    c2: Card,
  },
  ChooseFrom3Resp {
    id: u32,
    req_id: u32,
    chosen: Card,
    drop0: Card,
    drop1: Card,
  },
  Gold {
    id: u32,
    actor: PlayerIndex,
    round: u32,
    amount: u32,
  },
  Build {
    id: u32,
    actor: PlayerIndex,
    round: u32,
    card: Card,
  },
  First8Buildings {
    id: u32,
    actor: PlayerIndex,
    round: u32,
  },
  Nonfirst8Buildings {
    id: u32,
    actor: PlayerIndex,
    round: u32,
  },
  SellCard {
    id: u32,
    actor: PlayerIndex,
    round: u32,
    card: Card,
  },
  ShuffleDeck {
    id: u32,
    deck: Vec<Card>,
  },
  RevealRole {
    id: u32,
    actor: PlayerIndex,
    round: u32,
    role: Role,
  },
  MoveCrown {
    id: u32,
    round: u32,
    crown: PlayerIndex,
  },
  SkipKilledTurn {
    id: u32,
    actor: PlayerIndex,
    round: u32,
  },
  StealGold {
    id: u32,
    from: PlayerIndex,
    to: PlayerIndex,
    round: u32,
    amount: u32,
  },
  SwapCards {
    id: u32,
    actor: PlayerIndex,
    round: u32,
    i: PlayerIndex,
    j: PlayerIndex,
  },
  ReplaceCards {
    id: u32,
    actor: PlayerIndex,
    round: u32,
    removed: Vec<Card>,
    drawn: Vec<Card>,
  },
  FinishGame {
    id: u32,
  },
}

#[derive(Serialize, Deserialize, Debug)]
pub enum HistoryRespEvent {
  Ready,
}

pub struct History {
  id: AtomicU32,
  req_bcast_sender: mpsc::Sender<String>,
  resp_receiver: mpsc::Receiver<String>,
}

impl History {
  pub fn new(req_bcast_sender: mpsc::Sender<String>, resp_receiver: mpsc::Receiver<String>) -> Self {
    Self {
      id: AtomicU32::new(0),
      req_bcast_sender,
      resp_receiver,
    }
  }

  pub fn next_id(&mut self) -> u32 {
    self.id.fetch_add(1, Ordering::Relaxed)
  }

  pub async fn wait_for_ready(&mut self) {
    // TODO
    // id = ...
    // info!(...)
    // event = ...
    // send_event()
    let id = self.next_id();
    let req_event = HistoryReqEvent::WaitForReady { id };
    let json = serde_json::to_string(&req_event).unwrap();
    self.req_bcast_sender.send(json.clone()).await.unwrap();

    loop {
      select! {
        resp = self.resp_receiver.recv() => {
          let resp_event: HistoryRespEvent = serde_json::from_str(&resp.unwrap()).unwrap();
          match resp_event {
            HistoryRespEvent::Ready => {
              println!("Ready");
              break;
            }
          }
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
          info!("history wait for ready timeout");
          let req_event = HistoryReqEvent::WaitForReady { id: self.id.fetch_add(1, Ordering::Relaxed) };
          let json = serde_json::to_string(&req_event).unwrap();
          self.req_bcast_sender.send(json.clone()).await.unwrap();
        }
      }
    }
  }

  pub async fn game_start(&mut self, init_crown: PlayerIndex) {
    let id = self.next_id();

    info!(id = id, event = EVENT_START_GAME, init_crown = init_crown.value(),);

    let record = HistoryReqEvent::StartGame { id, init_crown };
    let json = serde_json::to_string(&record).unwrap();
    self.req_bcast_sender.send(json.clone()).await.unwrap();
  }

  pub async fn init_gold(&mut self, actor: PlayerIndex, gold: u32) {
    let id = self.next_id();

    info!(id = id, event = EVENT_INIT_GOLD, actor = actor.value(), gold = gold);

    let record = HistoryReqEvent::InitGold { id, actor, gold };
    let json = serde_json::to_string(&record).unwrap();
    self.req_bcast_sender.send(json.clone()).await.unwrap();
  }

  pub async fn init_card_req(&mut self, actor: PlayerIndex, obs: &Obs, c0: Card, c1: Card) -> u32 {
    let id = self.next_id();

    info!(
      id = id,
      event = EVENT_INIT_CARD_REQ,
      actor = actor.value(),
      obs = obs.as_value(),
      c0 = c0.as_value(),
      c1 = c1.as_value(),
    );

    let record = HistoryReqEvent::InitCardReq {
      id,
      actor,
      obs: obs.clone(),
      c0,
      c1,
    };
    let json = serde_json::to_string(&record).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    id
  }

  pub async fn init_card_resp(&mut self, req_id: u32, actor: PlayerIndex, obs: &Obs, chosen: Card, drop: Card) {
    let id = self.next_id();
    let event = HistoryReqEvent::InitCardResp {
      id,
      req_id,
      chosen,
      drop,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_INIT_CARD_RESP,
      actor = actor.value(),
      obs = obs.as_value(),
      chosen = chosen.as_value(),
      drop = drop.as_value(),
    );
  }

  pub async fn start_round(&mut self, round: u32, crown: PlayerIndex, deck: &Deck) {
    let id = self.next_id();
    let event = HistoryReqEvent::StartRound { id, round, crown };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_START_ROUND,
      round = round,
      crown = crown.value(),
      deck = deck.peek_deck().as_value(),
      drop = deck.peek_drop().as_value(),
    );
  }

  pub async fn public_drop_roles(&mut self, round: u32, roles: RoleSet) {
    let id = self.next_id();
    let event = HistoryReqEvent::PublicDropRoles { id, round, roles };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(id, event = EVENT_PUBLIC_DROP_ROLES, round, roles = roles.as_value(),);
  }

  pub async fn secret_first_drop_role(&mut self, round: u32, role: Role) {
    let id = self.next_id();
    let event = HistoryReqEvent::SecretFirstDropRole { id, round, role };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(id, event = EVENT_SECRET_FIRST_DROP_ROLE, round, role = role.as_value(),);
  }

  pub async fn secret_last_drop_role(&mut self, round: u32, role: Role) {
    let id = self.next_id();
    let event = HistoryReqEvent::SecretLastDropRole { id, round, role };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(id, event = EVENT_SECRET_LAST_DROP_ROLE, round, role = role.as_value(),);
  }

  pub async fn choose_role_req(&mut self, actor: PlayerIndex, obs: &Obs, choices: RoleSet) {
    let id = self.next_id();
    let record = HistoryReqEvent::ChooseRoleReq {
      id,
      actor,
      obs: obs.clone(),
      choices,
    };
    let json = serde_json::to_string(&record).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      event = EVENT_CHOOSE_ROLE_REQ,
      actor = actor.value(),
      obs = obs.as_value(),
      choices = choices.as_value(),
    );
  }

  pub async fn choose_role_resp(&mut self, actor: PlayerIndex, obs: &Obs, choices: RoleSet, chosen_role: Role) {
    let id = self.next_id();
    let record = HistoryReqEvent::ChooseRoleResp {
      id,
      actor,
      obs: obs.clone(),
      choices,
      chosen: chosen_role,
    };
    let json = serde_json::to_string(&record).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      event = EVENT_CHOOSE_ROLE_RESP,
      actor = actor.value(),
      obs = obs.as_value(),
      choices = choices.as_value(),
      chosen_role = chosen_role.as_value(),
    );
  }

  pub async fn kill_req(&mut self, actor: PlayerIndex, obs: &Obs, choices: RoleSet) -> u32 {
    let id = self.next_id();
    let event = HistoryReqEvent::KillReq {
      id,
      actor,
      obs: obs.clone(),
      choices,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_KILL_REQ,
      actor = actor.value(),
      obs = obs.as_value(),
      choices = choices.as_value(),
    );

    id
  }

  pub async fn kill_resp(&mut self, req_id: u32, chosen_role: Role) {
    let id = self.next_id();
    let event = HistoryReqEvent::KillResp {
      id,
      req_id,
      chosen: chosen_role,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      event = EVENT_KILL_RESP,
      id,
      req_id,
      chosen_role = chosen_role.as_value(),
    );
  }

  pub async fn steal_req(&mut self, actor: PlayerIndex, obs: &Obs, choices: RoleSet) -> u32 {
    let id = self.next_id();
    let event = HistoryReqEvent::StealReq {
      id,
      actor,
      obs: obs.clone(),
      choices,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_STEAL_REQ,
      obs = obs.as_value(),
      choices = choices.as_value(),
    );

    id
  }

  pub async fn steal_resp(&mut self, req_id: u32, chosen_role: Role) {
    let id = self.next_id();
    let event = HistoryReqEvent::StealResp {
      id,
      req_id,
      chosen: chosen_role,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_STEAL_RESP,
      req_id,
      chosen_role = chosen_role.as_value(),
    );
  }

  pub async fn magic_req(&mut self, actor: PlayerIndex, obs: &Obs) -> u32 {
    let id = self.next_id();
    let event = HistoryReqEvent::MagicReq {
      id,
      actor,
      obs: obs.clone(),
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(id, event = EVENT_MAGIC_REQ, actor = actor.value(), obs = obs.as_value(),);

    id
  }

  pub async fn magic_resp(&mut self, req_id: u32, chosen_skill: &MagicianSkill) {
    let id = self.next_id();
    let event = HistoryReqEvent::MagicResp {
      id,
      req_id,
      chosen: chosen_skill.clone(),
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_MAGIC_RESP,
      req_id,
      chosen_skill = chosen_skill.as_value(),
    );
  }

  pub async fn merchant(&mut self, actor: PlayerIndex, round: u32) {
    let id = self.next_id();
    info!(id, event = EVENT_MERCHANT, actor = actor.value(), round = round);
    let event = HistoryReqEvent::Merchant { id, actor, round };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();
  }

  pub async fn architect_draw_2_cards(&mut self, actor: PlayerIndex, round: u32, c0: Option<Card>, c1: Option<Card>) {
    let id = self.next_id();
    info!(
      id,
      event = EVENT_ARCHITECT_DRAW_2_CARDS,
      actor = actor.value(),
      round = round,
      c0 = c0.as_value(),
      c1 = c1.as_value()
    );
    let event = HistoryReqEvent::ArchitectDraw2Cards {
      id,
      actor,
      round,
      c0,
      c1,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();
  }

  pub async fn destroy_req(&mut self, actor: PlayerIndex, obs: &Obs, choices: &[DestroyTarget]) -> u32 {
    let id = self.next_id();
    let event = HistoryReqEvent::DestroyReq {
      id,
      actor,
      obs: obs.clone(),
      choices: choices.to_vec(),
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_DESTROY_REQ,
      actor = actor.value(),
      obs = obs.as_value(),
      choices = choices.as_value(),
    );

    id
  }

  pub async fn destroy_resp(&mut self, req_id: u32, chosen_index: Option<PlayerIndex>, chosen_card: Option<Card>) {
    let id = self.next_id();
    info!(
      id,
      event = EVENT_DESTROY_RESP,
      req_id,
      chosen_index = chosen_index.as_value(),
      chosen_card = chosen_card.as_value(),
    );
    let event = HistoryReqEvent::DestroyResp {
      id,
      req_id,
      chosen_index,
      chosen_card,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();
  }

  pub async fn tomb_req(&mut self, actor: PlayerIndex, obs: &Obs, card: Card) -> u32 {
    let id = self.next_id();
    let event = HistoryReqEvent::TombReq {
      id,
      actor,
      obs: obs.clone(),
      card,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      event = EVENT_TOMB_REQ,
      actor = actor.value(),
      obs = obs.as_value(),
      card = card.as_value(),
    );

    id
  }

  pub async fn tomb_resp(&mut self, req_id: u32, chosen: bool) {
    let id = self.next_id();
    let event = HistoryReqEvent::TombResp { id, req_id, chosen };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(id, event = EVENT_TOMB_RESP, req_id, chosen = chosen);
  }

  pub async fn oper_req(&mut self, actor: PlayerIndex, obs: &Obs, choices: &[Oper]) -> u32 {
    let id = self.next_id();
    let event = HistoryReqEvent::OperReq {
      id,
      actor,
      obs: obs.clone(),
      choices: choices.to_vec(),
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      event = EVENT_OPER_REQ,
      actor = actor.value(),
      obs = obs.as_value(),
      choices = choices.as_value(),
    );

    id
  }

  pub async fn oper_resp(&mut self, req_id: u32, chosen: Oper) {
    let id = self.next_id();
    let event = HistoryReqEvent::OperResp { id, req_id, chosen };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(id, event = EVENT_OPER_RESP, req_id, chosen = chosen.as_value(),)
  }

  pub async fn draw_2_cards(&mut self, round: u32, actor: PlayerIndex, c0: Option<Card>, c1: Option<Card>) {
    let id = self.next_id();
    let event = HistoryReqEvent::Draw2Cards {
      id,
      round,
      actor,
      c0,
      c1,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      event = EVENT_DRAW_2_CARDS,
      round = round,
      actor = actor.value(),
      c0 = c0.as_value(),
      c1 = c1.as_value()
    );
  }

  pub async fn draw_3_cards(
    &mut self, actor: PlayerIndex, round: u32, c0: Option<Card>, c1: Option<Card>, c2: Option<Card>,
  ) {
    let id = self.next_id();
    let event = HistoryReqEvent::Draw3Cards {
      id,
      round,
      actor,
      c0,
      c1,
      c2,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_DRAW_3_CARDS,
      actor = actor.value(),
      round = round,
      c0 = c0.as_value(),
      c1 = c1.as_value(),
      c2 = c2.as_value()
    );
  }

  pub async fn peek_2_cards(&mut self, actor: PlayerIndex, round: u32, c0: Option<Card>, c1: Option<Card>) {
    let id = self.next_id();
    let event = HistoryReqEvent::Peek2Cards {
      id,
      actor,
      round,
      c0,
      c1,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      event = EVENT_PEEK_2_CARDS,
      actor = actor.as_value(),
      round = round,
      c0 = c0.as_value(),
      c1 = c1.as_value(),
    )
  }

  pub async fn peek_3_cards(
    &mut self, actor: PlayerIndex, round: u32, c0: Option<Card>, c1: Option<Card>, c2: Option<Card>,
  ) {
    let id = self.next_id();
    let event = HistoryReqEvent::Peek3Cards {
      id,
      actor,
      round,
      c0,
      c1,
      c2,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_PEEK_3_CARDS,
      actor = actor.as_value(),
      round = round,
      c0 = c0.as_value(),
      c1 = c1.as_value(),
      c2 = c2.as_value(),
    )
  }

  pub async fn choose_from_1(&mut self, actor: PlayerIndex, round: u32, c: Card) {
    let id = self.next_id();
    let event = HistoryReqEvent::ChooseFrom1 { id, actor, round, c };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id = id,
      event = EVENT_CHOOSE_FROM_1,
      actor = actor.as_value(),
      round = round,
      c = c.as_value(),
    )
  }

  pub async fn choose_from_2_req(&mut self, actor: PlayerIndex, obs: &Obs, c0: Card, c1: Card) -> u32 {
    let id = self.next_id();
    let event = HistoryReqEvent::ChooseFrom2Req {
      id,
      actor,
      obs: obs.clone(),
      c0,
      c1,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_CHOOSE_FROM_2_REQ,
      actor = actor.as_value(),
      obs = obs.as_value(),
      c0 = c0.as_value(),
      c1 = c1.as_value(),
    );

    id
  }

  pub async fn choose_from_2_resp(&mut self, req_id: u32, chosen: Card, drop: Card) {
    let id = self.next_id();
    let event = HistoryReqEvent::ChooseFrom2Resp {
      id,
      req_id,
      chosen,
      drop,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_CHOOSE_FROM_2_RESP,
      req_id,
      chosen = chosen.as_value(),
      drop = drop.as_value(),
    )
  }

  pub async fn choose_from_3_req(&mut self, actor: PlayerIndex, obs: &Obs, c0: Card, c1: Card, c2: Card) -> u32 {
    let id = self.next_id();
    let event = HistoryReqEvent::ChooseFrom3Req {
      id,
      actor,
      obs: obs.clone(),
      c0,
      c1,
      c2,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_CHOOSE_FROM_3_REQ,
      actor = actor.as_value(),
      obs = obs.as_value(),
      c0 = c0.as_value(),
      c1 = c1.as_value(),
      c2 = c2.as_value(),
    );

    id
  }

  pub async fn choose_from_3_resp(&mut self, req_id: u32, chosen: Card, drop0: Card, drop1: Card) {
    let id = self.next_id();
    let event = HistoryReqEvent::ChooseFrom3Resp {
      id,
      req_id,
      chosen,
      drop0,
      drop1,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_CHOOSE_FROM_3_RESP,
      req_id,
      chosen = chosen.as_value(),
      drop0 = drop0.as_value(),
      drop1 = drop1.as_value(),
    )
  }

  pub async fn gold(&mut self, actor: PlayerIndex, round: u32, amount: u32) {
    let id = self.next_id();
    let event = HistoryReqEvent::Gold {
      id,
      actor,
      round,
      amount,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_GOLD,
      actor = actor.as_value(),
      round = round,
      amount = amount,
    )
  }

  pub async fn build(&mut self, actor: PlayerIndex, round: u32, c: Card) {
    let id = self.next_id();
    let event = HistoryReqEvent::Build {
      id,
      actor,
      round,
      card: c,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_BUILD,
      actor = actor.as_value(),
      round = round,
      card = c.as_value(),
    )
  }

  pub async fn first_8_buildings(&mut self, actor: PlayerIndex, round: u32) {
    let id = self.next_id();
    let event = HistoryReqEvent::First8Buildings { id, actor, round };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_FIRST_8_BUILDINGS,
      actor = actor.as_value(),
      round = round
    )
  }

  pub async fn nonfirst_8_buildings(&mut self, actor: PlayerIndex, round: u32) {
    let id = self.next_id();
    let event = HistoryReqEvent::Nonfirst8Buildings { id, actor, round };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_NONFIRST_8_BUILDINGS,
      actor = actor.as_value(),
      round = round,
    )
  }

  pub async fn sell_card(&mut self, actor: PlayerIndex, round: u32, c: Card) {
    let id = self.next_id();
    let event = HistoryReqEvent::SellCard {
      id,
      actor,
      round,
      card: c,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_SELL_CARD,
      actor = actor.as_value(),
      round = round,
      card = c.as_value(),
    )
  }

  pub async fn shuffle_deck(&mut self, deck: &Vec<Card>) {
    let id = self.next_id();
    let event = HistoryReqEvent::ShuffleDeck { id, deck: deck.clone() };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(id, event = EVENT_SHUFFLE_DECK, deck = deck.as_value(),)
  }

  pub async fn reveal_role(&mut self, actor: PlayerIndex, round: u32, role: Role) {
    let id = self.next_id();
    let event = HistoryReqEvent::RevealRole { id, actor, round, role };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_REVEAL_ROLE,
      actor = actor.as_value(),
      round = round,
      role = role.as_value(),
    )
  }

  pub async fn move_crown(&mut self, round: u32, crown: PlayerIndex) {
    let id = self.next_id();
    let event = HistoryReqEvent::MoveCrown { id, round, crown };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(id, event = EVENT_MOVE_CROWN, round = round, crown = crown.as_value(),)
  }

  pub async fn skip_killed_turn(&mut self, actor: PlayerIndex, round: u32) {
    let id = self.next_id();
    let event = HistoryReqEvent::SkipKilledTurn { id, actor, round };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_SKIP_KILLED_TURN,
      actor = actor.as_value(),
      round = round,
    )
  }

  pub async fn steal_gold(&mut self, from: PlayerIndex, to: PlayerIndex, round: u32, amount: u32) {
    let id = self.next_id();
    let event = HistoryReqEvent::StealGold {
      id,
      from,
      to,
      round,
      amount,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_STEAL_GOLD,
      from = from.as_value(),
      to = to.as_value(),
      round = round,
      amount = amount,
    )
  }

  pub async fn swap_cards(&mut self, actor: PlayerIndex, round: u32, i: PlayerIndex, j: PlayerIndex) {
    let id = self.next_id();
    let event = HistoryReqEvent::SwapCards { id, actor, round, i, j };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();

    info!(
      id,
      event = EVENT_SWAP_CARDS,
      actor = actor.as_value(),
      round = round,
      i = i.as_value(),
      j = j.as_value(),
    )
  }

  pub async fn replace_cards(&mut self, actor: PlayerIndex, round: u32, removed: Vec<Card>, drawn: Vec<Card>) {
    let id = self.next_id();

    info!(
      id,
      event = EVENT_REPLACE_CARDS,
      actor = actor.as_value(),
      round = round,
      removed = removed.as_value(),
      drawn = drawn.as_value(),
    );

    let event = HistoryReqEvent::ReplaceCards {
      id,
      actor,
      round,
      removed,
      drawn,
    };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();
  }

  pub async fn finish_game(&mut self) {
    let id = self.next_id();
    info!(id, event = EVENT_FHINSH_GAME);
    let event = HistoryReqEvent::FinishGame { id };
    let json = serde_json::to_string(&event).unwrap();
    self.req_bcast_sender.send(json).await.unwrap();
  }
}
