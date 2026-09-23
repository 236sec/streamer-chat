use axum::{
    extract::DefaultBodyLimit,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Json, Path, Query, State,
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use std::sync::Arc;

use crate::application::pin::{validate_command, PinCommand, PinEvent};
use crate::application::state::AppState;

#[derive(Deserialize)]
pub struct WidgetQuery {
    pub mock: Option<bool>,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ws", get(ws_handler_global))
        .route("/ws/widget/:id", get(ws_handler_widget))
        .route(
            "/internal/widgets/:id/pin",
            post(pin_handler).layer(DefaultBodyLimit::max(16 * 1024)),
        )
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

async fn pin_handler(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(command): Json<PinCommand>,
) -> Result<Json<PinEvent>, StatusCode> {
    let supplied = headers.get("x-pin-secret").and_then(|v| v.to_str().ok());
    if state.pin_command_secret.is_empty() || supplied != Some(state.pin_command_secret.as_str()) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let widget_id = uuid::Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    if !validate_command(&command, &id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pool = state.pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let lock = state.pin_lock(&id).await;
    let _guard = lock.lock().await;
    let message = match command {
        PinCommand::Pin { message } => Some(*message),
        PinCommand::Unpin => None,
    };
    let revision = crate::infrastructure::db::save_pin(pool, &widget_id, message.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let event = PinEvent {
        r#type: if message.is_some() {
            "pin_message"
        } else {
            "unpin_message"
        }
        .to_string(),
        revision,
        message,
    };
    if let Some(channel) = state.widget_channels.read().await.get(&id) {
        if let Ok(json) = serde_json::to_string(&event) {
            let _ = channel.send(json);
        }
    }
    Ok(Json(event))
}

async fn handle_socket(
    mut socket: WebSocket,
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

    let snapshot = if let (Some(id), Some(pool)) = (&widget_id, &state.pool) {
        if let Ok(uuid) = uuid::Uuid::parse_str(id) {
            let lock = state.pin_lock(id).await;
            let _guard = lock.lock().await;
            match crate::infrastructure::db::load_pin(pool, &uuid).await {
                Ok(Some(pin)) => Some(pin),
                Ok(None) => {
                    let _ = socket.send(Message::Close(None)).await;
                    return;
                }
                Err(error) => {
                    eprintln!(
                        "[WebSocket Backend] Failed to load pin state for Widget({id}): {error}"
                    );
                    let _ = socket.send(Message::Close(None)).await;
                    return;
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    let (mut sender, mut receiver) = socket.split();
    if let Some((revision, message)) = snapshot {
        let event = PinEvent {
            r#type: "pin_state".to_string(),
            revision,
            message,
        };
        if let Ok(json) = serde_json::to_string(&event) {
            if sender.send(Message::Text(json)).await.is_err() {
                return;
            }
        }
    }

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
                if let Some((_flag, cancel_token)) = workers.remove(&id) {
                    cancel_token.cancel();
                }
            }
        }
    }
}
