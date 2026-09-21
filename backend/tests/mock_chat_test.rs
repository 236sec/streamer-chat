use backend::application::state::AppState;
use backend::infrastructure::web::create_router;
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::time::Duration;
use tokio_tungstenite::connect_async;

#[tokio::test]
async fn test_mock_chat_broadcasting() {
    std::env::set_var("MOCK_CHAT", "true");

    let state = Arc::new(AppState::default());
    let app = create_router(state.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let ws_url = format!("ws://{}/ws/widget/123", addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    let timeout = tokio::time::timeout(Duration::from_secs(3), ws_stream.next()).await;
    let msg = timeout
        .expect("Did not receive a message in time")
        .expect("Stream closed")
        .unwrap();

    let text = msg.to_text().unwrap();
    let json: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(json["type"], "mock_message");
}
