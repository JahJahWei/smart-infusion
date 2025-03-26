use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, QueryBuilder, Sqlite};
use crate::db::get_db;


#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Device {
    id: Option<i64>,
    device_id: String,
}

impl Device {
    pub fn new(device_id: String) -> Self {
        Self { id: None, device_id }
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
    // Return early if there are no devices to insert
    if devices.is_empty() {
        return Ok(());
    }

    let mut tx = get_db().begin().await?;

    for device in devices {
        sqlx::query("INSERT INTO device (device_id) VALUES (?)")
        .bind(device.device_id.clone())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}



