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
    // Return early if there are no drugs to insert
    if drugs.is_empty() {
        return Ok(());
    }

    let db = get_db();

    let mut query_builder = QueryBuilder::<Sqlite>::new("INSERT INTO drug (drug_name, dosage, drip_rate, patient_no) VALUES ");

    query_builder.push_values(drugs, |mut query_builder, drug| {
        query_builder.push_bind(drug.drug_name);
        query_builder.push_bind(drug.dosage);
        query_builder.push_bind(drug.drip_rate);
        query_builder.push_bind(drug.patient_no);
    });

    let query = query_builder.build();
    query.execute(db.as_ref()).await?;

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
