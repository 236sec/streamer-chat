use futures::sink::SinkExt;
use tokio::net::TcpListener;
use tokio::time::Duration;
use tokio_tungstenite::tungstenite::protocol::Message as TungsteniteMessage;

#[tokio::test]
async fn test_kick_chat_lifecycle() {
    let ws_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ws_addr = ws_listener.local_addr().unwrap();
    let ws_url = format!("ws://{}", ws_addr);
    std::env::set_var("KICK_WS_URL", ws_url);

    let http_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let http_addr = http_listener.local_addr().unwrap();
    let http_base = format!("http://{}", http_addr);
    std::env::set_var("KICK_API_BASE_URL", http_base);

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = http_listener.accept().await {
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0; 4096];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);

                if request.contains("/api/v1/channels/") {
                    let body = serde_json::json!({
                        "chatroom": {
                            "id": 12345
                        }
                    });
                    let body_str = body.to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body_str.len(),
                        body_str
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                } else {
                    let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes()).await;
                }
            });
        }
    });

    let (tx, _rx) = tokio::sync::broadcast::channel(10);
    let tx_clone = tx.clone();
    let mut rx = tx.subscribe();

    let widget_id = "test_widget".to_string();
    let username = "kick_test_user".to_string();

    tokio::spawn(async move {
        backend::infrastructure::kick::spawn_kick_client(
            widget_id,
            username,
            tx_clone,
            tokio_util::sync::CancellationToken::new(),
        )
        .await;
    });

    let (stream, _) = ws_listener.accept().await.unwrap();
    let mut ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();

    // Expect the client to send a pusher subscribe message
    if let Some(msg) = futures::StreamExt::next(&mut ws_stream).await {
        let msg = msg.unwrap();
        if let TungsteniteMessage::Text(text) = msg {
            let json: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(json["event"], "pusher:subscribe");
            assert_eq!(json["data"]["channel"], "chatrooms.12345.v2");
        }
    }

    // Send a message event
    let notification = serde_json::json!({
        "event": "App\\Events\\ChatMessageEvent",
        "data": "{\"id\":\"msg-1\",\"chatroom_id\":12345,\"content\":\"hello kick\",\"sender\":{\"id\":1,\"username\":\"kick_user\",\"slug\":\"kick_user\"}}"
    });
    ws_stream
        .send(TungsteniteMessage::Text(notification.to_string()))
        .await
        .unwrap();

    let timeout = tokio::time::timeout(Duration::from_secs(3), rx.recv()).await;
    let msg = timeout
        .expect("Did not receive a message in time")
        .expect("Channel closed");

    let json: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(json["type"], "chat_message");
    assert_eq!(json["platform"], "kick");
    assert_eq!(json["author"], "kick_user");
    assert_eq!(json["content"], "hello kick");
}
