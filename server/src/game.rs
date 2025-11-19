use std::cmp::Ordering;

use rand::Rng;
use rand::rngs::StdRng;

use crate::RoleSelectService;
use crate::abstract_fa_agent::AbstractFAAgent;
use crate::abstract_fyi_agent::AbstractFYIAgent;
use crate::deck::Deck;
use crate::domain::{Camp, OptionRole, PlayerIndex, RoleSet};
use crate::history::History;
use crate::obs::Obs;
use crate::player::Player;
use crate::player_indexed_vec::PlayerIndexedVec;
use crate::services::{InitService, RoleExecutionService};

pub struct Game {
  num_players: usize,
  players: PlayerIndexedVec<Player>,
  fa_agents: PlayerIndexedVec<Box<dyn AbstractFAAgent>>,
  fyi_agents: PlayerIndexedVec<Box<dyn AbstractFYIAgent>>,
  crown: PlayerIndex,
  deck: Deck,
  observes: PlayerIndexedVec<Obs>,
  history: History,
}

pub struct RoundStats {
  pub round: u32, // round=0 for init
  pub pub_drop_roles: RoleSet,
  pub killed: OptionRole,
  pub stolen: OptionRole,
  pub stealer: Option<PlayerIndex>, // TODO: replace with offset
  pub has_first_8_buildings: bool,
  pub crown: PlayerIndex,
}

impl Game {
  pub async fn new(
    num_players: usize, mut players: PlayerIndexedVec<Player>, agents: PlayerIndexedVec<Box<dyn AbstractFAAgent>>,
    fyi_agents: PlayerIndexedVec<Box<dyn AbstractFYIAgent>>, mut rng: StdRng, mut history: History,
  ) -> Self {
    for (i, player) in players.iter_mut().enumerate() {
      player.set_index(PlayerIndex::from_usize(i));
    }
    let crown = PlayerIndex::from_usize(rng.random_range(0..players.len()));
    let deck = Deck::new(rng, &mut history).await;
    Self {
      num_players,
      players,
      fa_agents: agents,
      fyi_agents,
      crown,
      deck,
      observes: PlayerIndexedVec::<Obs>::new(),
      history,
    }
  }

  pub async fn run(&mut self) -> (f64, f64) {
    self.history.game_start(self.crown).await;

    InitService {
      players: &mut self.players,
      history: &mut self.history,
      observes: &mut self.observes,
      crown: self.crown,
      deck: &mut self.deck,
      fa_agents: &mut self.fa_agents,
      fyi_agents: &mut self.fyi_agents,
    }
    .run()
    .await;

    let mut round: u32 = 0;
    loop {
      round += 1;
      let mut has_8_buildings = false;
      self.run_round(round, &mut has_8_buildings).await;
      self.check_total_card_number();

      if has_8_buildings {
        self.history.finish_game().await;
        break;
      }
    }

    let mut total_score = [0, 0];

    for player in self.players.iter() {
      total_score[player.camp() as usize] += player.score();
    }
    // println!("total score 楚: {}", total_score[Camp::楚 as usize]);
    // println!("total score 汉: {}", total_score[Camp::汉 as usize]);

    match total_score[Camp::楚 as usize].cmp(&total_score[Camp::汉 as usize]) {
      Ordering::Greater => {
        // println!("楚胜");

        (1.0, 0.0)
      },
      Ordering::Less => {
        // println!("汉胜");

        (0.0, 1.0)
      },
      Ordering::Equal => {
        // println!("平局");

        (0.5, 0.5)
      },
    }
  }

  async fn run_round(&mut self, round: u32, has_8_buildings: &mut bool) {
    self.history.start_round(round, self.crown, &self.deck).await;
    for observer in (0..self.players.len()).map(PlayerIndex::from_usize) {
      self.observes[observer].set_round(round);
      self.fyi_agents[observer].obs_changed(&self.observes[observer]).await;
    }

    let mut round_stats = RoleSelectService {
      num_players: self.num_players,
      observes: &mut self.observes,
      fyi_agents: &mut self.fyi_agents,
      fa_agents: &mut self.fa_agents,
      players: &mut self.players,
      round,
      rng: self.deck.rng(),
      history: &mut self.history,
      crown: self.crown,
    }
    .run()
    .await;

    RoleExecutionService {
      num_players: self.num_players,
      observes: &mut self.observes,
      fyi_agents: &mut self.fyi_agents,
      fa_agents: &mut self.fa_agents,
      players: &mut self.players,
      history: &mut self.history,
      round_stats: &mut round_stats,
      deck: &mut self.deck,
    }
    .run()
    .await;

    self.crown = round_stats.crown;

    for player in self.players.iter_mut() {
      player.unset_role();
    }

    for obs in self.observes.iter_mut() {
      obs.reset();
    }

    *has_8_buildings = round_stats.has_first_8_buildings;
  }

  pub fn check_total_card_number(&self) {
    let mut total = 0;
    total += self.deck.peek_deck().len();
    total += self.deck.peek_drop().len();
    for player in self.players.iter() {
      total += player.cards_len();
      total += player.buildings_len();
    }
    assert_eq!(total, 66);
  }
}
