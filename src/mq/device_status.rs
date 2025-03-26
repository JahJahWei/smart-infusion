use amqprs::{channel::Channel, consumer::AsyncConsumer, BasicProperties, Deliver};
use axum::async_trait;
use serde::{Deserialize, Serialize};

use crate::db::get_db;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    pub device_id: String,
    pub status: u8,
}

pub struct DeviceStatusConsumer;

#[async_trait]
impl AsyncConsumer for DeviceStatusConsumer {
    async fn consume(&mut self, channel: &Channel, deliver: Deliver, basic_properties: BasicProperties, content: Vec<u8>) {
        let msg = String::from_utf8_lossy(&content);
        let device_status = serde_json::from_str::<DeviceStatus>(&msg);
        match device_status {
            Ok(device_status) => {
                println!("Device status: {:?}", device_status);
                let db = get_db();
                todo!()
            }
            Err(e) => {
                eprintln!("Failed to parse device status: {}", e);
            }
        }
    }
}


