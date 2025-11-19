use std::time::Instant;

use indicatif::ProgressBar;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use server::{
  AbstractFAAgent, AbstractFYIAgent, Game, History, NoopFYIAgent, Player, PlayerIndexedVec, RandomFAAgent, V2FAAgent,
  init_log,
};
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tracing::info;

async fn work() -> (f64, f64) {
  let mut rng = StdRng::seed_from_u64(rand::random());

  let (history_req_bcast_sender, mut history_req_bcast_receiver) = mpsc::channel::<String>(1024);
  // let history_uuid = uuid::Uuid::new_v4();
  let (_, history_resp_receiver) = mpsc::channel::<String>(1024);
  let history = History::new(history_req_bcast_sender, history_resp_receiver);

  tokio::spawn(async move {
    while let Some(event) = history_req_bcast_receiver.recv().await {
      info!("received history event: {:?}", event);
    }
  });

  // TODO: wrap to fn
  let (num_players, players, agents, fyi_agents) = if rng.random_range(0..2) == 0 {
    let first = PlayerIndexedVec::from4(
      Player::new_汉(uuid::Uuid::new_v4(), "刘邦".to_string()),
      Player::new_楚(uuid::Uuid::new_v4(), "项羽".to_string()),
      Player::new_汉(uuid::Uuid::new_v4(), "张良".to_string()),
      Player::new_楚(uuid::Uuid::new_v4(), "范增".to_string()),
    );

    let second = PlayerIndexedVec::<Box<dyn AbstractFAAgent>>::from4(
      Box::new(V2FAAgent::new()),
      Box::new(RandomFAAgent::new()),
      Box::new(V2FAAgent::new()),
      Box::new(RandomFAAgent::new()),
    );
    let third = PlayerIndexedVec::<Box<dyn AbstractFYIAgent>>::from4(
      Box::new(NoopFYIAgent::new()),
      Box::new(NoopFYIAgent::new()),
      Box::new(NoopFYIAgent::new()),
      Box::new(NoopFYIAgent::new()),
    );

    (4, first, second, third)
  } else {
    let first = PlayerIndexedVec::from6(
      Player::new_汉(uuid::Uuid::new_v4(), "刘邦".to_string()),
      Player::new_楚(uuid::Uuid::new_v4(), "项羽".to_string()),
      Player::new_汉(uuid::Uuid::new_v4(), "张良".to_string()),
      Player::new_楚(uuid::Uuid::new_v4(), "范增".to_string()),
      Player::new_汉(uuid::Uuid::new_v4(), "韩信".to_string()),
      Player::new_楚(uuid::Uuid::new_v4(), "龙且".to_string()),
    );

    let second = PlayerIndexedVec::<Box<dyn AbstractFAAgent>>::from6(
      Box::new(V2FAAgent::new()),
      Box::new(RandomFAAgent::new()),
      Box::new(V2FAAgent::new()),
      Box::new(RandomFAAgent::new()),
      Box::new(V2FAAgent::new()),
      Box::new(RandomFAAgent::new()),
    );
    let third = PlayerIndexedVec::<Box<dyn AbstractFYIAgent>>::from6(
      Box::new(NoopFYIAgent::new()),
      Box::new(NoopFYIAgent::new()),
      Box::new(NoopFYIAgent::new()),
      Box::new(NoopFYIAgent::new()),
      Box::new(NoopFYIAgent::new()),
      Box::new(NoopFYIAgent::new()),
    );

    (6, first, second, third)
  };

  let mut game = Game::new(num_players, players, agents, fyi_agents, rng, history).await;
  let result = game.run().await;

  (result.0, result.1)
}

#[tokio::main]
async fn main() {
  let _guard = init_log("sim");

  let mut win_rate = [0.0, 0.0];

  let num_games = 1000;

  let start = Instant::now();

  let pb = ProgressBar::new(num_games);

  let mut join_set = JoinSet::new();
  for _ in 0..num_games {
    join_set.spawn(work());
  }

  while let Some(result) = join_set.join_next().await {
    let (win_rate_0, win_rate_1) = result.unwrap();
    win_rate[0] += win_rate_0;
    win_rate[1] += win_rate_1;

    pb.inc(1);
  }

  let duration = start.elapsed();
  println!("num_games: {}", num_games);
  println!("Time taken: {:?}", duration);
  println!("win rate 楚: {}", win_rate[0] / num_games as f64);
  println!("win rate 汉: {}", win_rate[1] / num_games as f64);
}
