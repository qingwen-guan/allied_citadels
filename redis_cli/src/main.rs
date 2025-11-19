use redis::AsyncCommands;

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
  println!("Hello, world!");

  let redis_client = redis::Client::open("redis://127.0.0.1:6379")?;
  let redis_conn = redis_client.get_multiplexed_async_connection().await?;

  let mut redis_conn = redis_conn.clone();

  let _: () = redis_conn.set("key", "value").await?;
  let value: String = redis_conn.get("key").await?;
  println!("value: {}", value);

  Ok(())
}
