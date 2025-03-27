use serde::{Deserialize, Serialize};
use tracing::{info, error, Level};

use super::amqp::get_amqp_manager;

#[derive(Debug, Serialize, Deserialize)]
pub struct Alarm {
    device_id: u8,
    status: u8,
}

impl Alarm {
    pub fn new(device_id: u8, status: u8) -> Self {
        Self { device_id, status }
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

pub struct TurnOffOrStopDeviceCmd {
    first_controll_num: i8,
    second_controll_num: i8,
    device_id: u8,
    data: u8,
}

impl TurnOffOrStopDeviceCmd {
    pub fn new(device_id: u8, data: u8) -> Self {
        Self { 
            first_controll_num: -3, 
            second_controll_num: -35, 
            device_id, 
            data
        }
    }
}

pub struct SetUpDripRateCmd {
    first_controll_num: i8,
    second_controll_num: i8,
    fourth_controll_num: i8,
    fifth_controll_num: i8,
    sixth_controll_num: i8,
    seventh_controll_num: i8,
    eighth_controll_num: i8,
    device_id: u8,
    data: u8,
}

impl SetUpDripRateCmd {
    pub fn new(device_id: u8, data: u8) -> Self {
        Self { 
            first_controll_num: -3, 
            second_controll_num: -35, 
            fourth_controll_num: -2, 
            fifth_controll_num: 4, 
            sixth_controll_num: 0,
            seventh_controll_num: 2,
            eighth_controll_num: 0,
            device_id,
            data
        }
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

pub async fn publish_turn_off_or_stop_device_cmd(cmd: TurnOffOrStopDeviceCmd) {
    let manager = match get_amqp_manager() {
        Some(manager) => manager,
        None => {
            error!("AmqpManager 未初始化");
            return;
        }
    };

    let content = vec![
        cmd.first_controll_num as u8,
        cmd.second_controll_num as u8,
        cmd.device_id,
        cmd.data,
    ];

    let locked_manager = manager.lock().await;
    if let Err(e) = locked_manager.publish("amq.topic", "controll_device", content).await {
        error!("Failed to publish controll device cmd: {}", e);
    }
}

pub async fn publish_drip_rate(cmd: SetUpDripRateCmd) {
    let manager = match get_amqp_manager() {
        Some(manager) => manager,
        None => {
            error!("AmqpManager 未初始化");
            return;
        }
    };

    let content = vec![
        cmd.first_controll_num as u8,
        cmd.second_controll_num as u8,
        cmd.device_id,
        cmd.fourth_controll_num as u8,
        cmd.fifth_controll_num as u8,
        cmd.sixth_controll_num as u8,
        cmd.seventh_controll_num as u8,
        cmd.eighth_controll_num as u8,
        cmd.data,
    ];

    let locked_manager = manager.lock().await;
    if let Err(e) = locked_manager.publish("amq.topic", "controll_device", content).await {
        error!("Failed to publish controll device cmd: {}", e);
    }
}

