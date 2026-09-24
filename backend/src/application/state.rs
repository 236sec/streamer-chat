use crate::domain::message::ChatMessage;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio_util::sync::CancellationToken;

struct WorkerRun {
    generation: u64,
    cancel: CancellationToken,
}

struct SessionLifetime {
    connections: usize,
    generation: u64,
    worker: Option<WorkerRun>,
}

struct WidgetSession {
    tx: broadcast::Sender<String>,
    lifetime: StdMutex<SessionLifetime>,
}

pub struct WidgetConnection {
    state: Arc<AppState>,
    widget_id: String,
    session: Arc<WidgetSession>,
    pub tx: broadcast::Sender<String>,
    rx: Option<broadcast::Receiver<String>>,
}

impl WidgetConnection {
    pub fn take_receiver(&mut self) -> broadcast::Receiver<String> {
        self.rx.take().expect("widget receiver already taken")
    }
}

impl Drop for WidgetConnection {
    fn drop(&mut self) {
        let mut sessions = self.state.widget_sessions.lock().unwrap();
        let mut lifetime = self.session.lifetime.lock().unwrap();
        lifetime.connections -= 1;
        if lifetime.connections == 0 {
            if sessions
                .get(&self.widget_id)
                .is_some_and(|current| Arc::ptr_eq(current, &self.session))
            {
                sessions.remove(&self.widget_id);
            }
            if let Some(run) = lifetime.worker.take() {
                run.cancel.cancel();
            }
        }
    }
}

