
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/heartbeat", post(heart_beat))
        .route("/breath", post(breath));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to start server on 0.0.0.0:3000");

    axum::serve(listener, app.into_make_service()).await.unwrap();
}
