use std::sync::Arc;
use once_cell::sync::Lazy;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use crate::repository::Patient;

pub static DB_POOL: Lazy<Arc<SqlitePool>> = Lazy::new(|| {
    Arc::new(SqlitePoolOptions::new()
        .max_connections(5)
        .connect_lazy("sqlite::memory:")
        .unwrap())
});

pub async fn init_db() {
    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS infusion (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            gender TEXT NOT NULL,
            age INTEGER NOT NULL,
            bed_no TEXT NOT NULL,
            device_id TEXT NOT NULL
        )"
    )
    .execute(DB_POOL.as_ref())
    .await {
        Ok(_) => println!("infusion表创建成功"),
        Err(e) => println!("infusion表创建失败: {}", e)
    }

    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS device (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id VARCHAR(255) NOT NULL,
            status INTEGER NOT NULL default 0
        )"
    )
    .execute(DB_POOL.as_ref())
    .await {
        Ok(_) => println!("device表创建成功"),
        Err(e) => println!("device表创建失败: {}", e)
    }

    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS bed (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            bed_no VARCHAR(255) NOT NULL
        )"
    )
    .execute(DB_POOL.as_ref())
    .await {
        Ok(_) => println!("bed表创建成功"),
        Err(e) => println!("bed表创建失败: {}", e)
    }

    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS patient (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            patient_no VARCHAR(255) NOT NULL,
            name VARCHAR(255) NOT NULL,
            gender VARCHAR(10) NOT NULL,
            age INTEGER NOT NULL,
            bed_no VARCHAR(255) NOT NULL
        )"
    )
    .execute(DB_POOL.as_ref())
    .await {
        Ok(_) => println!("patient表创建成功"),
        Err(e) => println!("patient表创建失败: {}", e)
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
        Ok(_) => println!("drug表创建成功"),
        Err(e) => println!("drug表创建失败: {}", e)
    }
}

pub fn get_db() -> Arc<SqlitePool> {
    DB_POOL.clone()
}



