use backend::application::state::AppState;
use backend::infrastructure::web::create_router;
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as TungsteniteMessage};

#[tokio::test]
async fn test_ping_pong_websocket() {
    let state = Arc::new(AppState::default());
    let app = create_router(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let ws_url = format!("ws://{}/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    let ping_msg = serde_json::json!({ "type": "ping" });
    ws_stream
        .send(TungsteniteMessage::Text(ping_msg.to_string()))
        .await
        .unwrap();

    if let Some(msg) = ws_stream.next().await {
        let msg = msg.unwrap();
        if let TungsteniteMessage::Text(text) = msg {
            let json: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(json["type"], "pong");
        } else {
            panic!("Expected text message");
        }
    } else {
        panic!("No message received");
    }
}
