use crate::db::get_db;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, QueryBuilder, Sqlite};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Bed {
    id: Option<i64>,
    bed_no: String,
}

impl Bed {
    pub fn new(bed_no: String) -> Self {
        Self { id: None, bed_no }
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
        sqlx::query("INSERT INTO bed (bed_no) VALUES (?)")
        .bind(bed.bed_no.clone())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

