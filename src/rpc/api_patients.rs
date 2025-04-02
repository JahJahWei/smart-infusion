use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{http_client::HttpClient, repository::{insert_drugs, insert_patients, Drug, Patient}};


#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ApiPatient {
    patient_no: String,
    patient_name: String,
    gender: String,
    age: u16,
    drug_list: Vec<ApiPatientDrugList>,
    bed_no: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ApiPatientDrugList {
    drug_name: String,
    dosage: u16,
    drip_rate: u16,
}

impl From<ApiPatient> for Patient {
    fn from(patient: ApiPatient) -> Self {
        Patient::new(patient.patient_no, patient.patient_name, patient.gender, patient.age, patient.bed_no, None)
    }
}

impl From<ApiPatientDrugList> for Drug {
    fn from(drug: ApiPatientDrugList) -> Self {
        Drug::new(drug.drug_name, drug.dosage, drug.drip_rate)
    }
}

const API_PATIENTS_ENDPOINT: &str = "/api/patients";

pub async fn fetch_and_store_patients(http_client: &HttpClient) -> Result<()> {
    let api_patients = http_client.fetch_api_data::<Vec<ApiPatient>>(API_PATIENTS_ENDPOINT).await?;

    let patients: Vec<Patient> = api_patients.clone()
        .into_iter()
        .map(Into::into)
        .collect();
    insert_patients(patients).await?;

    let mut all_drugs = Vec::new();
    for patient in api_patients {
        for drug in &patient.drug_list {
            let mut drug_obj = Drug::new(drug.drug_name.clone(), drug.dosage, drug.drip_rate);
            drug_obj.set_patient_no(patient.patient_no.clone());
            all_drugs.push(drug_obj);
        }
    }
    insert_drugs(all_drugs).await?;

    Ok(())
}

