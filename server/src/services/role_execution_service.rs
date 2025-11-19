use std::cmp;
use std::cmp::Ordering;

use crate::abstract_fa_agent::AbstractFAAgent;
use crate::abstract_fyi_agent::AbstractFYIAgent;
use crate::deck::Deck;
use crate::domain::{Card, Color, DestroyTarget, MagicianSkill, Oper, PlayerIndex, PlayerOffset, Role, RoleSet};
use crate::game::RoundStats;
use crate::history::History;
use crate::obs::Obs;
use crate::player::Player;
use crate::player_indexed_vec::PlayerIndexedVec;

pub struct RoleExecutionService<'a> {
  pub num_players: usize,
  pub observes: &'a mut PlayerIndexedVec<Obs>,
  pub fyi_agents: &'a mut PlayerIndexedVec<Box<dyn AbstractFYIAgent>>,
  pub fa_agents: &'a mut PlayerIndexedVec<Box<dyn AbstractFAAgent>>,
  pub players: &'a mut PlayerIndexedVec<Player>,
  pub history: &'a mut History,
  pub round_stats: &'a mut RoundStats,
  pub deck: &'a mut Deck,
}

impl<'a> RoleExecutionService<'a> {
  pub async fn run(&mut self) {
    for role in Role::population() {
      let mut actor = None;

      for player in self.players.iter() {
        let player_role = player.role();
        if player_role == role {
          let player_index = player.index();
          actor = Some(player_index);
          break;
        }
      }

      if let Some(actor) = actor {
        self.history.reveal_role(actor, self.round_stats.round, role).await;

        for (observer, obs) in self.observes.iter_mut().enumerate() {
          let observer = PlayerIndex::from_usize(observer);
          if observer != actor {
            let offset = PlayerOffset::from_index(actor, observer, self.num_players);
            obs.set_villain_role(offset, role);
            self.fyi_agents[observer].villain_choose_role_resped(offset, role).await;
          }
        }

        self.execute_player_turn(actor).await;
      }

      if role == Role::小偷 {
        self.round_stats.stealer = actor;
      }
    }
  }

