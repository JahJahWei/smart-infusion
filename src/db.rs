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
        Ok(_) => println!("表创建成功"),
        Err(e) => println!("表创建失败: {}", e)
    }

    DB_POOL.set(Arc::new(pool)).expect("Failed to set DB_POOL");
}

pub fn get_db() -> Arc<SqlitePool> {
    DB_POOL.get().expect("Database not initialized").clone()
}



