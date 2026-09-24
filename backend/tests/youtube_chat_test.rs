use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

static TEST_MUTEX: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test]
async fn test_youtube_chat_ingestion_lifecycle() {
    let _guard = TEST_MUTEX.lock().await;

    // Set up mock HTTP server for YouTube Data API
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);

    std::env::set_var("YOUTUBE_API_BASE_URL", &base_url);
    std::env::set_var("YOUTUBE_DISCOVERY_INTERVAL_SECS", "1");

    let discovery_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let poll_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let page_token_seen = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let discoveries = discovery_count.clone();
    let polls = poll_count.clone();
    let saw_page_token = page_token_seen.clone();

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let discoveries = discoveries.clone();
            let polls = polls.clone();
            let saw_page_token = saw_page_token.clone();
            tokio::spawn(async move {
                let mut buf = [0; 4096];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);

                if request.contains("/liveBroadcasts") {
                    discoveries.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    let body = serde_json::json!({
                        "items": [
                            {
                                "id": "broadcast_123",
                                "snippet": {
                                    "title": "My Awesome Stream",
                                    "liveChatId": "chat_abc"
                                }
                            }
                        ]
                    });
                    let body_str = body.to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body_str.len(),
                        body_str
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                } else if request.contains("/liveChat/messages") {
                    polls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if request.contains("pageToken=token_2") {
                        saw_page_token.store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                    let body = serde_json::json!({
                        "pollingIntervalMillis": 200,
                        "nextPageToken": "token_2",
                        "items": [
                            {
                                "kind": "youtube#liveChatMessage",
                                "id": "yt_msg_1",
                                "snippet": {
                                    "type": "textMessageEvent",
                                    "displayMessage": "Hello from YouTube test!"
                                },
                                "authorDetails": {
                                    "displayName": "YT Tester",
                                    "profileImageUrl": "https://example.com/avatar.png"
                                }
                            }
                        ]
                    });
                    let body_str = body.to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body_str.len(),
                        body_str
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                } else {
                    let response =
                        "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes()).await;
                }
            });
        }
    });

    let (tx, mut rx) = tokio::sync::broadcast::channel::<String>(10);
    let tx_clone = tx.clone();

    let widget_id = "test_widget_yt".to_string();
    let token = "test_yt_token".to_string();

    let handle = tokio::spawn(async move {
        backend::infrastructure::youtube::spawn_youtube_client(
            widget_id,
            token,
            None,
            tx_clone,
            tokio_util::sync::CancellationToken::new(),
        )
        .await;
    });

    // Verify chat message is received
    let timeout = tokio::time::timeout(Duration::from_secs(3), rx.recv()).await;
    let msg = timeout
        .expect("Did not receive a YouTube chat message in time")
        .expect("Channel closed unexpectedly");

    let json: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(json["type"], "chat_message");
    assert_eq!(json["platform"], "youtube");
    assert_eq!(json["author"], "YT Tester");
    assert_eq!(json["content"], "Hello from YouTube test!");
    assert_eq!(json["avatar_url"], "https://example.com/avatar.png");

    tokio::time::timeout(Duration::from_secs(2), async {
        while poll_count.load(std::sync::atomic::Ordering::SeqCst) < 3 {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("healthy chat polling did not continue");
    assert_eq!(discovery_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert!(page_token_seen.load(std::sync::atomic::Ordering::SeqCst));

    // Teardown: drop receiver
    drop(rx);

    // Give the client a moment to detect receiver_count == 0 and exit
    let exit_result = tokio::time::timeout(Duration::from_secs(2), handle).await;
    assert!(
        exit_result.is_ok(),
        "Client did not terminate when receivers dropped"
    );
}

#[tokio::test]
async fn test_youtube_chat_token_refresh_on_401() {
    let _guard = TEST_MUTEX.lock().await;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);

    std::env::set_var("YOUTUBE_API_BASE_URL", &base_url);
    std::env::set_var("YOUTUBE_TOKEN_URL", format!("{}/token", base_url));
    std::env::set_var("YOUTUBE_DISCOVERY_INTERVAL_SECS", "1");

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0; 4096];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);

                if request.contains("/token") {
                    let body = serde_json::json!({
                        "access_token": "refreshed_valid_token",
                        "expires_in": 3600,
                        "token_type": "Bearer"
                    });
                    let body_str = body.to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body_str.len(),
                        body_str
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                } else if request.contains("/liveBroadcasts") {
                    if request.contains("refreshed_valid_token") {
                        let body = serde_json::json!({
                            "items": [
                                {
                                    "id": "broadcast_refreshed",
                                    "snippet": {
                                        "title": "Stream After Refresh",
                                        "liveChatId": "chat_refreshed"
                                    }
                                }
                            ]
                        });
                        let body_str = body.to_string();
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                            body_str.len(),
                            body_str
                        );
                        let _ = stream.write_all(response.as_bytes()).await;
                    } else {
                        // Return 401 Unauthorized for expired initial token
                        let response = "HTTP/1.1 401 Unauthorized\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
                        let _ = stream.write_all(response.as_bytes()).await;
                    }
                } else if request.contains("/liveChat/messages") {
                    let body = serde_json::json!({
                        "pollingIntervalMillis": 100,
                        "items": [
                            {
                                "kind": "youtube#liveChatMessage",
                                "id": "yt_msg_refreshed",
                                "snippet": {
                                    "type": "textMessageEvent",
                                    "displayMessage": "Message after refresh!"
                                },
                                "authorDetails": {
                                    "displayName": "Refreshed User"
                                }
                            }
                        ]
                    });
                    let body_str = body.to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body_str.len(),
                        body_str
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                } else {
                    let response =
                        "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes()).await;
                }
            });
        }
    });

    let (tx, mut rx) = tokio::sync::broadcast::channel::<String>(10);
    let tx_clone = tx.clone();

    let widget_id = "widget_refreshed".to_string();
    let initial_expired_token = "initial_expired_token".to_string();
    let refresh_token = Some("valid_refresh_token".to_string());

    let handle = tokio::spawn(async move {
        backend::infrastructure::youtube::spawn_youtube_client(
            widget_id,
            initial_expired_token,
            refresh_token,
            tx_clone,
            tokio_util::sync::CancellationToken::new(),
        )
        .await;
    });

    let timeout = tokio::time::timeout(Duration::from_secs(3), rx.recv()).await;
    let msg = timeout
        .expect("Did not receive a message after token refresh")
        .expect("Channel closed");

    let json: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(json["content"], "Message after refresh!");
    assert_eq!(json["author"], "Refreshed User");

    drop(rx);
    let exit_result = tokio::time::timeout(Duration::from_secs(2), handle).await;
    assert!(
        exit_result.is_ok(),
        "Client did not terminate when receivers dropped"
    );
}

