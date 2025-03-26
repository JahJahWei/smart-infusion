use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow};
use crate::db::{get_db};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Patient {
    id: Option<i64>,
    pub patient_no: String,
    pub name: String,
    pub gender: String,
    pub age: u16,
    pub bed_no: String,
}

impl Patient {
    pub fn new(patient_no: String, name: String, gender: String, age: u16, bed_no: String) -> Self {
        Self { id: None, patient_no, name, gender, age, bed_no }
    }
}

pub async fn query_patient() -> Result<Vec<Patient>, sqlx::Error> {
    let db = get_db();

    let patients = sqlx::query_as::<_, Patient>("SELECT * FROM patient")
        .fetch_all(db.as_ref())
        .await?;

    Ok(patients)
}

pub async fn insert_patients(patients: Vec<Patient>) -> Result<(), sqlx::Error> {
    if patients.is_empty() {
        return Ok(());
    }
    
    let mut tx = get_db().begin().await?;

    for patient in patients {
        sqlx::query("INSERT INTO patient (patient_no, name, gender, age, bed_no) VALUES (?, ?, ?, ?, ?)")
        .bind(patient.patient_no.clone())
        .bind(patient.name.clone())
        .bind(patient.gender.clone())
        .bind(patient.age)
        .bind(patient.bed_no.clone())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn fetch_patient_by_bed_no(bed_no: String) -> Result<Option<Patient>, sqlx::Error> {
    let db = get_db();

    let patient = sqlx::query_as::<_, Patient>("SELECT * FROM patient WHERE bed_no = ?")
        .bind(bed_no)
        .fetch_optional(db.as_ref())
        .await?;

    Ok(patient)
}



