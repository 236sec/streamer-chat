use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router,
};

pub fn app() -> Router {
    Router::new().route("/ws", get(ws_handler))
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        if let Ok(Message::Text(text)) = msg {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                if json["type"] == "ping" {
                    let pong = serde_json::json!({ "type": "pong" });
                    let _ = socket.send(Message::Text(pong.to_string())).await;
                }
            }
        }
    }
}
