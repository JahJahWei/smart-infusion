use axum::extract::Query;
use axum::response::Response;
use serde::{Deserialize, Serialize};
use crate::mq::{publish_drip_rate, publish_turn_off_or_stop_device_cmd, DeviceStatus, SetUpDripRateCmd, TurnOffOrStopDeviceCmd};
use crate::repository::{fetch_all_patient_page, query_bed, query_device, query_infusions, PatientDetail};
use crate::{db::get_db, repository::query_patient};
use crate::http_client::HttpClient;
use axum::{
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use tracing::{info, error};

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page_num: Option<u32>,
    pub page_size: Option<u32>,
    pub status: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct ModifyDripRate {
    pub device_id: u8,
    pub drip_rate: u8,
}

#[derive(Debug, Deserialize)]
pub struct StopDevice {
    pub device_id: u8,
}

#[derive(Debug, Deserialize)]
pub struct PatientDetailParam {
    pub page_num: u16,
    pub page_size:u16,
    pub status: Option<u16>,
    pub name: Option<String>,
}


pub async fn sync_remote_patient_data() -> impl IntoResponse {
    let http_client = HttpClient::new("http://172.16.80.253:1024/".to_string());
    
    match http_client.fetch_and_store_patients().await {
        Ok(_) => {
            info!("success to fetch and store patients data");
            let response = format!("success to fetch and store patients data");
            (StatusCode::OK, response).into_response()
        },
        Err(e) => {
            error!("failed to fetch and store patients data: {}", e);
            let error_msg = format!("failed to fetch and store patients data: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, error_msg).into_response()
        }
    }
}

pub async fn fetch_patients() -> impl IntoResponse {
    let patients = query_patient().await;
    match patients {    
        Ok(patients) => {
            (StatusCode::OK, Json(patients)).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn sync_remote_device_data() -> impl IntoResponse {
    let http_client = HttpClient::new("http://172.16.80.253:1024/".to_string());
    match http_client.fetch_and_store_devices().await {
        Ok(_) => {
            info!("success to fetch and store devices data");
            let response = format!("success to fetch and store devices data");
            (StatusCode::OK, response).into_response()
        }
        Err(e) => {
            error!("failed to fetch and store devices data: {}", e);
            let error_msg = format!("failed to fetch and store devices data: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, error_msg).into_response()
        }
    }
}

pub async fn fetch_devices() -> impl IntoResponse {
    match query_device().await {
        Ok(devices) => (StatusCode::OK, Json(devices)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
    }
}

pub async fn sync_remote_bed_data() -> impl IntoResponse {
    let http_client = HttpClient::new("http://172.16.80.253:1024/".to_string());
    match http_client.fetch_and_store_beds().await {
        Ok(_) => {
            info!("success to fetch and store beds data");
            let response = format!("success to fetch and store beds data");
            (StatusCode::OK, response).into_response()
        }
        Err(e) => {
            error!("failed to fetch and store beds data: {}", e);
            let error_msg = format!("failed to fetch and store beds data: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, error_msg).into_response()
        }
    }
}

pub async fn fetch_beds() -> impl IntoResponse {
    match query_bed().await {
        Ok(beds) => (StatusCode::OK, Json(beds)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
    }
}

pub async fn modify_drip_rate(Json(modify_drip_rate): Json<ModifyDripRate>) -> impl IntoResponse {
    publish_drip_rate(SetUpDripRateCmd::new(modify_drip_rate.device_id, modify_drip_rate.drip_rate)).await;

    (StatusCode::OK, "success").into_response()
}

pub async fn stop_device(Json(stop_device): Json<StopDevice>) -> impl IntoResponse {
    publish_turn_off_or_stop_device_cmd(TurnOffOrStopDeviceCmd::new(stop_device.device_id, 85)).await;

    (StatusCode::OK, "success").into_response()
}

pub async fn patient_detail(Query(patient_detail): Query<PatientDetailParam>) -> (StatusCode, Json<Vec<PatientDetail>>) {
    match fetch_all_patient_page(patient_detail.page_num, patient_detail.page_size, patient_detail.status, patient_detail.name).await {
        Ok(patients) => (StatusCode::OK, Json(patients)),
        Err(e) => {
            error!("Failed to fetch patient details: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
        }
    }
}