#[tokio::test]
async fn test_youtube_chat_retry_on_initial_404() {
    let _guard = TEST_MUTEX.lock().await;

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);

    std::env::set_var("YOUTUBE_API_BASE_URL", &base_url);
    std::env::set_var("YOUTUBE_DISCOVERY_INTERVAL_SECS", "1");

    let chat_request_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count_clone = chat_request_count.clone();

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let count = count_clone.clone();
            tokio::spawn(async move {
                let mut buf = [0; 4096];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);

                if request.contains("/liveBroadcasts") {
                    let body = serde_json::json!({
                        "items": [
                            {
                                "id": "broadcast_init",
                                "snippet": {
                                    "title": "Stream Initializing",
                                    "liveChatId": "chat_initializing"
                                },
                                "status": {
                                    "lifeCycleStatus": "live"
                                }
                            }
                        ]
                    });
                    let body_str = body.to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        body_str.len(),
                        body_str
                    );
                    let _ = stream.write_all(response.as_bytes()).await;
                } else if request.contains("/liveChat/messages") {
                    let c = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if c == 0 {
                        // First request: chat is not yet ready, return 404 liveChatNotFound
                        let err_body = serde_json::json!({
                            "error": {
                                "code": 404,
                                "message": "The live chat is not found",
                                "errors": [{ "reason": "liveChatNotFound" }]
                            }
                        });
                        let err_str = err_body.to_string();
                        let response = format!(
                            "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                            err_str.len(),
                            err_str
                        );
                        let _ = stream.write_all(response.as_bytes()).await;
                    } else {
                        // Second request (after retry): chat ready!
                        let body = serde_json::json!({
                            "pollingIntervalMillis": 100,
                            "items": [
                                {
                                    "kind": "youtube#liveChatMessage",
                                    "id": "yt_msg_retry_success",
                                    "snippet": {
                                        "type": "textMessageEvent",
                                        "displayMessage": "Message after 404 retry!"
                                    },
                                    "authorDetails": {
                                        "displayName": "Recovered User"
                                    }
                                }
                            ]
                        });
                        let body_str = body.to_string();
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                            body_str.len(),
                            body_str
                        );
                        let _ = stream.write_all(response.as_bytes()).await;
                    }
                } else {
                    let response =
                        "HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes()).await;
                }
            });
        }
    });

    let (tx, mut rx) = tokio::sync::broadcast::channel::<String>(10);
    let tx_clone = tx.clone();

    let widget_id = "widget_retry_test".to_string();
    let token = "valid_token".to_string();

    let handle = tokio::spawn(async move {
        backend::infrastructure::youtube::spawn_youtube_client(
            widget_id,
            token,
            None,
            tx_clone,
            tokio_util::sync::CancellationToken::new(),
        )
        .await;
    });

    let timeout = tokio::time::timeout(Duration::from_secs(5), rx.recv()).await;
    let msg = timeout
        .expect("Did not receive a message after 404 retry")
        .expect("Channel closed");

    let json: serde_json::Value = serde_json::from_str(&msg).unwrap();
    assert_eq!(json["content"], "Message after 404 retry!");
    assert_eq!(json["author"], "Recovered User");

    drop(rx);
    let exit_result = tokio::time::timeout(Duration::from_secs(2), handle).await;
    assert!(
        exit_result.is_ok(),
        "Client did not terminate when receivers dropped"
    );
}

