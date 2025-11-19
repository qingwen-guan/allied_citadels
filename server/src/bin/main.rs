use std::time::Instant;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use server::fa_agents::RedisProxyFAAgent;
use server::{
  AbstractFAAgent, AbstractFYIAgent, Config, Game, History, IdGen, NoopFYIAgent, Player, PlayerIndexedVec,
  RandomFAAgent, V2FAAgent, init_log,
};
use tokio::sync::mpsc;

async fn work() -> anyhow::Result<(f64, f64)> {
  let config = Config::load("config.toml")?;

  let mut rng = StdRng::seed_from_u64(rand::random());

  // let mut ws_dispatcher = WsDispatcher::new("127.0.0.1:7001".to_string());

  let (history_req_bcast_sender, mut history_req_bcast_receiver) = mpsc::channel::<String>(1024);
  // let history_uuid = uuid::Uuid::new_v4();
  let (_history_resp_sender, history_resp_receiver) = mpsc::channel::<String>(1024);
  println!("history_uuid: {}", config.history_uuid);
  // ws_dispatcher
  //   .add_end_point(config.history_uuid, history_req_bcast_receiver, history_resp_sender)
  //   .await;
  let /*mut*/ history = History::new(history_req_bcast_sender, history_resp_receiver);
  tokio::spawn(async move { while history_req_bcast_receiver.recv().await.is_some() {} });

  let id_gen = IdGen::new();

  let fallback: Box<dyn AbstractFAAgent> = Box::new(V2FAAgent::new());
  // let (ws_agent_req_bcast_sender, ws_agent_req_bcast_receiver) = mpsc::channel::<String>(1024);
  // let (ws_agent_resp_sender, ws_agent_resp_receiver) = mpsc::channel::<String>(1024);
  println!("ws_agent_uuid: {}", config.ws_agent_uuid);
  // ws_dispatcher
  //   .add_end_point(config.ws_agent_uuid, ws_agent_req_bcast_receiver, ws_agent_resp_sender)
  //   .await;
  let redis_client = redis::Client::open("redis://127.0.0.1:6379")?;
  let redis_conn: redis::aio::MultiplexedConnection = redis_client.get_multiplexed_async_connection().await?;
  let mut ws_agent = RedisProxyFAAgent::new(config.ws_agent_uuid, id_gen, redis_conn, fallback);

  // history.wait_for_ready().await;
  ws_agent.wait_for_ready().await;

  // TODO: wrap to fn
  let (num_players, players, agents, fyi_agents) = if rng.random_range(0..2) == 0 {
    let first = PlayerIndexedVec::from4(
      Player::new_汉(uuid::Uuid::new_v4(), "刘邦".to_string()),
      Player::new_楚(uuid::Uuid::new_v4(), "项羽".to_string()),
      Player::new_汉(uuid::Uuid::new_v4(), "张良".to_string()),
      Player::new_楚(uuid::Uuid::new_v4(), "范增".to_string()),
    );

    let second = PlayerIndexedVec::<Box<dyn AbstractFAAgent>>::from4(
      Box::new(ws_agent),
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
      Box::new(ws_agent),
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

  Ok((result.0, result.1))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let _guard = init_log("main");

  let num_games = 1;

  let start = Instant::now();

  let (win_rate_0, win_rate_1) = work().await?;

  let duration = start.elapsed();
  println!("num_games: {}", num_games);
  println!("Time taken: {:?}", duration);
  println!("win rate 楚: {}", win_rate_0 / num_games as f64);
  println!("win rate 汉: {}", win_rate_1 / num_games as f64);

  println!("wait 3 seconds for gracefully shutdown");
  tokio::time::sleep(std::time::Duration::from_secs(3)).await;

  Ok(())
}
