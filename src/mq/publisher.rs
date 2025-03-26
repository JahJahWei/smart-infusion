use serde::{Deserialize, Serialize};
use tracing::{info, error, Level};

use super::amqp::get_amqp_manager;

#[derive(Debug, Serialize, Deserialize)]
pub struct Alarm {
    device_id: String,
    message: String,
}

impl Alarm {
    pub fn new(device_id: String, message: String) -> Self {
        Self { device_id, message }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceStatus {
    device_id: String,
    status: u16,
}

impl DeviceStatus {
    pub fn new(device_id: String, status: u16) -> Self {
        Self { device_id, status }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DripRate {
    device_id: String,
    drip_rate: u16,
}

impl DripRate {
    pub fn new(device_id: String, drip_rate: u16) -> Self {
        Self { device_id, drip_rate }
    }
}


pub async fn publish_alarm(alarm: Alarm) {
    let manager = match get_amqp_manager() {
        Some(manager) => manager,
        None => {
            error!("AmqpManager 未初始化");
            return;
        }
    };

    let content = match serde_json::to_string(&alarm) {
        Ok(content) => content.into_bytes(),
        Err(e) => {
            error!("mq content serialize error: {}", e);
            return;
        }
    };

    let locked_manager = manager.lock().await;
    if let Err(e) = locked_manager.publish("amq.topic", "alarm", content).await {
        error!("Failed to publish alarm: {}", e);
    }
}

pub async fn publish_device_status(device_status: DeviceStatus) {
    let manager = match get_amqp_manager() {
        Some(manager) => manager,
        None => {
            error!("AmqpManager 未初始化");
            return;
        }
    };

    let content = match serde_json::to_string(&device_status) {
        Ok(content) => content.into_bytes(),
        Err(e) => {
            error!("mq content serialize error: {}", e);
            return;
        }
    };

    let locked_manager = manager.lock().await;
    if let Err(e) = locked_manager.publish("amq.topic", "device_status", content).await {
        error!("Failed to publish device status: {}", e);
    }
}

pub async fn publish_drip_rate(drip_rate: DripRate) {
    let manager = match get_amqp_manager() {
        Some(manager) => manager,
        None => {
            error!("AmqpManager 未初始化");
            return;
        }
    };

    let content = match serde_json::to_string(&drip_rate) {
        Ok(content) => content.into_bytes(),
        Err(e) => {
            error!("mq content serialize error: {}", e);
            return;
        }
    };

    let locked_manager = manager.lock().await;
    if let Err(e) = locked_manager.publish("amq.topic", "drip_rate", content).await {
        error!("Failed to publish drip rate: {}", e);
    }
}


