use crate::domain::normalize::normalize_message;
use futures::StreamExt;
use serde_json::Value;
use tokio::sync::broadcast;
use tokio_tungstenite::connect_async;

pub async fn spawn_twitch_client(widget_id: String, token: String, tx: broadcast::Sender<String>) {
    let ws_url = std::env::var("TWITCH_WS_URL")
        .unwrap_or_else(|_| "wss://eventsub.wss.twitch.tv/ws".to_string());
    let http_url = std::env::var("TWITCH_HTTP_URL")
        .unwrap_or_else(|_| "https://api.twitch.tv/helix/eventsub/subscriptions".to_string());
    let client_id =
        std::env::var("TWITCH_CLIENT_ID").unwrap_or_else(|_| "dummy_client_id".to_string());

    loop {
        if tx.receiver_count() == 0 {
            break;
        }

        match connect_async(&ws_url).await {
            Ok((mut ws_stream, _)) => {
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
                                    let client = reqwest::Client::new();
                                    let body = serde_json::json!({
                                        "type": "channel.chat.message",
                                        "version": "1",
                                        "condition": {
                                            "broadcaster_user_id": "1234",
                                            "user_id": "1234"
                                        },
                                        "transport": {
                                            "method": "websocket",
                                            "session_id": sid
                                        }
                                    });
                                    let res = client
                                        .post(&http_url)
                                        .bearer_auth(&token)
                                        .header("Client-Id", &client_id)
                                        .json(&body)
                                        .send()
                                        .await;

                                    if let Err(e) = res {
                                        eprintln!("Failed to subscribe to Twitch EventSub: {}", e);
                                    }
                                }
                            } else if msg_type == "notification" {
                                if let Some(payload_inner) = payload.get("payload") {
                                    if let Some(chat_msg) =
                                        normalize_message("twitch", &widget_id, payload_inner)
                                    {
                                        if let Ok(json) = serde_json::to_string(&chat_msg) {
                                            if let Err(e) = tx.send(json) {
                                                eprintln!("Failed to broadcast message: {}", e);
                                                break; // Receiver channel broken or empty
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to Twitch EventSub: {}", e);
            }
        }

        if tx.receiver_count() == 0 {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