pub struct AppState {
    pub tx: broadcast::Sender<String>,
    widget_sessions: StdMutex<HashMap<String, Arc<WidgetSession>>>,
    pub pool: Option<PgPool>,
    pub pin_locks: Arc<RwLock<HashMap<String, Arc<Mutex<()>>>>>,
    pub pin_command_secret: String,
    pub master_key: String,
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            tx,
            widget_sessions: StdMutex::new(HashMap::new()),
            pool: None,
            pin_locks: Arc::new(RwLock::new(HashMap::new())),
            pin_command_secret: std::env::var("PIN_COMMAND_SECRET").unwrap_or_default(),
            master_key: std::env::var("MASTER_DECRYPTION_KEY")
                .unwrap_or_else(|_| "0123456789abcdef0123456789abcdef".to_string()),
        }
    }
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Some(pool),
            ..Self::default()
        }
    }

    pub async fn pin_lock(&self, widget_id: &str) -> Arc<Mutex<()>> {
        let mut locks = self.pin_locks.write().await;
        locks
            .entry(widget_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    pub fn publish_widget(&self, widget_id: &str, message: String) {
        if let Some(session) = self.widget_sessions.lock().unwrap().get(widget_id) {
            let _ = session.tx.send(message);
        }
    }

    pub fn session_count(&self) -> usize {
        self.widget_sessions.lock().unwrap().len()
    }

    pub fn session_identity(&self, widget_id: &str) -> Option<usize> {
        self.widget_sessions
            .lock()
            .unwrap()
            .get(widget_id)
            .map(|session| Arc::as_ptr(session) as usize)
    }

    pub fn session_connection_count(&self, widget_id: &str) -> Option<usize> {
        self.widget_sessions
            .lock()
            .unwrap()
            .get(widget_id)
            .map(|session| session.lifetime.lock().unwrap().connections)
    }

    pub fn session_run_generation(&self, widget_id: &str) -> Option<u64> {
        self.widget_sessions
            .lock()
            .unwrap()
            .get(widget_id)
            .map(|session| session.lifetime.lock().unwrap().generation)
    }

    pub fn acquire_widget(self: &Arc<Self>, widget_id: &str, mock: bool) -> WidgetConnection {
        let mut sessions = self.widget_sessions.lock().unwrap();
        let session = sessions
            .entry(widget_id.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                Arc::new(WidgetSession {
                    tx,
                    lifetime: StdMutex::new(SessionLifetime {
                        connections: 0,
                        generation: 0,
                        worker: None,
                    }),
                })
            })
            .clone();
        let rx = session.tx.subscribe();
        let tx = session.tx.clone();
        let mut lifetime = session.lifetime.lock().unwrap();
        lifetime.connections += 1;
        if lifetime.worker.is_none() && (mock || self.pool.is_some()) {
            lifetime.generation += 1;
            let generation = lifetime.generation;
            let cancel = CancellationToken::new();
            lifetime.worker = Some(WorkerRun {
                generation,
                cancel: cancel.clone(),
            });
            self.start_worker(
                widget_id.to_string(),
                session.clone(),
                generation,
                cancel,
                mock,
            );
        }
        drop(lifetime);
        drop(sessions);
        WidgetConnection {
            state: self.clone(),
            widget_id: widget_id.to_string(),
            session,
            tx,
            rx: Some(rx),
        }
    }

    fn start_worker(
        self: &Arc<Self>,
        widget_id: String,
        session: Arc<WidgetSession>,
        generation: u64,
        cancel: CancellationToken,
        mock: bool,
    ) {
        let state = self.clone();
        tokio::spawn(async move {
            let tx = session.tx.clone();
            if mock {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
                loop {
                    tokio::select! {
                        _ = cancel.cancelled() => break,
                        _ = interval.tick() => {
                            let msg = ChatMessage::mock(&widget_id);
                            if let Ok(json) = serde_json::to_string(&msg) {
                                if tx.send(json).is_err() { break; }
                            }
                        }
                    }
                }
            } else if let Some(pool) = state.pool.clone() {
                if let Ok(widget_uuid) = uuid::Uuid::parse_str(&widget_id) {
                    if let Ok(tokens) =
                        crate::infrastructure::db::fetch_tokens(&pool, &widget_uuid).await
                    {
                        let mut handles = Vec::new();
                        if let Some(encrypted) = tokens.twitch {
                            let token = crate::infrastructure::db::decrypt_token(
                                &encrypted,
                                &state.master_key,
                            );
                            let (id, sender, stop) =
                                (widget_id.clone(), tx.clone(), cancel.clone());
                            handles.push(tokio::spawn(async move {
                                crate::infrastructure::twitch::spawn_twitch_client(
                                    id, token, sender, stop,
                                )
                                .await;
                            }));
                        }
                        if let Some(encrypted) = tokens.youtube {
                            let token = crate::infrastructure::db::decrypt_token(
                                &encrypted,
                                &state.master_key,
                            );
                            let refresh = tokens.youtube_refresh.map(|encrypted| {
                                crate::infrastructure::db::decrypt_token(
                                    &encrypted,
                                    &state.master_key,
                                )
                            });
                            let (id, sender, stop) =
                                (widget_id.clone(), tx.clone(), cancel.clone());
                            handles.push(tokio::spawn(async move {
                                crate::infrastructure::youtube::spawn_youtube_client(
                                    id, token, refresh, sender, stop,
                                )
                                .await;
                            }));
                        }
                        if let Some(encrypted) = tokens.kick {
                            let username = crate::infrastructure::db::decrypt_token(
                                &encrypted,
                                &state.master_key,
                            );
                            let (id, sender, stop) =
                                (widget_id.clone(), tx.clone(), cancel.clone());
                            handles.push(tokio::spawn(async move {
                                crate::infrastructure::kick::spawn_kick_client(
                                    id, username, sender, stop,
                                )
                                .await;
                            }));
                        }
                        for handle in handles {
                            let _ = handle.await;
                        }
                    }
                }
            }
            let mut lifetime = session.lifetime.lock().unwrap();
            if lifetime
                .worker
                .as_ref()
                .is_some_and(|run| run.generation == generation)
            {
                lifetime.worker = None;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn shared_session_releases_only_after_last_connection() {
        let state = Arc::new(AppState::default());
        let first = state.acquire_widget("same", true);
        let old_session = state
            .widget_sessions
            .lock()
            .unwrap()
            .get("same")
            .unwrap()
            .clone();
        let first_identity = state.session_identity("same");
        let second = state.acquire_widget("same", true);
        assert_eq!(state.session_identity("same"), first_identity);
        assert_eq!(state.session_connection_count("same"), Some(2));
        assert_eq!(state.session_run_generation("same"), Some(1));
        let other = state.acquire_widget("other", true);
        assert_eq!(state.session_count(), 2);
        drop(first);
        assert_eq!(state.session_connection_count("same"), Some(1));
        drop(second);
        assert_eq!(state.session_count(), 1);
        let replacement = state.acquire_widget("same", true);
        assert_ne!(state.session_identity("same"), first_identity);
        assert_eq!(state.session_run_generation("same"), Some(1));
        tokio::task::yield_now().await;
        assert_eq!(state.session_connection_count("same"), Some(1));
        drop(replacement);
        drop(other);
        drop(old_session);
        assert_eq!(state.session_count(), 0);
    }

    #[tokio::test]
    async fn ended_worker_run_restarts_on_later_connection() {
        let state = Arc::new(AppState::default());
        let first = state.acquire_widget("same", true);
        let session = state
            .widget_sessions
            .lock()
            .unwrap()
            .get("same")
            .unwrap()
            .clone();
        session
            .lifetime
            .lock()
            .unwrap()
            .worker
            .as_ref()
            .unwrap()
            .cancel
            .cancel();
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if session.lifetime.lock().unwrap().worker.is_none() {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let second = state.acquire_widget("same", true);
        assert_eq!(state.session_run_generation("same"), Some(2));
        assert_eq!(state.session_connection_count("same"), Some(2));
        drop(first);
        drop(second);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn final_release_cancels_old_worker_without_affecting_replacement() {
        let state = Arc::new(AppState::default());
        let old_connection = state.acquire_widget("same", true);
        let old_session = state
            .widget_sessions
            .lock()
            .unwrap()
            .get("same")
            .unwrap()
            .clone();
        let old_cancel = old_session
            .lifetime
            .lock()
            .unwrap()
            .worker
            .as_ref()
            .unwrap()
            .cancel
            .clone();
        drop(old_connection);
        assert!(old_cancel.is_cancelled());
        assert_eq!(state.session_count(), 0);

        // A current-thread executor cannot run the old completion between these steps.
        let replacement = state.acquire_widget("same", true);
        let replacement_identity = state.session_identity("same");
        assert_ne!(
            replacement_identity,
            Some(Arc::as_ptr(&old_session) as usize)
        );
        assert_eq!(state.session_run_generation("same"), Some(1));
        assert_eq!(Arc::strong_count(&old_session), 2);

        tokio::time::timeout(Duration::from_secs(1), async {
            while Arc::strong_count(&old_session) != 1 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("old worker did not finish after cancellation");
        assert_eq!(state.session_identity("same"), replacement_identity);
        assert_eq!(state.session_run_generation("same"), Some(1));
        assert_eq!(state.session_connection_count("same"), Some(1));
        drop(replacement);
    }
}
