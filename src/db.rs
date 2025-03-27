use std::sync::Arc;
use once_cell::sync::Lazy;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use tracing::{info, error};

pub static DB_POOL: Lazy<Arc<SqlitePool>> = Lazy::new(|| {
    Arc::new(SqlitePoolOptions::new()
        .max_connections(5)
        .connect_lazy("sqlite::memory:")
        .unwrap())
});

pub async fn init_db() {
    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS device (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id INTEGER NOT NULL,
            mac VARCHAR(255) NULL,
            status INTEGER NULL,
            drip_value INTEGER NULL,
            preset_amount INTEGER NULL,
            cumulative_amount INTEGER NULL,
            tem_value INTEGER NULL,
            tem_gear_value INTEGER NULL,
            power_state INTEGER NULL
        )"
    )
    .execute(DB_POOL.as_ref())
    .await {
        Ok(_) => info!("create device table success"),
        Err(e) => error!("create device table failed: {}", e)
    }

    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS bed (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            bed_no VARCHAR(255) NOT NULL,
            mac VARCHAR(255) NULL
        )"
    )
    .execute(DB_POOL.as_ref())
    .await {
        Ok(_) => info!("create bed table success"),
        Err(e) => error!("create bed table failed: {}", e)
    }

    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS patient (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            patient_no VARCHAR(255) NOT NULL,
            name VARCHAR(255) NOT NULL,
            gender VARCHAR(10) NULL,
            age INTEGER NULL,
            bed_no VARCHAR(255) NULL,
            device_id INTEGER NULL
        )"
    )
    .execute(DB_POOL.as_ref())
    .await {
        Ok(_) => info!("create patient table success"),
        Err(e) => error!("create patient table failed: {}", e)
    }

    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS drug (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            drug_name VARCHAR(255) NOT NULL,
            dosage INTEGER NOT NULL,
            drip_rate INTEGER NOT NULL,
            patient_no VARCHAR(255) NOT NULL
        )"
    )
    .execute(DB_POOL.as_ref())
    .await {
        Ok(_) => info!("create drug table success"),
        Err(e) => error!("create drug table failed: {}", e)
    }
}

pub fn get_db() -> Arc<SqlitePool> {
    DB_POOL.clone()
}



