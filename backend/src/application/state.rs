use crate::domain::message::ChatMessage;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

pub struct AppState {
    pub tx: broadcast::Sender<String>,
    pub widget_channels: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tx,
            widget_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl AppState {
    pub async fn get_or_create_channel(&self, widget_id: &str) -> broadcast::Sender<String> {
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

                if std::env::var("MOCK_CHAT").unwrap_or_else(|_| "false".to_string()) == "true" {
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
                }

                tx
            })
            .clone()
    }
}
