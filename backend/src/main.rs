use backend::app;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = app();
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
