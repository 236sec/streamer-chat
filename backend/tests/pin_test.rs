use backend::application::state::AppState;
use backend::infrastructure::web::create_router;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::test]
#[ignore = "requires a disposable TEST_DATABASE_URL"]
async fn legacy_widget_resolves_to_canonical_identity() {
    let url = std::env::var("TEST_DATABASE_URL").expect("disposable Postgres required");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    let owner = uuid::Uuid::new_v4();
    let legacy = uuid::Uuid::new_v4();
    let canonical = uuid::Uuid::new_v4();
    sqlx::query("CREATE TEMP TABLE widgets (id UUID PRIMARY KEY, user_id UUID NOT NULL, pinned_message JSONB, pin_revision BIGINT NOT NULL DEFAULT 0)")
        .execute(&pool).await.unwrap();
    sqlx::query(
        "CREATE TEMP TABLE widget_identities (user_id UUID PRIMARY KEY, widget_id UUID NOT NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();
    for id in [legacy, canonical] {
        sqlx::query("INSERT INTO widgets (id, user_id) VALUES ($1, $2)")
            .bind(id)
            .bind(owner)
            .execute(&pool)
            .await
            .unwrap();
    }
    sqlx::query("INSERT INTO widget_identities (user_id, widget_id) VALUES ($1, $2)")
        .bind(owner)
        .bind(canonical)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        backend::infrastructure::db::canonical_widget_id(&pool, &legacy)
            .await
            .unwrap(),
        Some(canonical)
    );
    assert_eq!(
        backend::infrastructure::db::canonical_widget_id(&pool, &canonical)
            .await
            .unwrap(),
        Some(canonical)
    );
    assert_eq!(
        backend::infrastructure::db::canonical_widget_id(&pool, &uuid::Uuid::new_v4())
            .await
            .unwrap(),
        None
    );
    let state = Arc::new(AppState::new(pool));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server_state = state.clone();
    tokio::spawn(async move {
        axum::serve(listener, create_router(server_state))
            .await
            .unwrap()
    });
    use futures::StreamExt;
    let (mut socket, _) =
        tokio_tungstenite::connect_async(format!("ws://{address}/ws/widget/{legacy}"))
            .await
            .unwrap();
    let frame = socket.next().await.unwrap().unwrap();
    let event: serde_json::Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
    assert_eq!(event["type"], "pin_state");
    assert_eq!(state.session_count(), 1);
    assert!(state.session_identity(&canonical.to_string()).is_some());
    assert!(state.session_identity(&legacy.to_string()).is_none());
}

