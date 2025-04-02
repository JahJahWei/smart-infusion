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
mod rpc;

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
    
    tokio::spawn(async move {
        loop {
            info!("Start fetching data from API...");

            match HttpClient::new("http://172.16.80.253:1024".to_string(), None)
                .fetch_all_data()
                .await {
                    Ok(_) => {
                        info!("Data fetched and stored successfully");
                    }
                    Err(e) => {
                        error!("Failed to fetch and store data: {}", e);
                    }
                }

            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    });
    
    let app = Router::new()
        .route("/", get(handle))
        .route("/syncPatientData", get(api::sync_remote_patient_data))
        .route("/fetchPatientData", get(api::fetch_patients))
        .route("/syncDeviceData", get(api::sync_remote_device_data))
        .route("/fetchDeviceData", get(api::fetch_devices))
        .route("/syncBedData", get(api::sync_remote_bed_data))
        .route("/fetchBedData", get(api::fetch_beds))
        .route("/patientDetail", get(api::patient_detail))
        .route("/modifyDripRate", post(api::modify_drip_rate))
        .route("/turnOffDevice", post(api::turn_off_device))
        .route("/startDrip", post(api::start_drip))
        .route("/stopDrip", post(api::stop_drip))
        .route("/modifyPresetAmount", post(api::modify_preset_amount));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to start server on 0.0.0.0:3000");

    info!("HTTP server started on 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn handle() -> &'static str {
    "Hello, World!"
}