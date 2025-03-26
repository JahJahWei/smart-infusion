use amqprs::{callbacks::{DefaultChannelCallback, DefaultConnectionCallback}, channel::{BasicConsumeArguments, Channel, QueueBindArguments, QueueDeclareArguments}, connection::{Connection, OpenConnectionArguments}, consumer::DefaultConsumer};

use super::start_device::StartDeviceConsumer;

pub async fn init_mq() {
    let connection = Connection::open(&OpenConnectionArguments::new(
        "127.0.0.1",
        5673,
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
        .basic_consume(StartDeviceConsumer, args)
        .await
        .unwrap();

    // 保持连接对象存活
    // 这里可以考虑使用全局变量或者某种形式的连接管理器来持有连接
    std::future::pending::<()>().await;
}


