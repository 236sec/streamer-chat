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

#[tokio::test]
async fn test_widget_reconnect_lifecycle() {
    let state = Arc::new(AppState::default());
    let app = create_router(state.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let ws_url = format!("ws://{}/ws/widget/reconnect_test?mock=true", addr);

    // First connection: connects and receives a mock message
    let (mut ws_stream1, _) = connect_async(&ws_url).await.expect("Failed to connect 1");
    let msg1 = tokio::time::timeout(tokio::time::Duration::from_secs(3), ws_stream1.next())
        .await
        .expect("Timeout on stream 1")
        .expect("Stream 1 closed")
        .unwrap();
    let text1 = msg1.to_text().unwrap();
    let json1: serde_json::Value = serde_json::from_str(text1).unwrap();
    assert_eq!(json1["type"], "chat_message");

    // Close first connection (simulating page reload or navigation)
    let _ = ws_stream1.close(None).await;
    drop(ws_stream1);

    // Wait a brief moment for close to process
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Second connection (reconnect): connects and receives new mock messages
    let (mut ws_stream2, _) = connect_async(&ws_url).await.expect("Failed to connect 2");
    let msg2 = tokio::time::timeout(tokio::time::Duration::from_secs(3), ws_stream2.next())
        .await
        .expect("Timeout on stream 2 (reconnect)")
        .expect("Stream 2 closed")
        .unwrap();
    let text2 = msg2.to_text().unwrap();
    let json2: serde_json::Value = serde_json::from_str(text2).unwrap();
    assert_eq!(json2["type"], "chat_message");

    let _ = ws_stream2.close(None).await;
}
