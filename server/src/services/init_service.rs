use crate::deck::Deck;
use crate::domain::{PlayerIndex, PlayerOffset};
use crate::obs::{HeroInfo, VillainInfo};
use crate::{AbstractFAAgent, AbstractFYIAgent, History, Obs, Player, PlayerIndexedVec};

pub struct InitService<'a> {
  pub players: &'a mut PlayerIndexedVec<Player>,
  pub history: &'a mut History,
  pub observes: &'a mut PlayerIndexedVec<Obs>,
  pub crown: PlayerIndex,
  pub deck: &'a mut Deck,
  pub fa_agents: &'a mut PlayerIndexedVec<Box<dyn AbstractFAAgent>>,
  pub fyi_agents: &'a mut PlayerIndexedVec<Box<dyn AbstractFYIAgent>>,
}

impl<'a> InitService<'a> {
  pub async fn run(&mut self) {
    self.init_gold().await;
    self.init_obs();
    self.init_card().await;
  }

  pub async fn init_gold(&mut self) {
    for player in self.players.iter_mut() {
      player.set_gold(2);
      self.history.init_gold(player.index(), player.gold()).await;
    }
  }

  pub fn init_obs(&mut self) {
    for actor in self.players.iter() {
      let mut villain_infos = Vec::new();
      for i in (actor.index().value() + 1..self.players.len()).map(PlayerIndex::from_usize) {
        villain_infos.push(VillainInfo::from(&self.players[i]));
      }
      for i in (0..actor.index().value()).map(PlayerIndex::from_usize) {
        villain_infos.push(VillainInfo::from(&self.players[i]));
      }

      let obs = Obs::new(
        self.players.len(),
        0,
        PlayerOffset::from_index(self.crown, actor.index(), self.players.len()),
        HeroInfo::from(actor),
        villain_infos,
        self.deck,
      );
      self.observes.push(obs);
    }
  }

  async fn init_card(&mut self) {
    let mut init_choices = Vec::new();
    let mut req_history_ids = PlayerIndexedVec::<u32>::with_len(self.players.len());

    for player in self.players.iter() {
      let actor = player.index();
      let c0 = self.deck.take(self.history).await.unwrap(); // 初始状态牌的数量肯定是够的
      let c1 = self.deck.take(self.history).await.unwrap(); // 初始状态牌的数量肯定是够的
      init_choices.push((actor, c0, c1));

      let history_id = self.history.init_card_req(actor, &self.observes[actor], c0, c1).await;
      req_history_ids[actor] = history_id;
    }

    use tokio::task::JoinSet;
    let mut join_set = JoinSet::new();

    // Create futures that don't borrow from self
    for (&(actor, c1, c2), agent) in init_choices.iter().zip(self.fa_agents.iter_mut()) {
      let obs = self.observes[actor].clone(); // Clone the observation to avoid borrowing
      let mut fa_agent = std::mem::replace(agent, Box::new(crate::fa_agents::NoopFAAgent::new())); // Move agent out temporarily
      let future = async move {
        let chosen = fa_agent.choose_init_card(&obs, c1, c2).await;
        let drop = if chosen == c1 { c2 } else { c1 };
        (actor, chosen, drop, fa_agent)
      };
      join_set.spawn(future);
    }

    // Handle responses on arrival
    while let Some(result) = join_set.join_next().await {
      let (actor, chosen, drop, fa_agent) = result.unwrap();
      self.fa_agents[actor] = fa_agent; // Put the agent back
      self
        .history
        .init_card_resp(req_history_ids[actor], actor, &self.observes[actor], chosen, drop)
        .await;
      self.players[actor].add_card(chosen);
      self.deck.drop(drop);

      for i in (0..self.players.len()).map(PlayerIndex::from_usize) {
        self.observes[i].update_infos(self.deck, self.players, i);
        self.fyi_agents[i].obs_changed(&self.observes[i]).await;
      }
    }
  }
}
