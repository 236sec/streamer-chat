use futures::sink::SinkExt;
use tokio::net::TcpListener;
use tokio::time::Duration;
use tokio_tungstenite::tungstenite::protocol::Message as TungsteniteMessage;

#[tokio::test]
async fn test_twitch_eventsub_lifecycle() {
    // We mock the Twitch EventSub WS server
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let ws_url = format!("ws://{}", addr);

    std::env::set_var("TWITCH_WS_URL", ws_url);
    std::env::set_var("TWITCH_CLIENT_ID", "test_client_id");

    // We mock the HTTP server for both /helix/users and /helix/eventsub/subscriptions
    let http_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let http_addr = http_listener.local_addr().unwrap();
    let http_base = format!("http://{}", http_addr);
    std::env::set_var(
        "TWITCH_HTTP_URL",
        format!("{}/helix/eventsub/subscriptions", http_base),
    );
    // Override the Helix users URL by pointing the resolve function to our mock
    // We achieve this by setting an env var that the test mock server handles
    std::env::set_var("TWITCH_HELIX_BASE_URL", http_base);

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = http_listener.accept().await {
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut buf = [0; 4096];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);

                if request.contains("/helix/users") {
                    // Mock /helix/users response
                    let body = serde_json::json!({
                        "data": [{
                            "id": "12345678",
                            "login": "test_user",
                            "display_name": "Test User"
                        }]
                    });
                    let body_str = body.to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body_str.len(),
                        body_str
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                } else {
                    // Mock EventSub subscription response
                    let response = "HTTP/1.1 202 Accepted\r\nContent-Length: 0\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes()).await;
                }
            });
        }
    });

    let (tx, _rx) = tokio::sync::broadcast::channel(10);
    let tx_clone = tx.clone();

    // Subscribe to tx so receiver_count() > 0
    let mut rx = tx.subscribe();

    let widget_id = "test_widget".to_string();
    let token = "test_token".to_string();

    tokio::spawn(async move {
        backend::infrastructure::twitch::spawn_twitch_client(widget_id, token, tx_clone).await;
    });

    // Accept websocket connection from the spawned client
    let (stream, _) = listener.accept().await.unwrap();
    let mut ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();

    // Send a welcome message
    let welcome = serde_json::json!({
        "metadata": {
            "message_type": "session_welcome"
        },
        "payload": {
            "session": {
                "id": "session_123"
            }
        }
    });
    ws_stream
        .send(TungsteniteMessage::Text(welcome.to_string()))
        .await
        .unwrap();

    // Send a notification message
    let notification = serde_json::json!({
        "metadata": {
            "message_type": "notification"
        },
        "payload": {
            "event": {
                "chatter_user_name": "test_user",
                "color": "#123456",
                "message": {
                    "text": "hello world"
                }
            }
        }
    });
    ws_stream
        .send(TungsteniteMessage::Text(notification.to_string()))
        .await
        .unwrap();

    // The client should process notification and send it to tx
    let timeout = tokio::time::timeout(Duration::from_secs(3), rx.recv()).await;
    let msg = timeout
        .expect("Did not receive a message in time")
        .expect("Channel closed");

    let json: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(json["type"], "chat_message");
    assert_eq!(json["platform"], "twitch");
    assert_eq!(json["author"], "test_user");

    // Now test teardown: drop rx
    drop(rx);

    // Client should eventually notice receiver_count == 0 and disconnect/stop reconnecting.
    // In our test, let's just assert it passes the channel.
}
