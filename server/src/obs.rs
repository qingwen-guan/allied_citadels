mod building_extra_score;
mod building_info;
mod common_player_info;
mod hero_info;
mod round_info;
mod villain_info;

pub use hero_info::HeroInfo;
use round_info::RoundInfo;
use serde::{Deserialize, Serialize};
use valuable::Valuable;
pub use villain_info::VillainInfo;

use crate::deck::Deck;
use crate::domain::{Camp, Card, PlayerIndex, PlayerOffset, PlayerOffsetSet, Role, RoleSet};
use crate::player::Player;
use crate::player_indexed_vec::PlayerIndexedVec;

#[derive(Clone, Valuable, Serialize, Deserialize, Debug)]
pub struct Obs {
  num_players: usize,
  round_info: RoundInfo,
  actor_info: HeroInfo,
  villain_infos: Vec<VillainInfo>, // 从自己的下家开始，逆时针的其他玩家，也就是 player_offset 为 1, 2, ... 的玩家
  deck_cnt: usize,
  drop_cnt: usize,
  total_score: [u32; 2],
}

impl Obs {
  pub fn new(
    num_players: usize, round: u32, crown: PlayerOffset, actor_info: HeroInfo, villain_infos: Vec<VillainInfo>,
    deck: &Deck,
  ) -> Self {
    Self {
      num_players,
      round_info: RoundInfo::new(round, crown),
      actor_info,
      villain_infos,
      deck_cnt: deck.peek_deck().len(),
      drop_cnt: deck.peek_drop().len(),
      total_score: [0, 0],
    }
  }

  pub fn round(&self) -> u32 {
    self.round_info.round()
  }

  pub fn set_crown(&mut self, crown: PlayerOffset) {
    self.round_info.set_crown(crown);
  }

  pub fn set_killed(&mut self, killed: PlayerOffset) {
    self.round_info.set_killed_offset(killed);
  }

  pub fn set_stolen(&mut self, stolen: PlayerOffset) {
    self.round_info.set_stolen_offset(stolen);
  }

  pub fn set_round(&mut self, round: u32) {
    self.round_info.set_round(round);
  }

  pub fn set_roles_public_dropped(&mut self, roles: RoleSet) {
    self.round_info.set_roles_public_dropped(roles);
  }

  pub fn set_players_choose_role_before(&mut self, offsets: PlayerOffsetSet) {
    self.round_info.set_players_choose_role_before(offsets);
  }

  pub fn set_players_choose_role_after(&mut self, offsets: PlayerOffsetSet) {
    self.round_info.set_players_choose_role_after(offsets);
  }

  pub fn set_roles_chosen_before(&mut self, roles: RoleSet) {
    self.round_info.set_roles_chosen_before(roles);
  }

  pub fn set_roles_chosen_after(&mut self, roles: RoleSet) {
    self.round_info.set_roles_chosen_after(roles);
  }

  pub fn set_villain_role(&mut self, offset: PlayerOffset, role: Role) {
    self.villain_infos[offset.value() - 1].set_role(role);
  }

  pub fn set_killed_role(&mut self, role: Role) {
    self.round_info.set_killed_role(role);
  }

  pub fn set_stolen_role(&mut self, role: Role) {
    self.round_info.set_stolen_role(role);
  }

  pub fn reset(&mut self) {
    self.actor_info.unset_role();
    for villain in self.villain_infos.iter_mut() {
      villain.unset_role();
    }
    self.round_info.reset();
  }

  pub fn set_actor_role(&mut self, role: Role) {
    self.actor_info.set_role(role);
  }

  pub fn update_infos(&mut self, deck: &Deck, players: &PlayerIndexedVec<Player>, actor: PlayerIndex) {
    self.deck_cnt = deck.peek_deck().len();
    self.drop_cnt = deck.peek_drop().len();

    let mut villains = Vec::new();
    for i in actor.value() + 1..players.len() {
      villains.push(&players[PlayerIndex::from_usize(i)]);
    }
    for i in 0..actor.value() {
      villains.push(&players[PlayerIndex::from_usize(i)]);
    }

    let actor = &players[actor];
    self.actor_info.update_info(actor);
    self.villain_infos.iter_mut().enumerate().for_each(|(i, villain)| {
      villain.update_info(villains[i]);
    });

    self.total_score = [0, 0];
    self.total_score[actor.camp() as usize] += actor.score();
    for villain in villains.iter() {
      self.total_score[villain.camp() as usize] += villain.score();
    }
  }

  pub fn actor_num_cards(&self) -> usize {
    self.actor_info.num_cards()
  }

  pub fn num_players(&self) -> usize {
    self.num_players
  }

  pub fn hero_card_at(&self, index: usize) -> Card {
    self.actor_info.card_at(index)
  }

  pub fn hero_camp(&self) -> Camp {
    self.actor_info.camp()
  }
}
