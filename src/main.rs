use amqprs::{callbacks::{DefaultChannelCallback, DefaultConnectionCallback}, channel::{BasicConsumeArguments, Channel, QueueBindArguments, QueueDeclareArguments}, connection::{Connection, OpenConnectionArguments}, consumer::{AsyncConsumer, DefaultConsumer}, BasicProperties, Deliver};
use axum::{async_trait, routing::{get, post}, Router};
// use winapi::um::wincon::FreeConsole;

mod api;
mod db;
mod mq;

#[tokio::main]
async fn main() {
    // Don't show console window
    // unsafe {
    //     FreeConsole();
    // }

    // 初始化数据库
    db::init_db().await;
    
    // 在单独的任务中运行 MQ
    tokio::spawn(async {
        mq::init_mq().await;
    });
    
    let app = Router::new()
        .route("/", get(handle))
        .route("/infusion", get(api::fetch_infusion))
        .route("/infusion", post(api::insert_infusions));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to start server on 0.0.0.0:3000");

    println!("HTTP服务器已启动在 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn handle() -> &'static str {
    "Hello, World!"
}