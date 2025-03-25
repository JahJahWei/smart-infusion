use axum::{routing::{get, post}, Router};
// use winapi::um::wincon::FreeConsole;
use serde::Deserialize;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;
use axum::Json;

mod api;
mod db;
mod mqtt;
mod broker;
use broker::{start_mqtt_broker, MqttConfig};

#[tokio::main]
async fn main() {
    // Don't show console window
    // unsafe {
    //     FreeConsole();
    // }

    db::init_db().await;

    // 启动MQTT服务器
    let mqtt_config = MqttConfig {
        host: "0.0.0.0".to_string(),
        port: 1883,
        max_connections: 1000,
        max_client_id_len: 256,
    };
    
    start_mqtt_broker(mqtt_config);
    println!("MQTT服务器已启动");
    
    mqtt::init_mqtt(
        "127.0.0.1".to_string(),
        1883,
        format!("smart-infusion-client-{}", uuid::Uuid::new_v4())
    ).await;

    // 注册不同主题的处理器
    mqtt::register_handler("infusion/+/status", |topic, payload| {
        println!("处理输液状态更新: {} - {}", topic, payload);
        // 在这里处理输液状态更新逻辑
    }).await;

    mqtt::register_handler("infusion/+/alarm", |topic, payload| {
        println!("处理输液报警: {} - {}", topic, payload);
        // 在这里处理输液报警逻辑
    }).await;

    mqtt::register_handler("infusion/+/data", |topic, payload| {
        println!("处理输液数据: {} - {}", topic, payload);
        // 在这里处理输液数据逻辑
    }).await;

    let app = Router::new()
        .route("/", get(handle))
        .route("/infusion", get(api::fetch_infusion))
        .route("/infusion", post(api::insert_infusions))
        // 添加MQTT相关的API端点
        .route("/mqtt/publish", post(publish_mqtt_message))
        .route("/mqtt/status", get(mqtt_status))
        .route("/mqtt/stats", get(get_broker_stats));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to start server on 0.0.0.0:3000");

    println!("HTTP服务器已启动在 0.0.0.0:3000");
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

async fn handle() -> &'static str {
    "Hello, World!"
}

// MQTT发布请求的数据结构
#[derive(Debug, Deserialize)]
struct MqttPublishRequest {
    topic: String,
    message: String,
}

// 处理MQTT发布消息的API端点
async fn publish_mqtt_message(
    Json(payload): Json<MqttPublishRequest>,
) -> impl IntoResponse {
    match mqtt::publish_message(&payload.topic, &payload.message).await {
        Ok(_) => (StatusCode::OK, "Message published successfully").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to publish message: {}", e)).into_response(),
    }
}

// MQTT服务器状态
#[derive(Debug, Serialize)]
struct MqttStatus {
    server_connections: usize,
    client_connected: bool,
}

// 获取MQTT服务器状态
async fn mqtt_status() -> impl IntoResponse {
    // 简化状态检查，因为mqtt_server中还没有实现get_connections_count
    let server_connections = 0; // 暂时硬编码为0
    let client_connected = mqtt::is_mqtt_connected().await;
    let status = MqttStatus { 
        server_connections,
        client_connected 
    };
    (StatusCode::OK, Json(status)).into_response()
}

#[derive(Debug, Serialize)]
struct BrokerStats {
    message_count: usize,
    server_running: bool,
}

async fn get_broker_stats() -> impl IntoResponse {
    let stats = BrokerStats {
        message_count: broker::get_message_count(),
        server_running: *broker::MQTT_STARTED.lock().unwrap(),
    };
    
    (StatusCode::OK, Json(stats)).into_response()
}
