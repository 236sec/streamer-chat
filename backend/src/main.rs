use backend::application::state::AppState;
use backend::infrastructure::web::create_router;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Failed to connect to the database");

    let refresh_seconds = backend::application::viewer_count::refresh_seconds_from(
        std::env::var("VIEWER_COUNT_REFRESH_SECONDS")
            .ok()
            .as_deref(),
    );
    let state = Arc::new(AppState::new(pool).with_viewer_count_refresh_seconds(refresh_seconds));

    let app = create_router(state);
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
