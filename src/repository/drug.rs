use sqlx::{QueryBuilder, Sqlite, prelude::FromRow};
use serde::{Deserialize, Serialize};
use crate::db::get_db;


#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Drug {
    id: Option<i64>,
    drug_name: String,
    dosage: u16,
    drip_rate: u16,
    patient_no: Option<String>,
}

impl Drug {
    pub fn new(drug_name: String, dosage: u16, drip_rate: u16) -> Self {
        Self { id: None, drug_name, dosage, drip_rate, patient_no: None }
    }

    pub fn set_patient_no(&mut self, patient_no: String) {
        self.patient_no = Some(patient_no);
    }
}

pub async fn insert_drugs(drugs: Vec<Drug>) -> Result<(), sqlx::Error> {
    if drugs.is_empty() {
        return Ok(());
    }

    let mut tx = get_db().begin().await?;

    for drug in drugs {
        sqlx::query("INSERT INTO drug (drug_name, dosage, drip_rate, patient_no) VALUES (?, ?, ?, ?)")
        .bind(drug.drug_name.clone())
        .bind(drug.dosage)
        .bind(drug.drip_rate)
        .bind(drug.patient_no.clone())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn query_drug_by_patient_no(patient_no: String) -> Result<Vec<Drug>, sqlx::Error> {
    let db = get_db();

    let drugs = sqlx::query_as::<_, Drug>("SELECT * FROM drug WHERE patient_no = ?")
        .bind(patient_no)
        .fetch_all(db.as_ref())
        .await?;

    Ok(drugs)
}

pub async fn fetch_drug_by_patient_no(patient_no: String) -> Result<Vec<Drug>, sqlx::Error> {
    let db = get_db();

    let drugs = sqlx::query_as::<_, Drug>("SELECT * FROM drug WHERE patient_no = ?")
        .bind(patient_no)
        .fetch_all(db.as_ref())
        .await?;

    Ok(drugs)
}

