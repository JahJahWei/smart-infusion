use std::sync::Arc;
use once_cell::sync::OnceCell;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

static DB_POOL: OnceCell<Arc<SqlitePool>> = OnceCell::new();

pub async fn init_db() {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await
        .unwrap();

    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS infusion (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            gender TEXT NOT NULL,
            age INTEGER NOT NULL,
            bed_no TEXT NOT NULL,
            drug_names TEXT NOT NULL,
            dosage INTEGER NOT NULL,
            temperature INTEGER NOT NULL,
            drip_rate INTEGER NOT NULL,
            status INTEGER NOT NULL
        )"
    )
    .execute(&pool)
    .await {
        Ok(_) => println!("infusion表创建成功"),
        Err(e) => println!("infusion表创建失败: {}", e)
    }

    match sqlx::query(
        "CREATE TABLE IF NOT EXISTS device (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            device_id VARCHAR(255) NOT NULL
        )"
    )
    .execute(&pool)
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
    .execute(&pool)
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
    .execute(&pool)
    .await {
        Ok(_) => println!("patient表创建成功"),
        Err(e) => println!("patient表创建失败: {}", e)
    }


    DB_POOL.set(Arc::new(pool)).expect("Failed to set DB_POOL");
}

pub fn get_db() -> Arc<SqlitePool> {
    DB_POOL.get().expect("Database not initialized").clone()
}



