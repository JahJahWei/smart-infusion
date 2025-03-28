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

pub struct TurnOffDeviceCmd {
    first_controll_num: i32,
    second_controll_num: i32,
    device_id: i32,
    data: i32,
}

impl TurnOffDeviceCmd {
    pub fn new(device_id: u8) -> Self {
        Self { 
            first_controll_num: -3, 
            second_controll_num: -35, 
            device_id: device_id as i32, 
            data: 124,
        }
    }
}

pub struct SetUpDripRateCmd {
    first_controll_num: i32,
    second_controll_num: i32,
    fourth_controll_num: i32,
    fifth_controll_num: i32,
    sixth_controll_num: i32,
    seventh_controll_num: i32,
    eighth_controll_num: i32,
    device_id: i32,
    data: i32,
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
            device_id: device_id as i32,
            data: data as i32,
        }
    }
}

pub struct StartOrStopDripCmd {
    first_controll_num: i32,
    second_controll_num: i32,
    device_id: i32,
    data: i32,
}

impl StartOrStopDripCmd {
    pub fn new(device_id: u8, data: i32) -> Self {
        Self { 
            first_controll_num: -3, 
            second_controll_num: -35, 
            device_id: device_id as i32, 
            data,
        }
    }
}

pub struct ModifyPresetAmountCmd {
    first_controll_num: i32,
    second_controll_num: i32,
    fourth_controll_num: i32,
    fifth_controll_num: i32,
    sixth_controll_num: i32,
    seventh_controll_num: i32,
    device_id: i32,
    data: i32,
}

impl ModifyPresetAmountCmd {
    pub fn new(device_id: u8, data: u16) -> Self {
        Self { 
            first_controll_num: -3, 
            second_controll_num: -35, 
            fourth_controll_num: -2, 
            fifth_controll_num: 8, 
            sixth_controll_num: 0,
            seventh_controll_num: 2,
            device_id: device_id as i32,
            data: data as i32,
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

pub async fn publish_turn_off_device_cmd(cmd: TurnOffDeviceCmd) {
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
        cmd.device_id as u8,
        cmd.data as u8,
    ];

    let xor = do_xor(&content);

    let payload = vec![
        &content[..],
        &[xor],
    ].concat();

    let locked_manager = manager.lock().await;
    if let Err(e) = locked_manager.publish("amq.topic", "controll_device", payload).await {
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
        cmd.device_id as u8,
        cmd.fourth_controll_num as u8,
        cmd.fifth_controll_num as u8,
        cmd.sixth_controll_num as u8,
        cmd.seventh_controll_num as u8,
        cmd.eighth_controll_num as u8,
        cmd.data as u8,
    ];

    let xor = do_xor(&content);

    let payload = vec![
        &content[..],
        &[xor],
    ].concat();

    let locked_manager = manager.lock().await;
    if let Err(e) = locked_manager.publish("amq.topic", "controll_device", payload).await {
        error!("Failed to publish controll device cmd: {}", e);
    }
}

pub async fn publish_start_or_stop_drip_cmd(cmd: StartOrStopDripCmd) {
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
        cmd.device_id as u8,
        cmd.data as u8,
    ];

    let xor = do_xor(&content);

    let payload = vec![
        &content[..],
        &[xor],
    ].concat();

    let locked_manager = manager.lock().await;
    if let Err(e) = locked_manager.publish("amq.topic", "controll_device", payload).await {
        error!("Failed to publish controll device cmd: {}", e);
    }
}

pub async fn publish_modify_preset_amount(cmd: ModifyPresetAmountCmd) {
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
        cmd.device_id as u8,
        cmd.fourth_controll_num as u8,
        cmd.fifth_controll_num as u8,
        cmd.sixth_controll_num as u8,
        cmd.seventh_controll_num as u8,
        (cmd.data / 256) as u8,
        (cmd.data % 256) as u8,
    ];

    let xor = do_xor(&content);

    let payload = vec![
        &content[..],
        &[xor],
    ].concat();

    let locked_manager = manager.lock().await;
    if let Err(e) = locked_manager.publish("amq.topic", "controll_device", payload).await {
        error!("Failed to publish controll device cmd: {}", e);
    }
}

fn do_xor(data: &[u8]) -> u8 {
    data.iter().fold(0, |acc, x| acc ^ x)
}

//开始：-22， 停止：-17
