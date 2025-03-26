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

    let db = get_db();

    let mut query_builder = QueryBuilder::<Sqlite>::new("INSERT INTO device (device_id) VALUES ");
    query_builder.push_values(devices, |mut query_builder, device| {
        query_builder.push_bind(device.device_id);
    });

    let query = query_builder.build();

    query.execute(db.as_ref()).await?;

    Ok(())
}



