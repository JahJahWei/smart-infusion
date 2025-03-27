use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, QueryBuilder, Sqlite};
use crate::{db::get_db, mq::{publish_alarm, Alarm}};

use super::Patient;


#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Device {
    id: Option<i64>,
    device_id: u8,
    mac: Option<String>,
    status: Option<u8>, //0: 关机，1：开机
    drip_value: Option<u8>,
    preset_amount: Option<u16>,
    cumulative_amount: Option<u16>,
    tem_value: Option<u8>,
    tem_gear_value: Option<u8>,
    power_state: Option<u8>,
}

impl Device {
    pub fn new(device_id: u8, mac: Option<String>) -> Self {
        Self { id: None, device_id, mac, status: None, drip_value: None, preset_amount: None, cumulative_amount: None, tem_value: None, tem_gear_value: None, power_state: None }
    }

    pub fn get_status(&self) -> Option<u8> {
        self.status
    }
}

pub async fn query_device() -> Result<Vec<Device>, sqlx::Error> {
    let db = get_db();

    let devices = sqlx::query_as::<_, Device>("SELECT * FROM device")
        .fetch_all(db.as_ref())
        .await?;

    Ok(devices)
}

pub async fn insert_devices(devices: Vec<Device>) -> Result<(), sqlx::Error> {
    if devices.is_empty() {
        return Ok(());
    }

    let mut tx = get_db().begin().await?;

    for device in devices {
        sqlx::query("INSERT INTO device (device_id, mac) VALUES (?, ?)")
        .bind(device.device_id)
        .bind(device.mac.clone())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn is_device_exist(device_id: String) -> Result<bool, sqlx::Error> {
    let db = get_db();

    let device = sqlx::query_as::<_, Device>("SELECT * FROM device WHERE device_id = ?")
        .bind(device_id)
        .fetch_optional(db.as_ref())
        .await?;

    Ok(device.is_some())
}

pub async fn update_device_status(device_id: u8, status: u8) -> Result<(), sqlx::Error> {
    let db = get_db();

    sqlx::query("UPDATE device SET status = ? WHERE device_id = ?")
        .bind(status)
        .bind(device_id)
        .execute(db.as_ref())
        .await?;

    let patient = sqlx::query_as::<_, Patient>("select * from patient where device_id = ?")
        .bind(device_id)
        .fetch_optional(db.as_ref())
        .await?;

    if patient.is_none() {
        publish_alarm(Alarm::new(device_id, 0)).await;
    }

    Ok(())
}

pub async fn fetch_device_by_device_id(device_id: u8) -> Result<Option<Device>, sqlx::Error> {
    let db = get_db();

    let device = sqlx::query_as::<_, Device>("SELECT * FROM device WHERE device_id = ?")
        .bind(device_id)
        .fetch_optional(db.as_ref())
        .await?;

    Ok(device)
}