  async fn execute_player_turn(&mut self, actor: PlayerIndex) {
    if self.players[actor].role() == Role::国王 {
      self.round_stats.crown = actor;

      self
        .history
        .move_crown(self.round_stats.round, self.round_stats.crown)
        .await;

      for observer in (0..self.num_players).map(PlayerIndex::from_usize) {
        self.observes[observer].set_crown(PlayerOffset::from_index(actor, observer, self.num_players));
        self.fyi_agents[observer].obs_changed(&self.observes[observer]).await;
      }
    }

    if self.round_stats.killed == self.players[actor].role() {
      self.history.skip_killed_turn(actor, self.round_stats.round).await;

      for observer in (0..self.num_players).map(PlayerIndex::from_usize) {
        self.observes[observer].set_killed(PlayerOffset::from_index(actor, observer, self.num_players));
        self.fyi_agents[observer].obs_changed(&self.observes[observer]).await;
      }

      return;
    }

    if self.round_stats.stolen == self.players[actor].role() {
      let player_gold = self.players[actor].gold();
      self.players[self.round_stats.stealer.unwrap()].add_gold(player_gold);
      self.players[actor].set_gold(0);

      self
        .history
        .steal_gold(
          actor,
          self.round_stats.stealer.unwrap(),
          self.round_stats.round,
          player_gold,
        )
        .await;

      for observer in (0..self.num_players).map(PlayerIndex::from_usize) {
        self.observes[observer].set_stolen(PlayerOffset::from_index(actor, observer, self.num_players));
        self.fyi_agents[observer].obs_changed(&self.observes[observer]).await;
      }
    }

    match self.players[actor].role() {
      Role::刺客 => {
        let mut choices = RoleSet::universal();
        let banned_roles = self.round_stats.pub_drop_roles | Role::刺客;
        choices -= banned_roles;

        // self.observes[actor].update_infos(&self.deck, &self.players, actor);

        let history_id = self.history.kill_req(actor, &self.observes[actor], choices).await;
        let chosen_role = self.fa_agents[actor]
          .choose_kill_target(&self.observes[actor], choices)
          .await;
        self.history.kill_resp(history_id, chosen_role).await;

        self.round_stats.killed = chosen_role.into();
        for observer in (0..self.num_players).map(PlayerIndex::from_usize) {
          self.observes[observer].set_killed_role(chosen_role);
          self.fyi_agents[observer].obs_changed(&self.observes[observer]).await;
        }
      },
      Role::小偷 => {
        let mut choices = RoleSet::universal();
        let banned_roles = self.round_stats.pub_drop_roles | Role::刺客 | self.round_stats.killed | Role::小偷;
        choices -= banned_roles;

        // self.observes[actor].update_infos(&self.deck, &self.players, actor);

        let history_id = self.history.steal_req(actor, &self.observes[actor], choices).await;
        let chosen_role = self.fa_agents[actor]
          .choose_steal_target(&self.observes[actor], choices)
          .await;
        self.history.steal_resp(history_id, chosen_role).await;

        self.round_stats.stolen = chosen_role.into();
        for observer in (0..self.num_players).map(PlayerIndex::from_usize) {
          self.observes[observer].set_stolen_role(chosen_role);
          self.fyi_agents[observer].obs_changed(&self.observes[observer]).await;
        }
      },
      Role::魔术师 => {
        let history_id = self.history.magic_req(actor, &self.observes[actor]).await;
        let chosen_skill = self.fa_agents[actor].choose_swap_target(&self.observes[actor]).await;
        self.history.magic_resp(history_id, &chosen_skill).await;

        match chosen_skill {
          MagicianSkill::Swap(offset) => {
            let i = PlayerOffset::ZERO.to_index(actor, self.num_players); // TODO: extract n
            let j = offset.to_index(actor, self.num_players);
            let card_len_1 = self.players[i].cards_len();
            let card_len_2 = self.players[j].cards_len();
            match i.cmp(&j) {
              Ordering::Less => {
                let (left_players, right_players) = self.players.split_at_mut(j);
                let left_cards = left_players[i.value()].cards_mut();
                let right_cards = right_players[0].cards_mut();
                std::mem::swap(left_cards, right_cards);
              },
              Ordering::Equal => {
                panic!()
              },
              Ordering::Greater => {
                let (left_players, right_players) = self.players.split_at_mut(i);
                let left_cards = left_players[j.value()].cards_mut();
                let right_cards = right_players[0].cards_mut();
                std::mem::swap(left_cards, right_cards);
              },
            }

            assert!(self.players[i].cards_len() == card_len_2);
            assert!(self.players[j].cards_len() == card_len_1);

            self.history.swap_cards(actor, self.round_stats.round, i, j).await;

            self.update_observe_infos().await;
          },
          MagicianSkill::制衡(cards) => {
            let removed = self.players[actor].remove_cards(cards, self.deck);
            let drawn = self.players[actor]
              .draw_card(removed.len(), self.deck, self.history)
              .await;

            self
              .history
              .replace_cards(actor, self.round_stats.round, removed, drawn)
              .await;

            self.update_observe_infos().await;
          },
          MagicianSkill::放弃 => {},
        }
      },
      Role::商人 => {
        self.history.merchant(actor, self.round_stats.round).await;
        self.players[actor].add_gold(1);

        self.update_observe_infos().await;
      },
      Role::建筑师 => {
        let c0 = self.deck.take(self.history).await;
        let c1 = self.deck.take(self.history).await;

        self
          .history
          .architect_draw_2_cards(actor, self.round_stats.round, c0, c1)
          .await;

        self.players[actor].add_option_card(c0);
        self.players[actor].add_option_card(c1);
        self.update_observe_infos().await;
      },
      Role::军阀 => {
        let mut choices = Vec::new();
        for i in (0..self.num_players).map(PlayerIndex::from_usize) {
          if i == actor {
            continue;
          }

          for b in self.players[i].iter_buildings() {
            let destroy_fee = self.players[i].building_destroy_fee(b);
            if let Some(destroy_fee) = destroy_fee
              && destroy_fee <= self.players[actor].gold()
            {
              choices.push(DestroyTarget {
                player_offset: PlayerOffset::from_index(i, actor, self.num_players),
                card: b,
              });
            }
          }
        }

        // self.observes[actor].update_infos(&self.deck, &self.players, actor);

        let history_id = self.history.destroy_req(actor, &self.observes[actor], &choices).await;
        let target = self.fa_agents[actor]
          .choose_destory_target(&self.observes[actor], &choices)
          .await;
        let (chosen_offset, chosen_card) = match target {
          Some(target) => (
            Some(target.player_offset.to_index(actor, self.num_players)),
            Some(target.card),
          ),
          None => (None, None),
        };
        self.history.destroy_resp(history_id, chosen_offset, chosen_card).await;

        if let Some(target) = target {
          let player_index = target.player_offset.to_index(actor, self.num_players);
          let destroy_fee = self.players[player_index].building_destroy_fee(target.card).unwrap();
          self.players[player_index].remove_building(target.card);
          self.players[actor].sub_gold(destroy_fee);

          self.update_observe_infos().await;

          let who_has_tomb = self.who_has_tomb();

          match who_has_tomb {
            Some(who_has_tomb) => {
              if who_has_tomb != actor && self.players[who_has_tomb].gold() >= 1 {
                self.observes[who_has_tomb].update_infos(self.deck, self.players, actor);

                let history_id = self
                  .history
                  .tomb_req(actor, &self.observes[who_has_tomb], target.card)
                  .await;
                let chosen = self.fa_agents[actor]
                  .choose_tomb(&self.observes[who_has_tomb], target.card)
                  .await;
                self.history.tomb_resp(history_id, chosen).await;

                if chosen {
                  self.players[who_has_tomb].sub_gold(1);
                  self.players[who_has_tomb].add_card(target.card);
                } else {
                  self.deck.drop(target.card);
                }
              } else {
                self.deck.drop(target.card);
              }
            },
            None => {
              self.deck.drop(target.card);
            },
          };

          self.update_observe_infos().await;
        }
      },
      _ => {},
    }

    let mut got_resources = false;
    let mut has_built_times = 0;
    let mut has_bought_card = false;
    let mut has_sold_card = false;
    loop {
      let mut choices = vec![Oper::EndRound];

      if !got_resources {
        if self.players[actor].has_building(Card::天文台) || self.players[actor].has_building(Card::图书馆) {
          if self.players[actor].has_building(Card::天文台) {
            choices.push(Oper::Card3Choose1);
          }

          if self.players[actor].has_building(Card::图书馆) {
            choices.push(Oper::Card2Choose2);
          }
        } else {
          choices.push(Oper::Card2Choose1);
        }

        let get_gold_amount = match self.players[actor].role() {
          Role::国王 => 2 + self.players[actor].building_cnt(Color::黄),
          Role::主教 => 2 + self.players[actor].building_cnt(Color::蓝),
          Role::商人 => 2 + self.players[actor].building_cnt(Color::绿),
          Role::军阀 => 2 + self.players[actor].building_cnt(Color::红),
          _ => 2,
        };

        choices.push(Oper::Gold(get_gold_amount));
      }

      let build_quota = cmp::min(
        if self.players[actor].role() == Role::建筑师 {
          3
        } else {
          1
        } - has_built_times,
        8 - self.players[actor].buildings_len() as u32,
      );

      if build_quota > 0 {
        for p in 0..self.players[actor].cards_len() {
          let card = self.players[actor].card_at(p);

          if card.fee() > self.players[actor].gold() {
            continue;
          }
          if self.players[actor].has_building(card) {
            continue;
          }

          let mut is_unique = true;

          for q in 0..p {
            if card == self.players[actor].card_at(q) {
              is_unique = false;
              break;
            }
          }

          if is_unique {
            choices.push(Oper::Build(card));
          }
        }
      }

      if !has_bought_card && self.players[actor].has_building(Card::铁匠铺) && self.players[actor].gold() >= 2 {
        choices.push(Oper::BuyCard);
      }

      if !has_sold_card && self.players[actor].has_building(Card::实验室) && self.players[actor].cards_len() > 0 {
        for p in 0..self.players[actor].cards_len() {
          let mut is_unique = true;

          for q in 0..p {
            if self.players[actor].card_at(p) == self.players[actor].card_at(q) {
              is_unique = false;
              break;
            }
          }

          if is_unique {
            choices.push(Oper::SellCard(self.players[actor].card_at(p)));
          }
        }
      }

      if choices.is_empty() {
        break;
      }

      // self.observes[actor].update_infos(&self.deck, &self.players, actor);

      let history_id = self.history.oper_req(actor, &self.observes[actor], &choices).await;
      let chosen_operation = self.fa_agents[actor].choose_oper(&self.observes[actor], &choices).await;
      self.history.oper_resp(history_id, chosen_operation).await;

      match chosen_operation {
        Oper::EndRound => {
          break;
        },
        Oper::Card2Choose2 => {
          let c0 = self.deck.take(self.history).await;
          let c1 = self.deck.take(self.history).await;
          self.history.draw_2_cards(self.round_stats.round, actor, c0, c1).await;
          self.players[actor].add_option_card(c0);
          self.players[actor].add_option_card(c1);

          got_resources = true;
        },
        Oper::Card3Choose1 => {
          let c0 = self.deck.take(self.history).await;
          let c1 = self.deck.take(self.history).await;
          let c2 = self.deck.take(self.history).await;
          self
            .history
            .peek_3_cards(actor, self.round_stats.round, c0, c1, c2)
            .await;

          self.observes[actor].update_infos(self.deck, self.players, actor);
          self.players[actor]
            .choose_from_3(
              &mut self.fa_agents[actor],
              &self.observes[actor],
              [c0, c1, c2],
              self.deck,
              self.history,
            )
            .await;

          got_resources = true;
        },
        Oper::Card2Choose1 => {
          let c0 = self.deck.take(self.history).await;
          let c1 = self.deck.take(self.history).await;
          self.history.peek_2_cards(actor, self.round_stats.round, c0, c1).await;

          self.observes[actor].update_infos(self.deck, self.players, actor);
          self.players[actor]
            .choose_from_2(
              &mut self.fa_agents[actor],
              &self.observes[actor],
              c0,
              c1,
              self.deck,
              self.history,
            )
            .await;
          got_resources = true;
        },
        Oper::Gold(amount) => {
          self.history.gold(actor, self.round_stats.round, amount).await;
          self.players[actor].add_gold(amount);
          got_resources = true;
        },
        Oper::Build(card) => {
          self.history.build(actor, self.round_stats.round, card).await;
          self.players[actor].build(card);
          if self.players[actor].buildings_len() == 8 {
            if !self.round_stats.has_first_8_buildings {
              self.round_stats.has_first_8_buildings = true;
              self.players[actor].set_is_first_8_buildings();
              self.history.first_8_buildings(actor, self.round_stats.round).await;
            } else {
              self.history.nonfirst_8_buildings(actor, self.round_stats.round).await;
            }
          }

          has_built_times += 1;
        },
        Oper::SellCard(card) => {
          self.history.sell_card(actor, self.round_stats.round, card).await;

          self.players[actor].remove_first_card(card);
          self.deck.drop(card);
          self.players[actor].add_gold(1);

          has_sold_card = true;
        },
        Oper::BuyCard => {
          // TODO: 花钱
          let c0 = self.deck.take(self.history).await;
          let c1 = self.deck.take(self.history).await;
          let c2 = self.deck.take(self.history).await;
          self
            .history
            .draw_3_cards(actor, self.round_stats.round, c0, c1, c2)
            .await;

          self.players[actor].add_option_card(c0);
          self.players[actor].add_option_card(c1);
          self.players[actor].add_option_card(c2);

          has_bought_card = true;
        },
      }

      self.update_observe_infos().await;
    }

    self.check_total_card_number();
  }

  async fn update_observe_infos(&mut self) {
    for observer in (0..self.num_players).map(PlayerIndex::from_usize) {
      self.observes[observer].update_infos(self.deck, self.players, observer);
      self.fyi_agents[observer].obs_changed(&self.observes[observer]).await;
    }
  }

  pub fn check_total_card_number(&self) {
    // TODO: remove
    let mut total = 0;
    total += self.deck.peek_deck().len();
    total += self.deck.peek_drop().len();
    for player in self.players.iter() {
      total += player.cards_len();
      total += player.buildings_len();
    }
    assert_eq!(total, 66);
  }

  fn who_has_tomb(&mut self) -> Option<PlayerIndex> {
    (0..self.num_players)
      .map(PlayerIndex::from_usize)
      .find(|&i| self.players[i].has_building(Card::墓地))
  }
}
