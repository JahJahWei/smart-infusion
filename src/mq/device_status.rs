use amqprs::{channel::Channel, consumer::AsyncConsumer, BasicProperties, Deliver};
use axum::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{info, error, Level};

use crate::repository::update_device_status;

use super::{publish_alarm, Alarm};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStatus {
    pub device_id: String,
    pub status: u16,
}

pub struct DeviceStatusConsumer;

#[async_trait]
impl AsyncConsumer for DeviceStatusConsumer {
    async fn consume(&mut self, channel: &Channel, deliver: Deliver, basic_properties: BasicProperties, content: Vec<u8>) {
        let msg = String::from_utf8_lossy(&content);
        let device_status = serde_json::from_str::<DeviceStatus>(&msg);
        match device_status {
            Ok(device_status) => {
                match device_status.status {
                    1 => {
                        match update_device_status(device_status.device_id.clone(), device_status.status).await {
                            Ok(_) => {
                                println!("device status updated");
                                publish_alarm(Alarm::new(device_status.device_id, "未绑定".to_string())).await;
                            }
                            Err(e) => {
                                error!("Failed to update device status: {}", e);
                            }
                        }
                    }
                    4 => {
                        match update_device_status(device_status.device_id.clone(), device_status.status).await {
                            Ok(_) => {
                                println!("device status updated");
                                todo!("use websocket to send message to client");
                            }
                            Err(e) => {
                                error!("Failed to update device status: {}", e);
                            }
                        }
                    }
                    _ => {} // Default case for all other status values
                }
            }
            Err(e) => {
                error!("Failed to parse device status: {}", e);
            }
        }
    }
}


