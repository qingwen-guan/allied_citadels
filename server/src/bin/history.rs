use futures_util::{SinkExt, StreamExt};
use server::{Config, HistoryReqEvent, HistoryRespEvent, init_log};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

async fn work() -> anyhow::Result<()> {
  let config = Config::load("config.toml")?;

  let addr = format!("ws://{}:{}/{}", config.host, config.port, config.history_uuid);
  println!("addr: {}", addr);

  let (mut ws_stream, _) = connect_async(&addr).await.unwrap();

  while let Some(msg) = ws_stream.next().await {
    match msg {
      Ok(Message::Close(close_frame)) => {
        println!("History stream closed: {:?}", close_frame);
        break;
      },
      Ok(Message::Text(text)) => {
        let event: HistoryReqEvent = serde_json::from_str(&text).unwrap();
        match event {
          HistoryReqEvent::WaitForReady { .. } => {
            println!("< {}", text);
            let event = HistoryRespEvent::Ready;
            let json = serde_json::to_string(&event).unwrap();
            println!("> {}", json);
            ws_stream.send(Message::Text(json.into())).await.unwrap();
          },
          _ => {
            println!("received other event");
          },
        }
      },
      Ok(_) => {
        println!("Unknown message");
      },
      Err(e) => {
        println!("Error: {}", e);
      },
    }
  }

  Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let _guard = init_log("history");

  work().await?;

  Ok(())
}
