use uuid::Uuid;

use crate::abstract_fa_agent::AbstractFAAgent;
use crate::deck::Deck;
use crate::domain::{Camp, Card, Color, PlayerIndex, Role};
use crate::history::History;
use crate::obs::Obs;

pub struct Player {
  index: PlayerIndex,
  uuid: Uuid,
  name: String,
  camp: Camp,
  gold: u32,
  cards: Vec<Card>,
  buildings: Vec<Card>,
  is_first_8_buildings: bool,
  role: Option<Role>,
}

impl Player {
  pub fn new_楚(uuid: Uuid, name: String) -> Self {
    Self::new(uuid, name, Camp::楚)
  }

  pub fn new_汉(uuid: Uuid, name: String) -> Self {
    Self::new(uuid, name, Camp::汉)
  }

  pub fn new(uuid: Uuid, name: String, camp: Camp) -> Self {
    Self {
      index: PlayerIndex::invalid(),
      uuid,
      name,
      camp,
      gold: 0,
      cards: Vec::new(),
      buildings: Vec::new(),
      is_first_8_buildings: false,
      role: None,
    }
  }

  pub fn get_name(&self) -> &str {
    // TODO: rename to name
    &self.name
  }

  pub fn uuid(&self) -> Uuid {
    self.uuid
  }

  pub fn camp(&self) -> Camp {
    self.camp
  }

  pub fn set_role(&mut self, role: Role) {
    assert!(self.role.is_none());
    self.role = Some(role);
  }

  pub fn unset_role(&mut self) {
    assert!(self.role.is_some());
    self.role = None;
  }

  pub fn index(&self) -> PlayerIndex {
    self.index
  }

  pub fn set_index(&mut self, index: PlayerIndex) {
    self.index = index;
  }

  pub fn role(&self) -> Role {
    self.role.unwrap()
  }

  pub fn gold(&self) -> u32 {
    self.gold
  }

  pub fn set_gold(&mut self, gold: u32) {
    self.gold = gold;
  }

  pub fn add_gold(&mut self, amount: u32) {
    self.gold += amount;
  }

  pub fn sub_gold(&mut self, amount: u32) {
    self.gold -= amount;
  }

  pub fn cards_len(&self) -> usize {
    self.cards.len()
  }

  pub fn buildings_len(&self) -> usize {
    self.buildings.len()
  }

  pub fn remove_building(&mut self, c: Card) {
    self.buildings.retain(|v: &Card| *v != c);
  }

  pub fn card_at(&self, idx: usize) -> Card {
    self.cards[idx]
  }

  pub fn cards_mut(&mut self) -> &mut Vec<Card> {
    &mut self.cards
  }

  pub fn cards(&self) -> &Vec<Card> {
    &self.cards
  }

  pub fn add_card(&mut self, c: Card) {
    self.cards.push(c);
  }

  pub fn add_option_card(&mut self, c: Option<Card>) {
    if let Some(c) = c {
      self.cards.push(c);
    }
  }

  pub fn remove_cards(&mut self, cards: Vec<Card>, deck: &mut Deck) -> Vec<Card> {
    let mut removed = Vec::new();

    for c in cards {
      let ok = self.remove_first_card(c);
      assert!(ok);
      deck.drop(c);
      removed.push(c);
    }

    removed
  }

  // None表示不可以拆
  pub fn building_destroy_fee(&self, c: Card) -> Option<u32> {
    let role = self.role;
    match role {
      Some(Role::主教) | Some(Role::军阀) => return None,
      _ => {},
    };
    if self.buildings.len() == 8 {
      return None;
    };

    match c {
      Card::要塞 => None,
      Card::城墙 => Some(c.fee()),
      _ => {
        if self.has_building(Card::城墙) {
          Some(c.fee())
        } else {
          Some(c.fee() - 1)
        }
      },
    }
  }

  pub fn has_building(&self, c: Card) -> bool {
    self.buildings.contains(&c)
  }

  pub fn building_cnt(&self, color: Color) -> u32 {
    self.buildings.iter().filter(|c: &&Card| c.color() == color).count() as u32
  }

