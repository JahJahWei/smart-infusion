use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow};
use crate::{db::get_db, mq::{publish_alarm, Alarm, DeviceData, DeviceStatus}};

use super::Drug;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Patient {
    id: Option<i64>,
    pub patient_no: String,
    pub name: String,
    pub gender: String,
    pub age: u16,
    pub bed_no: String,
    pub device_id: Option<u16>,
    pub current_drug_id: Option<String>,
    pub current_drop_rate: Option<u16>,
    pub current_temperature: Option<u16>,
    pub total_drop: Option<u16>,
    pub status: Option<u16>, //0: 未绑定，1: 已绑定（开机中）， 2：输液中， 3： 输液完成， 4： 输液暂停， 5： 待输液， 6： 输完液以解绑（关机中）
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatientDetail {
    id: Option<i64>,
    patient_no: String,
    name: String,
    gender: String,
    age: u16,
    bed_no: String,
    drugs: Option<Vec<DrugDetail>>,
    device_id: Option<u16>,
    current_drug_id: Option<String>,
    current_drop_rate: Option<u16>,
    current_temperature: Option<u16>,
    total_drop: Option<u16>,
    status: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrugDetail {
    id: Option<i64>,
    drug_name: Option<String>,
    dosage: Option<u16>,
}

impl Patient {
    pub fn new(patient_no: String, name: String, gender: String, age: u16, bed_no: String, device_id: Option<u16>) -> Self {
        Self {
             id: None, patient_no, name, gender, age, bed_no, device_id, current_drug_id: None,
             current_drop_rate: None, current_temperature: None, total_drop: None, status: None 
        }
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

pub async fn update_patient_by_device_id(device_data: DeviceData) -> Result<(), sqlx::Error> {
    let db = get_db();

    sqlx::query("UPDATE patient SET current_drop_rate = ?, current_temperature = ?, total_drop = ?, status = ? WHERE device_id = ?")
    .bind(device_data.drip_value)
    .bind(device_data.tem_value)
    .bind(device_data.cumulative_amount)
    .bind(device_data.status)
    .bind(device_data.device_id)
    .execute(db.as_ref())
    .await?;

    Ok(())
}

pub async fn update_patient_device_id(patient_no: String, device_id: u8) -> Result<(), sqlx::Error> {
    let db = get_db();

    sqlx::query("UPDATE patient SET device_id = ?, status = 1 WHERE patient_no = ?")
    .bind(device_id)
    .bind(patient_no)
    .execute(db.as_ref())
    .await?;

    Ok(())
}

pub async fn fetch_all_patient_page(page: u16, page_size: u16, status: Option<u16>, name: Option<String>) -> Result<Vec<PatientDetail>, sqlx::Error> {
    let db = get_db();

    let offset = (page - 1) * page_size;

    let mut query = String::from(
        "SELECT 
            id,
            patient_no,
            name,
            gender,
            age,
            bed_no,
            device_id,
            current_drug_id,
            current_drop_rate,
            current_temperature,
            total_drop,
            status
        FROM patient
        WHERE 1=1"
    );

    if status.is_some() {
        query.push_str(" AND status = ?");
    }

    if name.is_some() {
        query.push_str(" AND name LIKE ?");
    }

    query.push_str(" LIMIT ? OFFSET ?");

    let mut query_builder = sqlx::query_as::<_, Patient>(&query);

    if let Some(s) = status {
        query_builder = query_builder.bind(s);
    }

    if let Some(n) = name.clone() {
        query_builder = query_builder.bind(format!("%{}%", n));
    }

    query_builder = query_builder.bind(page_size).bind(offset);

    let patient_details = query_builder.fetch_all(db.as_ref()).await?;

    let mut result = Vec::<PatientDetail>::new();

    for patient in patient_details {
        let drugs = sqlx::query_as::<_, Drug>("SELECT * FROM drug WHERE patient_no = ?")
        .bind(patient.patient_no.clone())
        .fetch_all(db.as_ref())
        .await?;

        result.push(PatientDetail {
            id: patient.id,
            patient_no: patient.patient_no,
            name: patient.name,
            gender: patient.gender,
            age: patient.age,
            bed_no: patient.bed_no,
            drugs: Some(drugs.into_iter().map(|drug| DrugDetail {
                id: drug.id,
                drug_name: Some(drug.drug_name),
                dosage: Some(drug.dosage),
            }).collect()),
            status: patient.status,
            total_drop: patient.total_drop,
            device_id: patient.device_id,
            current_drug_id: patient.current_drug_id.clone(),
            current_drop_rate: patient.current_drop_rate,
            current_temperature: patient.current_temperature,
        });
    }

    Ok(result)
}


