use futures_util::{SinkExt, StreamExt};
use server::{AbstractFAAgent, Config, V2FAAgent, domain, init_log};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

async fn work() -> anyhow::Result<()> {
  let config = Config::load("config.toml")?;

  let mut agent = V2FAAgent::new();

  let addr = format!("ws://{}:{}/{}", config.host, config.port, config.ws_agent_uuid);
  println!("addr: {}", addr);

  let (mut ws_stream, _) = connect_async(&addr).await.unwrap();

  while let Some(msg) = ws_stream.next().await {
    match msg {
      Ok(Message::Close(close_frame)) => {
        println!("WsAgent stream closed: {:?}", close_frame);
        break;
      },
      Ok(Message::Text(text)) => {
        let event: domain::AgentReqEvent = serde_json::from_str(&text).unwrap();
        match event {
          domain::AgentReqEvent::WaitForReady { id } => {
            // TODO: only print on verbose mode
            println!("< {}", text.as_str());
            let event = domain::AgentRespEvent::WaitForReady { id };
            let json = serde_json::to_string(&event).unwrap();
            println!("> {}", json);
            ws_stream.send(Message::Text(json.into())).await.unwrap();
          },
          domain::AgentReqEvent::ChooseInitCard { id, obs, c0, c1 } => {
            println!("< {}", text.as_str());
            let chosen = agent.choose_init_card(&obs, c0, c1).await;
            let event = domain::AgentRespEvent::InitCard { id, chosen };
            let json = serde_json::to_string(&event).unwrap();
            println!("> {}", json);
            ws_stream.send(Message::Text(json.into())).await.unwrap();
          },
          domain::AgentReqEvent::ChooseRole { id, obs, roles } => {
            println!("< {}", text.as_str());
            let chosen = agent.choose_role(&obs, roles).await;
            let event = domain::AgentRespEvent::Role { id, chosen };
            let json = serde_json::to_string(&event).unwrap();
            println!("> {}", json);
            ws_stream.send(Message::Text(json.into())).await.unwrap();
          },
          domain::AgentReqEvent::ChooseKillTarget { id, obs, choices } => {
            println!("< {}", text.as_str());
            let chosen = agent.choose_kill_target(&obs, choices).await;
            let event = domain::AgentRespEvent::KillTarget { id, chosen };
            let json = serde_json::to_string(&event).unwrap();
            println!("> {}", json);
            ws_stream.send(Message::Text(json.into())).await.unwrap();
          },
          domain::AgentReqEvent::ChooseStealTarget { id, obs, choices } => {
            println!("< {}", text.as_str());
            let chosen = agent.choose_steal_target(&obs, choices).await;
            let event = domain::AgentRespEvent::StealTarget { id, chosen };
            let json = serde_json::to_string(&event).unwrap();
            println!("> {}", json);
            ws_stream.send(Message::Text(json.into())).await.unwrap();
          },
          domain::AgentReqEvent::ChooseMagicTarget { id, obs } => {
            println!("< {}", text.as_str());
            let chosen = agent.choose_swap_target(&obs).await;
            let event = domain::AgentRespEvent::MagicTarget { id, chosen };
            let json = serde_json::to_string(&event).unwrap();
            println!("> {}", json);
            ws_stream.send(Message::Text(json.into())).await.unwrap();
          },
          domain::AgentReqEvent::ChooseDestoryTarget { id, obs, choices } => {
            println!("< {}", text.as_str());
            let chosen = agent.choose_destory_target(&obs, &choices).await;
            let event = domain::AgentRespEvent::DestoryTarget { id, chosen };
            let json = serde_json::to_string(&event).unwrap();
            println!("> {}", json);
            ws_stream.send(Message::Text(json.into())).await.unwrap();
          },
          domain::AgentReqEvent::ChooseTomb { id, obs, c } => {
            println!("< {}", text.as_str());
            let chosen = agent.choose_tomb(&obs, c).await;
            let event = domain::AgentRespEvent::Tomb { id, chosen };
            let json = serde_json::to_string(&event).unwrap();
            println!("> {}", json);
            ws_stream.send(Message::Text(json.into())).await.unwrap();
          },
          domain::AgentReqEvent::ChooseOper { id, obs, choices } => {
            println!("< {}", text.as_str());
            let chosen = agent.choose_oper(&obs, &choices).await;
            let event = domain::AgentRespEvent::Oper { id, chosen };
            let json = serde_json::to_string(&event).unwrap();
            println!("> {}", json);
            ws_stream.send(Message::Text(json.into())).await.unwrap();
          },
          domain::AgentReqEvent::ChooseFrom2 { id, obs, c0, c1 } => {
            println!("< {}", text.as_str());
            let chosen = agent.choose_from_2(&obs, c0, c1).await;
            let event = domain::AgentRespEvent::From2 { id, chosen };
            let json = serde_json::to_string(&event).unwrap();
            println!("> {}", json);
            ws_stream.send(Message::Text(json.into())).await.unwrap();
          },
          domain::AgentReqEvent::ChooseFrom3 { id, obs, c0, c1, c2 } => {
            println!("< {}", text.as_str());
            let chosen = agent.choose_from_3(&obs, c0, c1, c2).await;
            let event = domain::AgentRespEvent::From3 { id, chosen };
            let json = serde_json::to_string(&event).unwrap();
            println!("> {}", json);
            ws_stream.send(Message::Text(json.into())).await.unwrap();
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
  let _guard = init_log("ws_agent");

  work().await?;

  Ok(())
}
