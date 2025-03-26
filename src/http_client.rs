use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use std::time::Duration;
use crate::{api, db::get_db, repository::{insert_beds, insert_devices, insert_drugs, insert_patients, Bed, Device, Drug, Patient}};
use tracing::{info};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiPatient {
    patient_no: String,
    patient_name: String,
    gender: String,
    age: u16,
    drug_list: Vec<ApiPatientDrugList>,
    bed_no: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiPatientDrugList {
    drug_name: String,
    dosage: u16,
    drip_rate: u16,
}

impl From<ApiPatient> for Patient {
    fn from(patient: ApiPatient) -> Self {
        Patient::new(patient.patient_no, patient.patient_name, patient.gender, patient.age, patient.bed_no)
    }
}

impl From<ApiPatientDrugList> for Drug {
    fn from(drug: ApiPatientDrugList) -> Self {
        Drug::new(drug.drug_name, drug.dosage, drug.drip_rate)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiBed {
    bed_no: String,
}
impl From<ApiBed> for Bed {
    fn from(bed: ApiBed) -> Self {
        Bed::new(bed.bed_no)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiDevice {
    device_id: String,
}
impl From<ApiDevice> for Device {
    fn from(device: ApiDevice) -> Self {
        Device::new(device.device_id)
    }
}

pub struct HttpClient {
    client: reqwest::Client,
    api_base_url: String,
}

impl HttpClient {
    pub fn new(api_base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_base_url,
        }
    }

    pub async fn fetch_and_store_patients(&self) -> Result<()> {

        let url = format!("{}patientInfoDashboard/queryList", self.api_base_url);
        info!("Fetching patients data from API {}", url);

        let response = self.client.get(&url)
            .send()
            .await
            .context("Failed to send request to API")?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "API request failed with status: {}", 
                response.status()
            ));
        }

        let api_response: Vec<ApiPatient> = response.json()
            .await
            .context("Failed to parse API response")?;
        info!("Patients data parsed successfully");
        
        
        let patients: Vec<Patient> = api_response.iter().map(|p| {
            Patient::new(
                p.patient_no.clone(), 
                p.patient_name.clone(), 
                p.gender.clone(), 
                p.age, 
                p.bed_no.clone()
            )
        }).collect();
        insert_patients(patients).await?;
        info!("Patients data stored successfully");

        let mut all_drugs = Vec::new();
        for patient in &api_response {
            for drug in &patient.drug_list {
                let mut drug_obj = Drug::new(
                    drug.drug_name.clone(), 
                    drug.dosage, 
                    drug.drip_rate
                );
                drug_obj.set_patient_no(patient.patient_no.clone());
                all_drugs.push(drug_obj);
            }
        }
        
        insert_drugs(all_drugs).await?;
        info!("Drugs data stored successfully");

        Ok(())
    }

    pub async fn fetch_and_store_beds(&self) -> Result<()> {
        info!("Fetching beds data from API...");

        let url = format!("{}/beds", self.api_base_url);
        
        let response = self.client.get(&url)
            .send()
            .await
            .context("Failed to send request to API")?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "API request failed with status: {}", 
                response.status()
            ));
        }
        
        let beds: Vec<ApiBed> = response.json()
            .await
            .context("Failed to parse API response")?;
        info!("Beds data parsed successfully");

        let beds = beds.into_iter().map(|bed| bed.into()).collect();
        insert_beds(beds).await?;
        info!("Beds data stored successfully");
        
        Ok(())
    }

    pub async fn fetch_and_store_devices(&self) -> Result<()> {
        info!("Fetching devices data from API...");

        let url = format!("{}/devices", self.api_base_url);
        
        let response = self.client.get(&url)
            .send()
            .await
            .context("Failed to send request to API")?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "API request failed with status: {}", 
                response.status()
            ));
        }
        
        let devices: Vec<ApiDevice> = response.json()
            .await
            .context("Failed to parse API response")?;
        info!("Devices data parsed successfully");
        
        let devices = devices.into_iter().map(|device| device.into()).collect();
        insert_devices(devices).await?;
        info!("Devices data stored successfully");
        
        Ok(())
    }
    
    
    pub fn set_api_base_url(&mut self, api_base_url: String) {
        self.api_base_url = api_base_url;
    }
}
