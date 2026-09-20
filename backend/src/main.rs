use backend::application::state::AppState;
use backend::infrastructure::web::create_router;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState::default());
    let app = create_router(state);
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
