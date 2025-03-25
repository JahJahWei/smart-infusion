use amqprs::{channel::Channel, consumer::AsyncConsumer, BasicProperties, Deliver};
use axum::async_trait;

pub struct StartDeviceConsumer;

#[async_trait]
impl AsyncConsumer for StartDeviceConsumer {
    async fn consume(&mut self, channel: &Channel, deliver: Deliver, basic_properties: BasicProperties, content: Vec<u8>) {
        println!(" [x] Received message: {:?}", deliver);
        let msg = String::from_utf8_lossy(&content);
        println!("Received message: {}", msg);
    }
}


