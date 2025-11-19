use serde::{Deserialize, Serialize};

use super::{Card, DestroyTarget, MagicianSkill, Oper, Role};

#[derive(Serialize, Deserialize, Debug)]
pub enum AgentRespEvent {
  WaitForReady { id: u32 },
  InitCard { id: u32, chosen: Card },
  Role { id: u32, chosen: Role },
  KillTarget { id: u32, chosen: Role },
  StealTarget { id: u32, chosen: Role },
  MagicTarget { id: u32, chosen: MagicianSkill },
  DestoryTarget { id: u32, chosen: Option<DestroyTarget> },
  Tomb { id: u32, chosen: bool },
  Oper { id: u32, chosen: Oper },
  From2 { id: u32, chosen: Card },
  From3 { id: u32, chosen: Card },
}
