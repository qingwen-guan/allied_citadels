use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::{Html, Json};
use axum::routing::{get, post};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

#[derive(Serialize, Deserialize)]
struct ApiResponse {
  message: String,
  timestamp: u64,
}

#[derive(Serialize, Deserialize)]
struct EchoRequest {
  text: String,
}

// Shared state for WebSocket connections
type ConnectionMap = Arc<Mutex<HashMap<String, usize>>>;

// HTTP GET endpoint
async fn hello() -> Html<&'static str> {
  Html(
    r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Axum HTTP + WebSocket Demo</title>
        <style>
            body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
            .section { margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }
            button { padding: 10px 20px; margin: 5px; cursor: pointer; }
            #messages { height: 300px; overflow-y: auto; border: 1px solid #ccc; padding: 10px; margin: 10px 0; }
            .message { margin: 5px 0; padding: 5px; background: #f5f5f5; }
        </style>
    </head>
    <body>
        <h1>Axum HTTP + WebSocket Demo</h1>
        
        <div class="section">
            <h2>HTTP API Test</h2>
            <button onclick="testGet()">Test GET /api/hello</button>
            <button onclick="testPost()">Test POST /api/echo</button>
            <div id="http-result"></div>
        </div>
        
        <div class="section">
            <h2>WebSocket Test</h2>
            <button onclick="connectWs()">Connect WebSocket</button>
            <button onclick="disconnectWs()">Disconnect</button>
            <button onclick="sendMessage()">Send Message</button>
            <div id="messages"></div>
        </div>
        
        <script>
            let ws = null;
            
            function addMessage(msg) {
                const div = document.getElementById('messages');
                const p = document.createElement('div');
                p.className = 'message';
                p.textContent = new Date().toLocaleTimeString() + ': ' + msg;
                div.appendChild(p);
                div.scrollTop = div.scrollHeight;
            }
            
            async function testGet() {
                try {
                    const res = await fetch('/api/hello');
                    const data = await res.json();
                    document.getElementById('http-result').innerHTML = 
                        '<pre>' + JSON.stringify(data, null, 2) + '</pre>';
                } catch (e) {
                    document.getElementById('http-result').innerHTML = 'Error: ' + e;
                }
            }
            
            async function testPost() {
                try {
                    const res = await fetch('/api/echo', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ text: 'Hello from HTTP!' })
                    });
                    const data = await res.json();
                    document.getElementById('http-result').innerHTML = 
                        '<pre>' + JSON.stringify(data, null, 2) + '</pre>';
                } catch (e) {
                    document.getElementById('http-result').innerHTML = 'Error: ' + e;
                }
            }
            
            function connectWs() {
                if (ws && ws.readyState === WebSocket.OPEN) {
                    addMessage('Already connected!');
                    return;
                }
                
                const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
                ws = new WebSocket(`${protocol}//${window.location.host}/ws`);
                
                ws.onopen = () => addMessage('WebSocket connected!');
                ws.onmessage = (event) => addMessage('Received: ' + event.data);
                ws.onerror = (error) => addMessage('Error: ' + error);
                ws.onclose = () => addMessage('WebSocket disconnected');
            }
            
            function disconnectWs() {
                if (ws) {
                    ws.close();
                    ws = null;
                }
            }
            
            function sendMessage() {
                if (ws && ws.readyState === WebSocket.OPEN) {
                    const msg = 'Hello from client at ' + new Date().toLocaleTimeString();
                    ws.send(msg);
                    addMessage('Sent: ' + msg);
                } else {
                    addMessage('Not connected! Click "Connect WebSocket" first.');
                }
            }
        </script>
    </body>
    </html>
    "#,
  )
}

// HTTP GET API endpoint
async fn api_hello() -> Json<ApiResponse> {
  Json(ApiResponse {
    message: "Hello from Axum HTTP API!".to_string(),
    timestamp: std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs(),
  })
}

// HTTP POST API endpoint
async fn api_echo(Json(payload): Json<EchoRequest>) -> Json<ApiResponse> {
  Json(ApiResponse {
    message: format!("Echo: {}", payload.text),
    timestamp: std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_secs(),
  })
}

// WebSocket handler
async fn websocket_handler(
  ws: WebSocketUpgrade, axum::extract::State(state): axum::extract::State<ConnectionMap>,
) -> axum::response::Response {
  ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: ConnectionMap) {
  let (mut sender, mut receiver) = socket.split();
  let conn_id = uuid::Uuid::new_v4().to_string();

  // Add connection to state
  {
    let mut conns = state.lock().await;
    let count = conns.len();
    conns.insert(conn_id.clone(), count);
    println!("New WebSocket connection: {} (total: {})", conn_id, count + 1);
  }

  // Send welcome message
  let welcome_msg = format!("Welcome! Your connection ID: {}", conn_id);
  let _ = sender.send(Message::Text(welcome_msg)).await;

  // Handle incoming messages
  while let Some(Ok(msg)) = receiver.next().await {
    match msg {
      Message::Text(text) => {
        println!("Received from {}: {}", conn_id, text);
        let echo = format!("Echo: {} (from server)", text);
        if sender.send(Message::Text(echo)).await.is_err() {
          break;
        }
      },
      Message::Close(_) => {
        println!("WebSocket connection {} closed", conn_id);
        break;
      },
      _ => {},
    }
  }

  // Remove connection from state
  {
    let mut conns = state.lock().await;
    conns.remove(&conn_id);
    println!("WebSocket connection {} removed (total: {})", conn_id, conns.len());
  }
}

#[tokio::main]
async fn main() {
  // Initialize tracing
  tracing_subscriber::fmt::init();

  // Shared state for WebSocket connections
  let connections: ConnectionMap = Arc::new(Mutex::new(HashMap::new()));

  // Build the router with both HTTP and WebSocket routes
  let app = Router::new()
        // HTTP routes
        .route("/", get(hello))
        .route("/api/hello", get(api_hello))
        .route("/api/echo", post(api_echo))
        // WebSocket route
        .route("/ws", get(websocket_handler))
        // Add CORS middleware
        .layer(ServiceBuilder::new().layer(CorsLayer::permissive()))
        // Add shared state
        .with_state(connections);

  // Bind to address
  let addr = "127.0.0.1:3000";
  let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

  println!("🚀 Server running on http://{}", addr);
  println!("📡 WebSocket available at ws://{}/ws", addr);
  println!("🌐 Open http://{} in your browser to test", addr);

  axum::serve(listener, app).await.unwrap();
}
