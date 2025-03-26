use amqprs::{channel::Channel, consumer::AsyncConsumer, BasicProperties, Deliver};
use axum::async_trait;
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    pub device_id: String,
    pub bed_no: String,
}

pub struct BindingConsumer;

#[async_trait]
impl AsyncConsumer for BindingConsumer {
    async fn consume(&mut self, channel: &Channel, deliver: Deliver, basic_properties: BasicProperties, content: Vec<u8>) {
        let msg = String::from_utf8_lossy(&content);
        let binding = serde_json::from_str::<Binding>(&msg);
        match binding {
            Ok(binding) => {
                println!("Binding: {:?}", binding);
            }
            Err(e) => {
                eprintln!("Failed to parse binding: {}", e);
            }
        }
    }
}
