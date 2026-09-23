use crate::domain::message::ChatMessage;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_util::sync::CancellationToken;

pub type WorkerEntry = (Arc<std::sync::atomic::AtomicBool>, CancellationToken);

pub struct AppState {
    pub tx: broadcast::Sender<String>,
    pub widget_channels: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
    pub widget_workers: Arc<RwLock<HashMap<String, WorkerEntry>>>,
    pub pool: Option<PgPool>,
    pub master_key: String,
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(100);
        let master_key = std::env::var("MASTER_DECRYPTION_KEY")
            .unwrap_or_else(|_| "0123456789abcdef0123456789abcdef".to_string());
        Self {
            tx,
            widget_channels: Arc::new(RwLock::new(HashMap::new())),
            widget_workers: Arc::new(RwLock::new(HashMap::new())),
            pool: None,
            master_key,
        }
    }
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        let (tx, _) = broadcast::channel(100);
        let master_key = std::env::var("MASTER_DECRYPTION_KEY")
            .unwrap_or_else(|_| "0123456789abcdef0123456789abcdef".to_string());
        Self {
            tx,
            widget_channels: Arc::new(RwLock::new(HashMap::new())),
            widget_workers: Arc::new(RwLock::new(HashMap::new())),
            pool: Some(pool),
            master_key,
        }
    }

    pub async fn get_or_create_channel(
        &self,
        widget_id: &str,
        mock: bool,
    ) -> broadcast::Sender<String> {
        let mut channels = self.widget_channels.write().await;
        let tx = channels
            .entry(widget_id.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                tx
            })
            .clone();

        let mut workers = self.widget_workers.write().await;
        let (worker_running, cancel_token) = workers
            .entry(widget_id.to_string())
            .or_insert_with(|| {
                (
                    Arc::new(std::sync::atomic::AtomicBool::new(false)),
                    CancellationToken::new(),
                )
            })
            .clone();

        if !worker_running.load(std::sync::atomic::Ordering::SeqCst) {
            worker_running.store(true, std::sync::atomic::Ordering::SeqCst);
            let flag = worker_running.clone();

            if mock {
                let tx_clone = tx.clone();
                let w_id = widget_id.to_string();
                let ct_mock = cancel_token.clone();
                tokio::spawn(async move {
                    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
                    loop {
                        interval.tick().await;
                        if tx_clone.receiver_count() == 0 || ct_mock.is_cancelled() {
                            break;
                        }
                        let msg = ChatMessage::mock(&w_id);
                        if let Ok(json) = serde_json::to_string(&msg) {
                            if tx_clone.send(json).is_err() {
                                break;
                            }
                        }
                    }
                    flag.store(false, std::sync::atomic::Ordering::SeqCst);
                });
            } else if let Some(pool) = self.pool.clone() {
                let w_id = widget_id.to_string();
                let tx_clone = tx.clone();
                let master_key = self.master_key.clone();
                tokio::spawn(async move {
                    if let Ok(w_uuid) = uuid::Uuid::parse_str(&w_id) {
                        if let Ok(tokens) =
                            crate::infrastructure::db::fetch_tokens(&pool, &w_uuid).await
                        {
                            let mut handles = vec![];

                            if let Some(twitch_enc) = tokens.twitch {
                                let twitch_token = crate::infrastructure::db::decrypt_token(
                                    &twitch_enc,
                                    &master_key,
                                );

                                let w_id_clone = w_id.clone();
                                let tx_platform = tx_clone.clone();
                                let ct_platform = cancel_token.clone();
                                handles.push(tokio::spawn(async move {
                                    crate::infrastructure::twitch::spawn_twitch_client(
                                        w_id_clone,
                                        twitch_token,
                                        tx_platform,
                                        ct_platform,
                                    )
                                    .await;
                                }));
                            }

                            if let Some(youtube_enc) = tokens.youtube {
                                let youtube_token = crate::infrastructure::db::decrypt_token(
                                    &youtube_enc,
                                    &master_key,
                                );
                                let youtube_refresh = tokens.youtube_refresh.map(|enc| {
                                    crate::infrastructure::db::decrypt_token(&enc, &master_key)
                                });

                                let w_id_clone = w_id.clone();
                                let tx_platform = tx_clone.clone();
                                let ct_platform = cancel_token.clone();
                                handles.push(tokio::spawn(async move {
                                    crate::infrastructure::youtube::spawn_youtube_client(
                                        w_id_clone,
                                        youtube_token,
                                        youtube_refresh,
                                        tx_platform,
                                        ct_platform,
                                    )
                                    .await;
                                }));
                            }

                            if let Some(kick_enc) = tokens.kick {
                                let kick_username = crate::infrastructure::db::decrypt_token(
                                    &kick_enc,
                                    &master_key,
                                );

                                let w_id_clone = w_id.clone();
                                let tx_platform = tx_clone.clone();
                                let ct_platform = cancel_token.clone();
                                handles.push(tokio::spawn(async move {
                                    crate::infrastructure::kick::spawn_kick_client(
                                        w_id_clone,
                                        kick_username,
                                        tx_platform,
                                        ct_platform,
                                    )
                                    .await;
                                }));
                            }

                            for handle in handles {
                                let _ = handle.await;
                            }
                        }
                    }
                    flag.store(false, std::sync::atomic::Ordering::SeqCst);
                });
            } else {
                flag.store(false, std::sync::atomic::Ordering::SeqCst);
            }
        }

        tx
    }
}