#[tokio::test]
#[ignore = "requires a disposable TEST_DATABASE_URL"]
async fn open_legacy_socket_switches_after_first_mapping_without_reconnect() {
    use futures::StreamExt;
    let url = std::env::var("TEST_DATABASE_URL").expect("disposable Postgres required");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    let owner = uuid::Uuid::new_v4();
    let legacy = uuid::Uuid::new_v4();
    let canonical = uuid::Uuid::new_v4();
    sqlx::query("CREATE TEMP TABLE widgets (id UUID PRIMARY KEY, user_id UUID NOT NULL, pinned_message JSONB, pin_revision BIGINT NOT NULL DEFAULT 0)")
        .execute(&pool).await.unwrap();
    sqlx::query(
        "CREATE TEMP TABLE widget_identities (user_id UUID PRIMARY KEY, widget_id UUID NOT NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();
    for id in [legacy, canonical] {
        sqlx::query("INSERT INTO widgets (id, user_id) VALUES ($1, $2)")
            .bind(id)
            .bind(owner)
            .execute(&pool)
            .await
            .unwrap();
    }
    let state = Arc::new(AppState::new(pool.clone()));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server_state = state.clone();
    tokio::spawn(async move {
        axum::serve(listener, create_router(server_state))
            .await
            .unwrap()
    });
    let (mut socket, _) =
        tokio_tungstenite::connect_async(format!("ws://{address}/ws/widget/{legacy}"))
            .await
            .unwrap();
    let frame = socket.next().await.unwrap().unwrap();
    let initial: serde_json::Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
    assert_eq!(initial["type"], "pin_state");
    assert!(state.session_identity(&legacy.to_string()).is_some());
    let pin = serde_json::json!({
        "id":"chat-1","type":"chat_message","widget_id":canonical.to_string(),
        "platform":"twitch","author":"Viewer","content":"Pinned after mapping",
        "fragments":[{"type":"text","text":"Pinned after mapping"}]
    });
    sqlx::query("UPDATE widgets SET pinned_message = $1, pin_revision = 4 WHERE id = $2")
        .bind(pin)
        .bind(canonical)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO widget_identities (user_id, widget_id) VALUES ($1, $2)")
        .bind(owner)
        .bind(canonical)
        .execute(&pool)
        .await
        .unwrap();
    let frame = tokio::time::timeout(std::time::Duration::from_secs(15), socket.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let mapped: serde_json::Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
    assert_eq!(mapped["type"], "pin_state");
    assert_eq!(mapped["message"]["widget_id"], canonical.to_string());
    assert_eq!(mapped["revision"], 4);
    assert!(state.session_identity(&canonical.to_string()).is_some());
    assert!(state.session_identity(&legacy.to_string()).is_none());
    state.publish_widget(&canonical.to_string(), serde_json::json!({"type":"chat_message","widget_id":canonical.to_string(),"content":"new chat"}).to_string());
    let frame = socket.next().await.unwrap().unwrap();
    let chat: serde_json::Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
    assert_eq!(chat["content"], "new chat");
}

#[tokio::test]
async fn pin_command_requires_server_secret() {
    let state = Arc::new(AppState::default());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, create_router(state.clone()))
            .await
            .unwrap()
    });
    let response = reqwest::Client::new()
        .post(format!(
            "http://{address}/internal/widgets/00000000-0000-0000-0000-000000000001/pin"
        ))
        .json(&serde_json::json!({"action":"unpin"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn pin_command_rejects_invalid_payload_after_authentication() {
    let mut state = AppState::default();
    state.pin_command_secret = "test-only-secret".to_string();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, create_router(Arc::new(state)))
            .await
            .unwrap()
    });
    let response = reqwest::Client::new()
        .post(format!("http://{address}/internal/widgets/00000000-0000-0000-0000-000000000001/pin"))
        .header("x-pin-secret", "test-only-secret")
        .json(&serde_json::json!({"action":"pin","message":{"id":"1","type":"chat_message","widget_id":"different","platform":"twitch","author":"A","content":"hello","fragments":[]}}))
        .send().await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[ignore = "requires a disposable TEST_DATABASE_URL"]
async fn pin_broadcasts_to_one_widget_and_reconnects_from_database() {
    let url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must point to disposable Postgres");
    use futures::StreamExt;
    use sqlx::postgres::PgPoolOptions;
    use tokio_tungstenite::connect_async;
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    sqlx::query("CREATE TEMP TABLE widgets (id UUID PRIMARY KEY, pinned_message JSONB, pin_revision BIGINT NOT NULL DEFAULT 0)")
        .execute(&pool).await.unwrap();
    let first = uuid::Uuid::new_v4();
    let second = uuid::Uuid::new_v4();
    for id in [first, second] {
        sqlx::query("INSERT INTO widgets (id) VALUES ($1)")
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }
    let mut state = AppState::new(pool);
    state.pin_command_secret = "test-only-secret".to_string();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, create_router(Arc::new(state)))
            .await
            .unwrap()
    });
    let (mut first_ws, _) = connect_async(format!("ws://{address}/ws/widget/{first}"))
        .await
        .unwrap();
    let (mut second_ws, _) = connect_async(format!("ws://{address}/ws/widget/{second}"))
        .await
        .unwrap();
    for ws in [&mut first_ws, &mut second_ws] {
        let frame = ws.next().await.unwrap().unwrap();
        let event: serde_json::Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
        assert_eq!(event["type"], "pin_state");
        assert_eq!(event["revision"], 0);
    }
    let message = serde_json::json!({
        "id":"chat-1","type":"chat_message","widget_id":first.to_string(),
        "platform":"twitch","author":"Viewer","avatar_url":null,"color":null,
        "badges":[],"content":"Hello","fragments":[{"type":"text","text":"Hello"}]
    });
    let client = reqwest::Client::new();
    let command_url = format!("http://{address}/internal/widgets/{first}/pin");
    let response = client
        .post(&command_url)
        .header("x-pin-secret", "test-only-secret")
        .json(&serde_json::json!({"action":"pin","message":message}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let pin: serde_json::Value = response.json().await.unwrap();
    assert_eq!(pin["revision"], 1);
    let frame = first_ws.next().await.unwrap().unwrap();
    let event: serde_json::Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
    assert_eq!(event["type"], "pin_message");
    assert_eq!(event["message"]["content"], "Hello");
    assert!(
        tokio::time::timeout(std::time::Duration::from_millis(100), second_ws.next())
            .await
            .is_err()
    );
    let (mut late_ws, _) = connect_async(format!("ws://{address}/ws/widget/{first}"))
        .await
        .unwrap();
    let frame = late_ws.next().await.unwrap().unwrap();
    let snapshot: serde_json::Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
    assert_eq!(snapshot["type"], "pin_state");
    assert_eq!(snapshot["revision"], 1);
    assert_eq!(snapshot["message"]["content"], "Hello");
    let response = client
        .post(&command_url)
        .header("x-pin-secret", "test-only-secret")
        .json(&serde_json::json!({"action":"unpin"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let frame = first_ws.next().await.unwrap().unwrap();
    let event: serde_json::Value = serde_json::from_str(frame.to_text().unwrap()).unwrap();
    assert_eq!(event["type"], "unpin_message");
    assert_eq!(event["revision"], 2);
}

#[tokio::test]
async fn widget_connection_closes_when_pin_snapshot_cannot_load() {
    use futures::StreamExt;
    use sqlx::postgres::PgPoolOptions;
    use tokio_tungstenite::connect_async;
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_millis(300))
        .connect_lazy("postgres://postgres:postgres@127.0.0.1:1/postgres")
        .unwrap();
    let state = Arc::new(AppState::new(pool));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server_state = state.clone();
    tokio::spawn(async move {
        axum::serve(listener, create_router(server_state))
            .await
            .unwrap()
    });
    let widget_id = uuid::Uuid::new_v4();
    let (mut ws, _) = connect_async(format!("ws://{address}/ws/widget/{widget_id}"))
        .await
        .unwrap();
    let frame = tokio::time::timeout(std::time::Duration::from_secs(2), ws.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(frame.is_close());
    assert_eq!(state.session_count(), 0);
}
