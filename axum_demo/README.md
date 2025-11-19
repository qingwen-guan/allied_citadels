# Axum HTTP + WebSocket Demo

A simple demo showing how to run both HTTP and WebSocket on the same port using Axum.

## Features

- ✅ HTTP GET endpoint: `/api/hello`
- ✅ HTTP POST endpoint: `/api/echo`
- ✅ WebSocket endpoint: `/ws`
- ✅ Interactive HTML page for testing
- ✅ All running on the same port (3000)

## Running

```bash
cd axum_demo
cargo run
```

Then open http://127.0.0.1:3000 in your browser.

## Testing

### HTTP API

```bash
# GET request
curl http://127.0.0.1:3000/api/hello

# POST request
curl -X POST http://127.0.0.1:3000/api/echo \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello World"}'
```

### WebSocket

You can test the WebSocket using:
- The interactive HTML page at http://127.0.0.1:3000
- Or use a WebSocket client like `websocat`:

```bash
websocat ws://127.0.0.1:3000/ws
```

## Key Points

1. **Same Port**: Both HTTP and WebSocket use the same TCP port (3000)
2. **Route Separation**: HTTP routes use `/api/*` and WebSocket uses `/ws`
3. **Upgrade Pattern**: Axum automatically upgrades HTTP connections to WebSocket when requested
4. **Shared State**: The example shows how to share state between connections using `Arc<Mutex<>>`

