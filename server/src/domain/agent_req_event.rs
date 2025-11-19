use serde::{Deserialize, Serialize};

use super::{Card, DestroyTarget, Oper, RoleSet};
use crate::obs::Obs;

#[derive(Serialize, Deserialize, Debug)]
pub enum AgentReqEvent {
  WaitForReady {
    id: u32,
  },
  ChooseInitCard {
    id: u32,
    obs: Obs,
    c0: Card,
    c1: Card,
  },
  ChooseRole {
    id: u32,
    obs: Obs,
    roles: RoleSet,
  },
  ChooseKillTarget {
    id: u32,
    obs: Obs,
    choices: RoleSet,
  },
  ChooseStealTarget {
    id: u32,
    obs: Obs,
    choices: RoleSet,
  },
  ChooseMagicTarget {
    id: u32,
    obs: Obs,
  },
  ChooseDestoryTarget {
    id: u32,
    obs: Obs,
    choices: Vec<DestroyTarget>,
  },
  ChooseTomb {
    id: u32,
    obs: Obs,
    c: Card,
  },
  ChooseOper {
    id: u32,
    obs: Obs,
    choices: Vec<Oper>,
  },
  ChooseFrom2 {
    id: u32,
    obs: Obs,
    c0: Card,
    c1: Card,
  },
  ChooseFrom3 {
    id: u32,
    obs: Obs,
    c0: Card,
    c1: Card,
    c2: Card,
  },
}
