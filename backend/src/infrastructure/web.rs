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

use crate::application::pin::{self, PinCommand, PinError, PinEvent};
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
            "/public/widgets/:id/viewer-counts",
            get(viewer_count_handler),
        )
        .route(
            "/internal/widgets/:id/pin",
            post(pin_handler).layer(DefaultBodyLimit::max(16 * 1024)),
        )
        .with_state(state)
}

async fn viewer_count_handler(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<crate::application::viewer_count::ViewerCountResponse>, StatusCode> {
    let addressed = uuid::Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let pool = state.pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let repository = super::viewer_count::PostgresViewerCountRepository {
        pool,
        master_key: &state.master_key,
    };
    let platforms = super::viewer_count::HttpPlatformViewerCounts::from_env()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    state
        .viewer_count_service
        .read(&repository, &platforms, addressed)
        .await
        .map(Json)
        .map_err(|error| match error {
            crate::application::viewer_count::ViewerCountError::NotFound => StatusCode::NOT_FOUND,
            crate::application::viewer_count::ViewerCountError::Unavailable => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            crate::application::viewer_count::ViewerCountError::Timeout => {
                StatusCode::GATEWAY_TIMEOUT
            }
        })
}

#[cfg(test)]
mod viewer_count_route_tests {
    use super::create_router;
    use crate::application::state::AppState;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn viewer_count_route_validates_uuid_before_backend_lookup() {
        let app = create_router(Arc::new(AppState::default()));
        let invalid = app
            .clone()
            .oneshot(
                Request::get("/public/widgets/not-a-uuid/viewer-counts")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);

        let valid_id = uuid::Uuid::new_v4();
        let valid = app
            .oneshot(
                Request::get(format!("/public/widgets/{valid_id}/viewer-counts"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(valid.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
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
    if !pin::validate_command(&command, &id) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let pool = state.pool.as_ref().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let canonical = crate::infrastructure::db::canonical_widget_id(pool, &widget_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    if canonical != widget_id {
        return Err(StatusCode::NOT_FOUND);
    }
    let storage = crate::infrastructure::db::PostgresPins { pool };
    let event = pin::transition(&state, &storage, &id, &widget_id, command)
        .await
        .map_err(|error| match error {
            PinError::InvalidCommand => StatusCode::BAD_REQUEST,
            PinError::MissingWidget => StatusCode::NOT_FOUND,
            PinError::Storage | PinError::Serialization => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
    Ok(Json(event))
}

async fn handle_socket(
    mut socket: WebSocket,
    state: Arc<AppState>,
    widget_id: Option<String>,
    mock: bool,
) {
    let (widget_id, addressed_id, mut needs_recheck) =
        if let (Some(id), Some(pool)) = (&widget_id, &state.pool) {
            let addressed = match uuid::Uuid::parse_str(id) {
                Ok(value) => value,
                Err(_) => {
                    let _ = socket.send(Message::Close(None)).await;
                    return;
                }
            };
            match crate::infrastructure::db::widget_identity(pool, &addressed).await {
                Ok(Some(identity)) => (
                    Some(identity.canonical_id.to_string()),
                    Some(addressed),
                    !identity.mapped,
                ),
                _ => {
                    let _ = socket.send(Message::Close(None)).await;
                    return;
                }
            }
        } else {
            (widget_id, None, false)
        };
    let connection_type = if let Some(id) = &widget_id {
        format!("Widget({})", id)
    } else {
        "Global".to_string()
    };
    println!(
        "[WebSocket Backend] New connection established: {}",
        connection_type
    );

    let mut widget_connection = widget_id.as_ref().map(|id| state.acquire_widget(id, mock));
    let mut tx = widget_connection
        .as_ref()
        .map_or_else(|| state.tx.clone(), |connection| connection.tx.clone());
    let mut rx = widget_connection.as_mut().map_or_else(
        || state.tx.subscribe(),
        |connection| connection.take_receiver(),
    );

    let snapshot = if let (Some(id), Some(pool)) = (&widget_id, &state.pool) {
        if let Ok(uuid) = uuid::Uuid::parse_str(id) {
            let storage = crate::infrastructure::db::PostgresPins { pool };
            match pin::snapshot(&state, &storage, id, &uuid).await {
                Ok((_, json)) => Some(json),
                Err(_) => {
                    eprintln!("[WebSocket Backend] Failed to load pin state for Widget({id})");
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
    if let Some(json) = snapshot {
        if sender.send(Message::Text(json)).await.is_err() {
            return;
        }
    }

    // Only sockets opened before the account's first mapping poll. Once mapping exists,
    // identity is permanent and this query stops for the rest of the connection.
    let period = tokio::time::Duration::from_secs(5);
    let mut recheck = tokio::time::interval_at(tokio::time::Instant::now() + period, period);
    recheck.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            event = rx.recv() => match event {
                Ok(message) => {
                    if sender.send(Message::Text(message)).await.is_err() { break; }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("[WebSocket Backend] Receiver lagged by {n} messages for {connection_type}");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            },
            incoming = receiver.next() => match incoming {
                Some(Ok(Message::Text(text))) => {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        if json["type"] == "ping" {
                            let pong = serde_json::json!({ "type": "pong" });
                            if let Err(error) = tx.send(pong.to_string()) {
                                eprintln!("[WebSocket Backend] Failed to broadcast pong for {connection_type}: {error}");
                            }
                        }
                    }
                }
                Some(Ok(Message::Close(_))) | None => break,
                Some(Err(error)) => {
                    eprintln!("[WebSocket Backend] Error receiving message from {connection_type}: {error}");
                    break;
                }
                _ => {},
            },
            _ = recheck.tick(), if needs_recheck => {
                let (Some(addressed), Some(pool)) = (addressed_id, state.pool.as_ref()) else { break; };
                match crate::infrastructure::db::widget_identity(pool, &addressed).await {
                    Ok(Some(identity)) if identity.mapped => {
                        needs_recheck = false;
                        let canonical = identity.canonical_id.to_string();
                        let mut next_connection = if widget_id.as_deref() != Some(canonical.as_str()) {
                            Some(state.acquire_widget(&canonical, mock))
                        } else { None };
                        let next_rx = next_connection.as_mut().map(|connection| connection.take_receiver());
                        let storage = crate::infrastructure::db::PostgresPins { pool };
                        let snapshot = pin::snapshot(&state, &storage, &canonical, &identity.canonical_id).await;
                        let Ok((_, json)) = snapshot else { break; };
                        if sender.send(Message::Text(json)).await.is_err() { break; }
                        if let (Some(next_connection), Some(next_rx)) = (next_connection, next_rx) {
                            tx = next_connection.tx.clone();
                            rx = next_rx;
                            widget_connection = Some(next_connection);
                        }
                    }
                    Ok(Some(_)) | Err(_) => {},
                    Ok(None) => break,
                }
            }
        }
    }

    println!("[WebSocket Backend] Connection closed: {}", connection_type);

    drop(widget_connection);
}
