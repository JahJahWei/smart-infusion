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
    
    
    let app = Router::new()
        .route("/", get(handle))
        .route("/infusion", get(api::fetch_infusion))
        .route("/infusion", post(api::insert_infusions));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to start server on 0.0.0.0:3000");

    println!("HTTP服务器已启动在 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();

    // open a connection to RabbitMQ server
    let connection = Connection::open(&OpenConnectionArguments::new(
        "localhost",
        5672,
        "admin",
        "120111432@qq.com",
    ))
    .await
    .unwrap();
    connection
        .register_callback(DefaultConnectionCallback)
        .await
        .unwrap();

    let channel = connection.open_channel(None).await.unwrap();
    channel
        .register_callback(DefaultChannelCallback)
        .await
        .unwrap();

    let (queue_name, _, _) = channel
        .queue_declare(QueueDeclareArguments::default())
        .await
        .unwrap()
        .unwrap();

    let routing_key = "amqprs.example";
    let exchange_name = "amq.topic";
    channel
        .queue_bind(QueueBindArguments::new(
            &queue_name,
            exchange_name,
            routing_key,
    ))
    .await
    .unwrap();

    let args = BasicConsumeArguments::new(
        &queue_name,
        "example_basic_pub_sub"
    );

        channel
            .basic_consume(MyConsumer, args)
            .await
            .unwrap();
}

async fn handle() -> &'static str {
    "Hello, World!"
}

struct MyConsumer;

#[async_trait]
impl AsyncConsumer for MyConsumer {
    async fn consume(&mut self, channel: &Channel, deliver: Deliver, basic_properties: BasicProperties, content: Vec<u8>) {
        let msg = String::from_utf8_lossy(&content);
        println!("Received message: {}", msg);
    }
}

// AMQP 设置和消息处理
