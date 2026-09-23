use crate::domain::normalize::normalize_message;
use futures::StreamExt;
use serde_json::Value;
use tokio::sync::broadcast;
use tokio_tungstenite::connect_async;

/// Resolve the authenticated user's Twitch numeric ID via the Helix API.
async fn resolve_twitch_user_id(
    client: &reqwest::Client,
    token: &str,
    client_id: &str,
) -> Result<String, String> {
    let helix_base = std::env::var("TWITCH_HELIX_BASE_URL")
        .unwrap_or_else(|_| "https://api.twitch.tv".to_string());
    let users_url = format!("{}/helix/users", helix_base);

    let res = client
        .get(&users_url)
        .bearer_auth(token)
        .header("Client-Id", client_id)
        .send()
        .await
        .map_err(|e| format!("Helix /users request failed: {}", e))?;

    let status = res.status();
    let body = res
        .text()
        .await
        .unwrap_or_else(|_| "<unreadable>".to_string());

    if !status.is_success() {
        return Err(format!("Helix /users returned {}: {}", status, body));
    }

    let json: Value = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse /users response: {}", e))?;

    json.get("data")
        .and_then(|d| d.as_array())
        .and_then(|arr| arr.first())
        .and_then(|user| user.get("id"))
        .and_then(|id| id.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| format!("No user found in /users response: {}", body))
}

pub async fn spawn_twitch_client(widget_id: String, token: String, tx: broadcast::Sender<String>) {
    let ws_url = std::env::var("TWITCH_WS_URL")
        .unwrap_or_else(|_| "wss://eventsub.wss.twitch.tv/ws".to_string());
    let http_url = std::env::var("TWITCH_HTTP_URL")
        .unwrap_or_else(|_| "https://api.twitch.tv/helix/eventsub/subscriptions".to_string());
    let client_id = match std::env::var("TWITCH_CLIENT_ID") {
        Ok(id) => id,
        Err(_) => {
            eprintln!(
                "[Twitch] TWITCH_CLIENT_ID env var is not set — Twitch integration will not work"
            );
            return;
        }
    };

    // Resolve the authenticated user's Twitch ID once before entering the loop
    let http_client = reqwest::Client::new();
    let twitch_user_id = match resolve_twitch_user_id(&http_client, &token, &client_id).await {
        Ok(id) => {
            println!("[Twitch] Resolved user ID: {}", id);
            id
        }
        Err(e) => {
            eprintln!("[Twitch] Failed to resolve user ID: {}", e);
            return;
        }
    };

    loop {
        if tx.receiver_count() == 0 {
            break;
        }

        match connect_async(&ws_url).await {
            Ok((mut ws_stream, _)) => {
                println!("[Twitch] Connected to EventSub WebSocket");
                while let Some(msg) = ws_stream.next().await {
                    if tx.receiver_count() == 0 {
                        // Widget disconnected, teardown connection
                        break;
                    }

                    if let Ok(tokio_tungstenite::tungstenite::Message::Text(text)) = msg {
                        if let Ok(payload) = serde_json::from_str::<Value>(&text) {
                            let msg_type = payload
                                .get("metadata")
                                .and_then(|m| m.get("message_type"))
                                .and_then(|t| t.as_str())
                                .unwrap_or("");

                            if msg_type == "session_welcome" {
                                if let Some(sid) = payload
                                    .get("payload")
                                    .and_then(|p| p.get("session"))
                                    .and_then(|s| s.get("id"))
                                    .and_then(|i| i.as_str())
                                {
                                    let body = serde_json::json!({
                                        "type": "channel.chat.message",
                                        "version": "1",
                                        "condition": {
                                            "broadcaster_user_id": twitch_user_id,
                                            "user_id": twitch_user_id
                                        },
                                        "transport": {
                                            "method": "websocket",
                                            "session_id": sid
                                        }
                                    });
                                    let res = http_client
                                        .post(&http_url)
                                        .bearer_auth(&token)
                                        .header("Client-Id", &client_id)
                                        .json(&body)
                                        .send()
                                        .await;

                                    match res {
                                        Ok(response) => {
                                            let status = response.status();
                                            let resp_body = response
                                                .text()
                                                .await
                                                .unwrap_or_else(|_| "<unreadable>".to_string());
                                            if status.is_success() {
                                                println!(
                                                    "[Twitch] EventSub subscription created for user {}",
                                                    twitch_user_id
                                                );
                                            } else {
                                                eprintln!(
                                                    "[Twitch] EventSub subscription failed ({}): {}",
                                                    status, resp_body
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!(
                                                "[Twitch] Failed to send EventSub subscription request: {}",
                                                e
                                            );
                                        }
                                    }
                                }
                            } else if msg_type == "notification" {
                                if let Some(payload_inner) = payload.get("payload") {
                                    if let Some(chat_msg) =
                                        normalize_message("twitch", &widget_id, payload_inner)
                                    {
                                        if let Ok(json) = serde_json::to_string(&chat_msg) {
                                            if let Err(e) = tx.send(json) {
                                                eprintln!(
                                                    "[Twitch] Failed to broadcast message: {}",
                                                    e
                                                );
                                                break; // Receiver channel broken or empty
                                            }
                                        }
                                    }
                                }
                            } else if msg_type == "session_keepalive" {
                                // Expected heartbeat, no action needed
                            } else if msg_type == "session_reconnect" {
                                // Twitch is asking us to reconnect
                                println!("[Twitch] Received reconnect request, will reconnect");
                                break;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[Twitch] Failed to connect to EventSub: {}", e);
            }
        }

        if tx.receiver_count() == 0 {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
