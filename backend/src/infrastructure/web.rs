use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use std::sync::Arc;

use crate::application::state::AppState;

#[derive(Deserialize)]
pub struct WidgetQuery {
    pub mock: Option<bool>,
}

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
    ws.on_upgrade(move |socket| handle_socket(socket, state, None, false))
}

async fn ws_handler_widget(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    Query(query): Query<WidgetQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, Some(id), query.mock.unwrap_or(false)))
}

async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    widget_id: Option<String>,
    mock: bool,
) {
    let connection_type = if let Some(id) = &widget_id {
        format!("Widget({})", id)
    } else {
        "Global".to_string()
    };
    println!(
        "[WebSocket Backend] New connection established: {}",
        connection_type
    );

    let tx = match &widget_id {
        Some(id) => state.get_or_create_channel(id, mock).await,
        None => state.tx.clone(),
    };
    let mut rx = tx.subscribe();

    let (mut sender, mut receiver) = socket.split();

    let conn_type_for_send = connection_type.clone();
    let mut send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    if let Err(e) = sender.send(Message::Text(msg)).await {
                        eprintln!(
                            "[WebSocket Backend] Failed to send message to {}: {}",
                            conn_type_for_send, e
                        );
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!(
                        "[WebSocket Backend] Receiver lagged by {} messages for {}",
                        n, conn_type_for_send
                    );
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });

    let conn_type_for_recv = connection_type.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(msg) => {
                    if let Message::Text(text) = msg {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                            if json["type"] == "ping" {
                                let pong = serde_json::json!({ "type": "pong" });
                                if let Err(e) = tx.send(pong.to_string()) {
                                    eprintln!(
                                        "[WebSocket Backend] Failed to broadcast pong for {}: {}",
                                        conn_type_for_recv, e
                                    );
                                }
                            }
                        }
                    } else if let Message::Close(_) = msg {
                        println!(
                            "[WebSocket Backend] Received close frame from {}",
                            conn_type_for_recv
                        );
                        break;
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[WebSocket Backend] Error receiving message from {}: {}",
                        conn_type_for_recv, e
                    );
                    break;
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => {
            println!("[WebSocket Backend] Send task ended for {}", connection_type);
            recv_task.abort();
            let _ = recv_task.await;
        },
        _ = (&mut recv_task) => {
            println!("[WebSocket Backend] Receive task ended for {}", connection_type);
            send_task.abort();
            let _ = send_task.await;
        },
    }

    println!("[WebSocket Backend] Connection closed: {}", connection_type);

    if let Some(id) = widget_id {
        let mut channels = state.widget_channels.write().await;
        if let Some(channel) = channels.get(&id) {
            if channel.receiver_count() == 0 {
                println!("[WebSocket Backend] Cleaning up channel for Widget({})", id);
                channels.remove(&id);
                let mut workers = state.widget_workers.write().await;
                workers.remove(&id);
            }
        }
    }
}
