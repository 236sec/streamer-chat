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

#[tokio::test]
async fn widget_connections_share_session_and_route_by_widget() {
    let state = Arc::new(AppState::default());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = create_router(state.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let (mut first, _) = connect_async(format!("ws://{address}/ws/widget/first?mock=true"))
        .await
        .unwrap();
    // Consume the first tick before subscribing the second connection.
    let initial = tokio::time::timeout(tokio::time::Duration::from_secs(3), first.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(initial.to_text().unwrap()).unwrap()["widget_id"],
        "first"
    );
    let (mut second, _) = connect_async(format!("ws://{address}/ws/widget/first?mock=true"))
        .await
        .unwrap();
    let (mut other, _) = connect_async(format!("ws://{address}/ws/widget/other?mock=true"))
        .await
        .unwrap();
    assert_eq!(state.session_count(), 2);
    assert_eq!(state.session_connection_count("first"), Some(2));
    assert_eq!(state.session_run_generation("first"), Some(1));
    let (first_frame, second_frame, other_frame) =
        tokio::time::timeout(tokio::time::Duration::from_secs(3), async {
            tokio::join!(first.next(), second.next(), other.next())
        })
        .await
        .unwrap();
    let first_message: serde_json::Value =
        serde_json::from_str(first_frame.unwrap().unwrap().to_text().unwrap()).unwrap();
    let second_message: serde_json::Value =
        serde_json::from_str(second_frame.unwrap().unwrap().to_text().unwrap()).unwrap();
    let other_message: serde_json::Value =
        serde_json::from_str(other_frame.unwrap().unwrap().to_text().unwrap()).unwrap();
    assert_eq!(first_message["widget_id"], "first");
    assert_eq!(first_message, second_message);
    assert_eq!(other_message["widget_id"], "other");
    first.close(None).await.unwrap();
    tokio::time::timeout(tokio::time::Duration::from_secs(1), async {
        while state.session_connection_count("first") != Some(1) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let frame = tokio::time::timeout(tokio::time::Duration::from_secs(3), second.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let message: serde_json::Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
    assert_eq!(message["widget_id"], "first");
    second.close(None).await.unwrap();
    tokio::time::timeout(tokio::time::Duration::from_secs(1), async {
        while state.session_identity("first").is_some() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(state.session_count(), 1);
    other.close(None).await.unwrap();
}
