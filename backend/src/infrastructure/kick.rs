use crate::domain::normalize::normalize_message;
use futures::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::broadcast;
use tokio_tungstenite::connect_async;

async fn resolve_kick_chatroom_id(client: &reqwest::Client, username: &str) -> Result<u64, String> {
    let api_base =
        std::env::var("KICK_API_BASE_URL").unwrap_or_else(|_| "https://kick.com".to_string());
    let url = format!("{}/api/v1/channels/{}", api_base, username);

    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Kick /channels request failed: {}", e))?;

    let status = res.status();
    let body = res
        .text()
        .await
        .unwrap_or_else(|_| "<unreadable>".to_string());

    if !status.is_success() {
        return Err(format!("Kick /channels returned {}: {}", status, body));
    }

    let json: Value =
        serde_json::from_str(&body).map_err(|e| format!("Failed to parse Kick response: {}", e))?;

    let chatroom_id = json
        .get("chatroom")
        .and_then(|c| c.get("id"))
        .and_then(|id| id.as_u64())
        .ok_or_else(|| "Could not find chatroom.id in response".to_string())?;

    Ok(chatroom_id)
}

use crate::infrastructure::util::sleep_or_cancel;
use tokio_util::sync::CancellationToken;

pub async fn spawn_kick_client(
    widget_id: String,
    username: String,
    tx: broadcast::Sender<String>,
    cancel: CancellationToken,
) {
    let ws_url = std::env::var("KICK_WS_URL")
        .unwrap_or_else(|_| "wss://ws-us2.pusher.com/app/32cbd69e4b950bf97679?protocol=7&client=js&version=7.4.0&flash=false".to_string());

    let http_client = reqwest::Client::new();
    let mut chatroom_id: Option<u64> = None;

    loop {
        if tx.receiver_count() == 0 || cancel.is_cancelled() {
            break;
        }

        let cid = match chatroom_id {
            Some(id) => id,
            None => match resolve_kick_chatroom_id(&http_client, &username).await {
                Ok(id) => {
                    println!("[Kick] Resolved chatroom ID: {}", id);
                    chatroom_id = Some(id);
                    id
                }
                Err(e) => {
                    eprintln!("[Kick] Failed to resolve chatroom ID: {}", e);
                    if !sleep_or_cancel(tokio::time::Duration::from_secs(5), &tx, &cancel).await {
                        break;
                    }
                    continue;
                }
            },
        };

        match connect_async(&ws_url).await {
            Ok((mut ws_stream, _)) => {
                println!("[Kick] Connected to Pusher WebSocket");

                // Subscribe to the chatroom channel
                let subscribe_msg = serde_json::json!({
                    "event": "pusher:subscribe",
                    "data": {
                        "channel": format!("chatrooms.{}.v2", cid)
                    }
                });

                if let Err(e) = ws_stream
                    .send(tokio_tungstenite::tungstenite::Message::Text(
                        subscribe_msg.to_string(),
                    ))
                    .await
                {
                    eprintln!("[Kick] Failed to send subscribe message: {}", e);
                    continue;
                }

                while let Some(msg) = ws_stream.next().await {
                    if tx.receiver_count() == 0 || cancel.is_cancelled() {
                        break;
                    }

                    match msg {
                        Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                            if let Ok(payload) = serde_json::from_str::<Value>(&text) {
                                let event =
                                    payload.get("event").and_then(|e| e.as_str()).unwrap_or("");

                                if event == "pusher:ping" {
                                    let pong = serde_json::json!({ "event": "pusher:pong" });
                                    let _ = ws_stream
                                        .send(tokio_tungstenite::tungstenite::Message::Text(
                                            pong.to_string(),
                                        ))
                                        .await;
                                } else if event == "pusher_internal:subscription_succeeded" {
                                    println!("[Kick] Successfully subscribed to chatroom {}", cid);
                                } else if event == "pusher:error" {
                                    eprintln!("[Kick] Pusher error: {}", text);
                                } else if event == "App\\Events\\ChatMessageEvent" {
                                    if let Some(chat_msg) =
                                        normalize_message("kick", &widget_id, &payload)
                                    {
                                        if let Ok(json) = serde_json::to_string(&chat_msg) {
                                            if let Err(e) = tx.send(json) {
                                                eprintln!(
                                                    "[Kick] Failed to broadcast message: {}",
                                                    e
                                                );
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Ok(tokio_tungstenite::tungstenite::Message::Close(frame)) => {
                            eprintln!("[Kick] Pusher closed connection: {:?}", frame);
                            break;
                        }
                        Err(e) => {
                            eprintln!("[Kick] WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("[Kick] Failed to connect to Pusher WebSocket: {}", e);
            }
        }

        if !sleep_or_cancel(tokio::time::Duration::from_secs(5), &tx, &cancel).await {
            break;
        }
    }
}