#[tokio::test]
async fn test_youtube_rediscovers_after_terminal_chat_404() {
    let _guard = TEST_MUTEX.lock().await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    std::env::set_var("YOUTUBE_API_BASE_URL", format!("http://{addr}"));
    std::env::set_var("YOUTUBE_DISCOVERY_INTERVAL_SECS", "1");
    let requests = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));
    let observed = requests.clone();
    let server = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let observed = observed.clone();
            tokio::spawn(async move {
                let mut buf = [0; 4096];
                let n = stream.read(&mut buf).await.unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]);
                let path = if request.contains("/liveBroadcasts") {
                    "discovery"
                } else if request.contains("/liveChat/messages") {
                    "chat"
                } else {
                    "video"
                };
                observed.lock().await.push(path.to_string());
                let (status, body) = match path {
                    "discovery" => (
                        "200 OK",
                        serde_json::json!({"items":[{"id":"broadcast_404","snippet":{"liveChatId":"chat_404"}}]}).to_string(),
                    ),
                    "chat" => ("404 Not Found", "{}".to_string()),
                    _ => ("200 OK", serde_json::json!({"items":[]}).to_string()),
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
            });
        }
    });
    let (tx, _rx) = tokio::sync::broadcast::channel::<String>(10);
    let cancel = tokio_util::sync::CancellationToken::new();
    let worker_cancel = cancel.clone();
    let worker = tokio::spawn(async move {
        backend::infrastructure::youtube::spawn_youtube_client(
            "widget_404".to_string(),
            "token".to_string(),
            None,
            tx,
            worker_cancel,
        )
        .await;
    });
    tokio::time::timeout(Duration::from_secs(16), async {
        loop {
            if requests
                .lock()
                .await
                .iter()
                .filter(|path| *path == "discovery")
                .count()
                >= 2
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("discovery did not resume after terminal 404");
    cancel.cancel();
    tokio::time::timeout(Duration::from_secs(2), worker)
        .await
        .unwrap()
        .unwrap();
    server.abort();
    let paths = requests.lock().await.clone();
    assert_eq!(paths[0], "discovery");
    assert_eq!(paths[1], "chat");
    assert_eq!(paths[2], "video");
    assert!(paths[3..8].iter().all(|path| path == "chat"));
    assert_eq!(paths[8], "discovery");
}
