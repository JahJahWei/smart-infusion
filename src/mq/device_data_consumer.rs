use amqprs::consumer::AsyncConsumer;
use amqprs::channel::Channel;
use amqprs::{BasicProperties, Deliver};
use axum::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{info, error, Level};

use crate::repository::update_patient_by_device_id;

enum DeviceStatus {
    OFF = 0,
    ON = 1,
    WAITINFUSION = 2,
    INFUSION = 3,
    DONE = 4,
    STOP = 5
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeviceData {
    pub device_id: u8, //设备编号
    pub drip_value: u8, //滴速
    pub preset_amount: u16, //预设量
    pub cumulative_amount: u16, //累计量
    pub tem_value: u8, //温度
    pub tem_gear_value: u8, //温度档位
    pub status: u8, //状态
    pub power_state: u8, //电源状态
}

impl DeviceData {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 28 {
            return Err(format!("the length of the data is less than 28, it is {}", bytes.len()));
        }
        
        let data = DeviceData {
            device_id: bytes[2],
            drip_value: bytes[12],
            preset_amount: bytes[13] as u16 * 256 + bytes[14] as u16,
            cumulative_amount: bytes[15] as u16 * 256 + bytes[16] as u16,
            tem_gear_value: bytes[17],
            tem_value: bytes[18],
            status: match bytes[22] {
                17 => DeviceStatus::ON as u8,
                85 => match bytes[26] {
                    115 => DeviceStatus::DONE as u8,
                    _ => DeviceStatus::STOP as u8,
                },
                34 => DeviceStatus::OFF as u8,
                _ => DeviceStatus::OFF as u8,
            },
            power_state: bytes[23],
        };
        
        Ok(data)
    }
}

pub struct DeviceDataConsumer;

#[async_trait]
impl AsyncConsumer for DeviceDataConsumer {

    async fn consume(&mut self, channel: &Channel, deliver: Deliver, basic_properties: BasicProperties, content: Vec<u8>) {
        println!("收到设备数据消息，长度: {}", content.len());
        
        match DeviceData::from_bytes(&content) {
            Ok(device_data) => {
                match update_patient_by_device_id(device_data).await {
                    Ok(_) => {},
                    Err(e) => error!("update patient data failed: {}", e),
                }
            },
            Err(e) => {
                error!("failed to parse device data: {}", e);
            }
        }
    }
}

