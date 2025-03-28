use crate::db::get_db;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Bed {
    id: Option<i64>,
    pub bed_no: String,
    pub mac: Option<String>,
}

impl Bed {
    pub fn new(bed_no: String, mac: Option<String>) -> Self {
        Self { id: None, bed_no, mac }
    }
}

pub async fn query_bed() -> Result<Vec<Bed>, sqlx::Error> {
    let db = get_db();

    let beds = sqlx::query_as::<_, Bed>("SELECT * FROM bed")
        .fetch_all(db.as_ref())
        .await?;

    Ok(beds)
}

pub async fn insert_beds(beds: Vec<Bed>) -> Result<(), sqlx::Error> {
    if beds.is_empty() {
        return Ok(());
    }

    let mut tx = get_db().begin().await?;

    for bed in beds {
        sqlx::query("INSERT INTO bed (bed_no, mac) VALUES (?, ?)")
        .bind(bed.bed_no.clone())
        .bind(bed.mac.clone())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn fetch_bed_by_bed_mac(bed_mac: String) -> Result<Option<Bed>, sqlx::Error> {
    let db = get_db();

    let bed = sqlx::query_as::<_, Bed>("SELECT * FROM bed WHERE mac = ? limit 1")
        .bind(bed_mac)
        .fetch_optional(db.as_ref())
        .await?;

    Ok(bed)
}

