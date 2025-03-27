use amqprs::{channel::Channel, consumer::AsyncConsumer, BasicProperties, Deliver};
use axum::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{info, error, Level};
use crate::repository::{fetch_bed_by_bed_no, fetch_device_by_device_id, fetch_drug_by_patient_no, fetch_patient_by_bed_no, insert_infusion, update_device_status, update_patient_by_device_id, update_patient_device_id, Infusion};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    pub device_id: u8,
    pub bed_no: String, 
}

pub struct BindingConsumer;

#[async_trait]
impl AsyncConsumer for BindingConsumer {
    async fn consume(&mut self, channel: &Channel, deliver: Deliver, basic_properties: BasicProperties, content: Vec<u8>) {
        let msg = String::from_utf8_lossy(&content);
        
        let binding = match serde_json::from_str::<Binding>(&msg) {
            Ok(binding) => binding,
            Err(e) => {
                error!("Failed to parse binding: {}", e);
                return;
            }
        };
        
        let device = match fetch_device_by_device_id(binding.device_id).await {
            Ok(Some(device)) => device,
            Ok(None) => {
                error!("Device not found");
                return;
            },
            Err(e) => {
                error!("Failed to fetch device: {}", e);
                return;
            }
        };

        let patient = match fetch_patient_by_bed_no(binding.bed_no.clone()).await {
            Ok(Some(patient)) => patient,
            Ok(None) => {
                error!("Patient not found");
                return;
            },
            Err(e) => {
                error!("Failed to fetch patient: {}", e);
                return;
            }
        };

        if device.get_status().is_some() && device.get_status() == Some(1) {
            match update_patient_device_id(patient.patient_no.clone(), binding.device_id).await {
                Ok(_) => {
                    info!("Patient device id updated");
                }
                Err(e) => {
                    error!("Failed to update patient device id: {}", e);
                }
            };
        }
    }
}
