use crate::domain::message::ChatMessage;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

pub struct AppState {
    pub tx: broadcast::Sender<String>,
    pub widget_channels: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
    pub pool: Option<PgPool>,
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tx,
            widget_channels: Arc::new(RwLock::new(HashMap::new())),
            pool: None,
        }
    }
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tx,
            widget_channels: Arc::new(RwLock::new(HashMap::new())),
            pool: Some(pool),
        }
    }

    pub async fn get_or_create_channel(
        &self,
        widget_id: &str,
        mock: bool,
    ) -> broadcast::Sender<String> {
        let channels = self.widget_channels.read().await;
        if let Some(sender) = channels.get(widget_id) {
            return sender.clone();
        }
        drop(channels);

        let mut channels = self.widget_channels.write().await;
        channels
            .entry(widget_id.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);

                if mock {
                    let tx_clone = tx.clone();
                    let w_id = widget_id.to_string();
                    tokio::spawn(async move {
                        let mut interval =
                            tokio::time::interval(tokio::time::Duration::from_secs(2));
                        loop {
                            interval.tick().await;
                            if tx_clone.receiver_count() == 0 {
                                break;
                            }
                            let msg = ChatMessage::mock(&w_id);
                            if let Ok(json) = serde_json::to_string(&msg) {
                                if tx_clone.send(json).is_err() {
                                    break;
                                }
                            }
                        }
                    });
                } else if let Some(pool) = self.pool.clone() {
                    let w_id = widget_id.to_string();
                    tokio::spawn(async move {
                        if let Ok(w_uuid) = uuid::Uuid::parse_str(&w_id) {
                            if let Ok(tokens) =
                                crate::infrastructure::db::fetch_tokens(&pool, &w_uuid).await
                            {
                                let key =
                                    std::env::var("MASTER_DECRYPTION_KEY").unwrap_or_else(|_| {
                                        "0123456789abcdef0123456789abcdef".to_string()
                                    });

                                if let Some(twitch_enc) = tokens.twitch {
                                    let _twitch_token =
                                        crate::infrastructure::db::decrypt_token(&twitch_enc, &key);
                                    // Normally we would connect to twitch WS here and loop.
                                    // For this test, we simulate normalized messages.
                                    let _ = crate::domain::normalize::normalize_message(
                                        "twitch",
                                        &w_id,
                                        &serde_json::json!({"message": "test"}),
                                    );
                                }
                            }
                        }
                    });
                }

                tx
            })
            .clone()
    }
}