  pub fn iter_buildings(&self) -> impl Iterator<Item = Card> {
    self.buildings.iter().copied()
  }

  pub fn set_is_first_8_buildings(&mut self) {
    self.is_first_8_buildings = true;
  }

  pub fn has_all_colors(&self) -> bool {
    let mut has_color = [false, false, false, false, false];
    for c in self.buildings.iter() {
      has_color[c.color() as usize] = true;
    }
    has_color.iter().all(|&c| c)
  }

  pub fn is_first_8_buildings(&self) -> bool {
    self.is_first_8_buildings
  }

  pub fn extra_score(&self) -> u32 {
    let mut value = 0;
    if self.has_all_colors() {
      value += 3;
    }
    if self.buildings.len() == 8 {
      value += 2;
    }
    if self.is_first_8_buildings {
      value += 2;
    }
    value
  }

  pub fn score(&self) -> u32 {
    let mut score = 0;
    for c in self.buildings.iter() {
      score += c.score();
    }
    score += self.extra_score();
    score
  }

  pub async fn choose_from_2(
    &mut self, fa_agent: &mut Box<dyn AbstractFAAgent>, obs: &Obs, c0: Option<Card>, c1: Option<Card>,
    deck: &mut Deck, history: &mut History,
  ) {
    let c0 = match c0 {
      Some(c) => c,
      None => return,
    };
    let c1 = match c1 {
      Some(c) => c,
      None => {
        history.choose_from_1(self.index, obs.round(), c0).await;
        self.cards.push(c0);
        return;
      },
    };

    let history_id = history.choose_from_2_req(self.index, obs, c0, c1).await;
    let chosen = fa_agent.choose_from_2(obs, c0, c1).await;
    let drop = if chosen == c0 { c1 } else { c0 };
    history.choose_from_2_resp(history_id, chosen, drop).await;

    self.cards.push(chosen);
    deck.drop(drop);
  }

  pub async fn choose_from_3(
    &mut self, agent: &mut Box<dyn AbstractFAAgent>, obs: &Obs, cards: [Option<Card>; 3], deck: &mut Deck,
    history: &mut History,
  ) {
    let c0 = match cards[0] {
      Some(c) => c,
      None => return,
    };
    let c1 = match cards[1] {
      Some(c) => c,
      None => {
        history.choose_from_1(self.index, obs.round(), c0).await;
        self.cards.push(c0);
        return;
      },
    };
    match cards[2] {
      Some(c2) => {
        let history_id = history.choose_from_3_req(self.index, obs, c0, c1, c2).await;
        let chosen = agent.choose_from_3(obs, c0, c1, c2).await;
        let (drop0, drop1) = if chosen == c0 {
          (c1, c2)
        } else if chosen == c1 {
          (c0, c2)
        } else {
          (c0, c1)
        };

        history.choose_from_3_resp(history_id, chosen, drop0, drop1).await;

        self.cards.push(chosen);
        deck.drop(drop0);
        deck.drop(drop1);
      },
      None => {
        let history_id = history.choose_from_2_req(self.index, obs, c0, c1).await;
        let chosen = agent.choose_from_2(obs, c0, c1).await;
        let drop = if chosen == c0 { c1 } else { c0 };
        history.choose_from_2_resp(history_id, chosen, drop).await;

        self.cards.push(chosen);
        deck.drop(drop);
      },
    }
  }

  pub fn build(&mut self, card: Card) {
    self.remove_first_card(card);
    self.buildings.push(card);
    self.sub_gold(card.fee());
  }

  // TODO: must use return value
  pub fn remove_first_card(&mut self, c: Card) -> bool {
    match self.cards.iter().position(|p| *p == c) {
      Some(index) => {
        self.cards.remove(index);
        true
      },
      None => false,
    }
  }

  pub async fn draw_card(&mut self, n: usize, deck: &mut Deck, history: &mut History) -> Vec<Card> {
    let mut drawn = Vec::new();

    for _ in 0..n {
      if let Some(c) = deck.take(history).await {
        self.cards.push(c);
        drawn.push(c);
      }
    }

    drawn
  }
}
