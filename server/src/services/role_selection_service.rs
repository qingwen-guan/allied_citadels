use rand::rngs::StdRng;

use crate::abstract_fa_agent::AbstractFAAgent;
use crate::abstract_fyi_agent::AbstractFYIAgent;
use crate::domain::PlayerIndex; // TODO: rename to Index
use crate::domain::{OptionRole, PlayerOffset, PlayerOffsetSet, RoleSet};
use crate::game::RoundStats;
use crate::history::History;
use crate::obs::Obs;
use crate::player::Player;
use crate::player_indexed_vec::PlayerIndexedVec;

pub struct RoleSelectService<'a> {
  pub num_players: usize,
  pub observes: &'a mut PlayerIndexedVec<Obs>,
  pub fyi_agents: &'a mut PlayerIndexedVec<Box<dyn AbstractFYIAgent>>,
  pub fa_agents: &'a mut PlayerIndexedVec<Box<dyn AbstractFAAgent>>,
  pub players: &'a mut PlayerIndexedVec<Player>,
  pub round: u32,
  pub rng: &'a mut StdRng,
  pub history: &'a mut History,
  pub crown: PlayerIndex,
}

impl<'a> RoleSelectService<'a> {
  pub async fn run(&mut self) -> RoundStats {
    let mut round_stats = RoundStats {
      round: self.round,
      pub_drop_roles: RoleSet::empty(),
      killed: OptionRole::None,
      stolen: OptionRole::None,
      stealer: None,
      has_first_8_buildings: false,
      crown: self.crown,
    };

    let mut roles = RoleSet::universal();
    if self.num_players == 4 {
      let pub_drop_role_0 = roles.random_choose(self.rng);
      roles -= pub_drop_role_0;

      let pub_drop_role_1 = roles.random_choose(self.rng);
      roles -= pub_drop_role_1;

      round_stats.pub_drop_roles = RoleSet::from_pair(pub_drop_role_0, pub_drop_role_1);

      self
        .history
        .public_drop_roles(self.round, round_stats.pub_drop_roles)
        .await;

      for i in (0..self.num_players).map(PlayerIndex::from_usize) {
        self.observes[i].set_roles_public_dropped(round_stats.pub_drop_roles);
        self.fyi_agents[i].obs_changed(&self.observes[i]).await;
      }
    }

    let mut roles_chosen = RoleSet::empty();

    {
      let drop_role = roles.random_choose(self.rng);
      roles -= drop_role;

      roles_chosen |= drop_role;
      self.history.secret_first_drop_role(self.round, drop_role).await;
      for fyi_agent in self.fyi_agents.iter_mut() {
        fyi_agent.first_role_dropped().await;
      }
    }

    let player_indices = {
      let mut v = (0..self.num_players).map(PlayerIndex::from_usize).collect::<Vec<_>>();
      v.rotate_left(self.crown.value());
      v
    };
    assert!(player_indices[0] == self.crown);

    for (index, &actor) in player_indices.iter().enumerate() {
      for (jndex, fyi_agent) in self.fyi_agents.iter_mut().enumerate() {
        fyi_agent
          .villain_choose_role_reqed(
            PlayerOffset::from_index(actor, PlayerIndex::from_usize(jndex), self.num_players),
            roles.len(),
          )
          .await;
      }

      {
        let mut player_offsets = PlayerOffsetSet::empty();
        (0..index).for_each(|idx| {
          let player_index = player_indices[idx];
          let offset = PlayerOffset::from_index(player_index, actor, self.num_players);
          player_offsets |= offset;
        });

        self.observes[actor].set_players_choose_role_before(player_offsets);
      }

      {
        let mut player_offsets = PlayerOffsetSet::empty();
        (index + 1..player_indices.len()).for_each(|index| {
          let player_index = player_indices[index];
          let offset = PlayerOffset::from_index(player_index, actor, self.num_players);
          player_offsets |= offset;
        });
        self.observes[actor].set_players_choose_role_after(player_offsets);
      }

      self.observes[actor].set_roles_chosen_before(roles_chosen);

      self.history.choose_role_req(actor, &self.observes[actor], roles).await;
      let chosen = self.fa_agents[actor].choose_role(&self.observes[actor], roles).await;
      self
        .history
        .choose_role_resp(actor, &self.observes[actor], roles, chosen)
        .await;

      self.players[actor].set_role(chosen);
      roles_chosen |= chosen;
      roles -= chosen;

      self.observes[actor].set_roles_chosen_after(roles);
      self.observes[actor].set_actor_role(chosen);
    }

    {
      assert!(roles.len() == 1);
      let role = roles.into();

      self.history.secret_last_drop_role(self.round, role).await;

      for fyi_agent in self.fyi_agents.iter_mut() {
        fyi_agent.last_role_dropped().await;
      }

      round_stats
    }
  }
}
