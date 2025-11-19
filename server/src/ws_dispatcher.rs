use core::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::frame::{CloseFrame, Utf8Bytes};
use tracing::{error, info, warn};
use uuid::Uuid;

pub struct WsDispatcher {
  end_points: Arc<Mutex<HashMap<uuid::Uuid, EndPoint>>>,
}

struct EndPoint {
  pub req_client_all_senders: Arc<RwLock<Vec<mpsc::Sender<String>>>>,
  pub resp_sender: mpsc::Sender<String>,
}

impl EndPoint {
  pub fn new(mut req_bcast_receiver: mpsc::Receiver<String>, resp_sender: mpsc::Sender<String>) -> Self {
    let req_client_all_senders = Arc::new(RwLock::new(Vec::<mpsc::Sender<String>>::new()));

    let req_client_all_senders_clone = req_client_all_senders.clone();
    tokio::spawn(async move {
      while let Some(msg) = req_bcast_receiver.recv().await {
        let mut all_senders = req_client_all_senders_clone.write().await;
        // println!("req_bcast_receiver: {:?}, all_senders: {:?}", msg, all_senders);
        all_senders.retain(|s| s.clone().try_send(msg.clone()).is_ok());
      }
      println!("req_bcast_receiver closed");
      {
        let mut all_senders = req_client_all_senders_clone.write().await;
        all_senders.clear();
        // for s in all_senders.iter_mut() {
        //   drop(s);
        // }
      }
    });

    Self {
      req_client_all_senders,
      resp_sender,
    }
  }
}

impl WsDispatcher {
  pub fn new(addr: String) -> Self {
    let end_points = Arc::new(Mutex::new(HashMap::new()));

    tokio::spawn({
      let end_points: Arc<Mutex<HashMap<Uuid, EndPoint>>> = end_points.clone();

      async move {
        let listener = TcpListener::bind(addr).await.unwrap();

        while let Ok((stream, _)) = listener.accept().await {
          let end_points = end_points.clone();

          let peer = stream.peer_addr().unwrap();
          tokio::spawn(async move {
            WsDispatcher::handle_connection(end_points, peer, stream).await; // TODO: join handle_connection
          });
        }
      }
    });

    Self { end_points }
  }

  async fn handle_connection(
    end_points: Arc<Mutex<HashMap<uuid::Uuid, EndPoint>>>, peer: SocketAddr, stream: TcpStream,
  ) {
    let mut captured_request: Option<Request> = None;
    let callback = |req: &Request, response: Response| {
      // Capture the request for later use
      captured_request = Some(req.clone());

      Ok(response)
    };

    let mut ws_stream = accept_hdr_async(stream, callback).await.unwrap();

    let path = captured_request.unwrap().uri().path().to_string();
    let id = path.strip_prefix("/").unwrap();
    let id = Uuid::parse_str(id).unwrap();

    let (req_client_tx, mut req_client_rx) = mpsc::channel::<String>(1024);

    {
      let mut end_points_guard = end_points.lock().await;
      let end_point = end_points_guard.get_mut(&id).unwrap();
      end_point.req_client_all_senders.write().await.push(req_client_tx);
      // println!("end_point.req_client_all_senders: {:?}", end_point.req_client_all_senders.read().await);
    }
    let resp_tx = {
      let end_points_guard = end_points.lock().await;
      end_points_guard.get(&id).unwrap().resp_sender.clone()
    };

    loop {
      tokio::select! {
        msg_result = req_client_rx.recv() => {
          match msg_result {
            Some(msg) => {
              if msg.len() > 1024 * 1024 { // 1MB limit
                error!("Message too large from {}, dropping connection", peer);
                break;
              }

              let message = Message::Text(msg.clone().into());
              if let Err(e) = ws_stream.send(message).await {
                error!("Failed to send message to {}: {}", peer, e);
                break;
              }

              info!("Sent message to {}: {}", peer, msg);
            }
            None => {
              warn!("Client channel closed for {}, terminating connection", peer);
              let _ = ws_stream.close(Some(CloseFrame { code: CloseCode::Normal, reason: Utf8Bytes::from("req_client_rx closed") })).await;
              break;
            }
          }
        }

        msg_result = ws_stream.next() => {
          match msg_result {
            Some(msg) => {
              match msg {
                Ok(Message::Text(text)) => {
                  if text.len() > 1024 * 1024 { // 1MB limit
                    error!("Message too large from {}, dropping connection", peer);
                    break;
                  }
                  info!("Received message from {}: {}", peer, text);

                  resp_tx.send(text.to_string()).await.unwrap();
                }
                Ok(Message::Close(close_frame)) => {
                  info!("Client {} requested connection close: {:?}", peer, close_frame);
                  break;
                }
                _ => {
                  error!("Received unknown message from {}, dropping connection", peer);
                  break;
                }
              }
            }
            None => {
              warn!("WebSocket stream ended for {}, terminating connection", peer);
              break;
            }
          }
        }
      }
    }

    // TODO: remove all_req_client_txs
  }

  pub async fn add_end_point(
    &mut self, end_point: uuid::Uuid, req_bcast_receiver: mpsc::Receiver<String>, resp_sender: mpsc::Sender<String>,
  ) {
    self
      .end_points
      .lock()
      .await
      .insert(end_point, EndPoint::new(req_bcast_receiver, resp_sender));
  }

  pub async fn remove_end_point(&mut self, end_point: uuid::Uuid) {
    self.end_points.lock().await.remove(&end_point);
  }
}
