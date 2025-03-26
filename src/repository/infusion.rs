use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use crate::{db::get_db, mq::{publish_alarm, Alarm}};

use super::Drug;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Infusion {
    id: Option<i64>,
    pub name: String,
    pub gender: String,
    pub age: u16,
    pub bed_no: String,
    pub device_id: String,
    
}

impl Infusion {
    pub fn new(name: String, gender: String, age: u16, bed_no: String, device_id: String) -> Self {
        Self { id: None, name, gender, age, bed_no, device_id }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct InfusionWithDetail {
    pub id: Option<i64>,
    pub name: String,
    pub gender: String,
    pub age: u16,
    pub bed_no: String,
    pub device_id: String,
    pub drugs: Vec<Drug>,
}

impl InfusionWithDetail {
    pub fn new(infusion: Infusion, drugs: Vec<Drug>) -> Self {
        Self { id: infusion.id, name: infusion.name, gender: infusion.gender, age: infusion.age, bed_no: infusion.bed_no, device_id: infusion.device_id, drugs }
    }
}

pub async fn insert_infusion(infusion: Infusion) -> Result<(), sqlx::Error> {
    println!("infusion inserted");

    let mut tx = get_db().begin().await?;

    sqlx::query("INSERT INTO infusion (name, gender, age, bed_no, device_id) VALUES (?, ?, ?, ?, ?)")
        .bind(infusion.name.clone())
        .bind(infusion.gender.clone())
        .bind(infusion.age)
        .bind(infusion.bed_no.clone())
        .bind(infusion.device_id.clone())
        .execute(&mut *tx)
        .await?;

    println!("infusion inserted");

    sqlx::query("UPDATE device SET status = 2 WHERE device_id = ?")
        .bind(infusion.device_id.clone())
        .execute(&mut *tx)
        .await?;
    println!("device status updated");

    tx.commit().await?;

    Ok(())
}

pub async fn query_infusions() -> Result<Vec<Infusion>, sqlx::Error> {
    let db = get_db();

    let infusions = sqlx::query_as::<_, Infusion>("SELECT * FROM infusion")
        .fetch_all(db.as_ref())
        .await?;

    Ok(infusions)
}
