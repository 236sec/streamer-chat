use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;

use crate::application::state::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ws", get(ws_handler_global))
        .route("/ws/widget/:id", get(ws_handler_widget))
        .with_state(state)
}

async fn ws_handler_global(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, None))
}

async fn ws_handler_widget(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, Some(id)))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>, widget_id: Option<String>) {
    let mut rx = match &widget_id {
        Some(id) => state.get_or_create_channel(id).await.subscribe(),
        None => state.tx.subscribe(),
    };

    let tx = match &widget_id {
        Some(id) => state.get_or_create_channel(id).await,
        None => state.tx.clone(),
    };

    let (mut sender, mut receiver) = socket.split();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Err(_e) = sender.send(Message::Text(msg)).await {
                // Ignore error, it means client disconnected
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if json["type"] == "ping" {
                        let pong = serde_json::json!({ "type": "pong" });
                        if let Err(_e) = tx.send(pong.to_string()) {
                            // ignore error
                        }
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    if let Some(id) = widget_id {
        let mut channels = state.widget_channels.write().await;
        if let Some(channel) = channels.get(&id) {
            if channel.receiver_count() == 0 {
                channels.remove(&id);
            }
        }
    }
}
