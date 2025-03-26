use std::time::Duration;

use axum::{routing::{get, post}, Router};
use http_client::HttpClient;
use tracing::{info, error, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
// use winapi::um::wincon::FreeConsole;

mod api;
mod db;
mod mq;
mod repository;
mod http_client;

#[tokio::main]
async fn main() {
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "logs",  
        "application.log",  
    );
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    tracing_subscriber::registry()
        .with(fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false))  
        .with(EnvFilter::from_default_env()
            .add_directive(Level::INFO.into()))
        .init();

    info!("Start the application");

    // Don't show console window
    // unsafe {
    //     FreeConsole();
    // }

    db::init_db().await;
    info!("Database initialization completed");
    
    tokio::spawn(async {
        mq::init_mq().await;
    });
    info!("MQ initialization completed");
    
    let http_client = HttpClient::new("https://api.example.com".to_string());
    
    // tokio::spawn(async move {
    //     loop {
    //         info!("Start fetching data from API...");
    //         match http_client.fetch_and_store_patients().await {
    //             Ok(_) => info!("Successfully fetched and stored patients"),
    //             Err(err) => error!("Failed to fetch data from API: {}", err),
    //         }
            
    //         tokio::time::sleep(Duration::from_secs(3600)).await;
    //     }
    // });
    
    let app = Router::new()
        .route("/", get(handle))
        .route("/fetch-external-data", get(api::fetch_external_data))
        .route("/fetch-patients", get(api::fetch_patients))
        .route("/sync-devices", get(api::sync_devices))
        .route("/fetch-devices", get(api::fetch_devices))
        .route("/sync-beds", get(api::sync_beds))
        .route("/fetch-beds", get(api::fetch_beds))
        .route("/fetch-infusions", get(api::fetch_infusions));



    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to start server on 0.0.0.0:3000");

    info!("HTTP server started on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn handle() -> &'static str {
    "Hello, World!"
}