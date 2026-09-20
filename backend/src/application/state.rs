use tokio::sync::broadcast;

pub struct AppState {
    pub tx: broadcast::Sender<String>,
}

impl Default for AppState {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }
}
