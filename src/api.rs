use axum::extract::Query;
use serde::{Deserialize, Serialize};
use crate::mq::{publish_drip_rate, publish_modify_preset_amount, publish_start_or_stop_drip_cmd, publish_turn_off_device_cmd, ModifyPresetAmountCmd, SetUpDripRateCmd, StartOrStopDripCmd, TurnOffDeviceCmd};
use crate::repository::{fetch_all_patient_page, query_bed, query_device, PatientDetail};
use crate::rpc::{fetch_and_store_beds, fetch_and_store_devices, fetch_and_store_patients};
use crate::repository::query_patient;
use crate::http_client::HttpClient;
use axum::{
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use tracing::{info, error};

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse<T> {
    pub code: u32,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn new(code: u32, message: String, data: Option<T>) -> Self {
        Self { code, message, data }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyDripRate {
    pub device_id: u8,
    pub drip_rate: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnOffDevice {
    pub device_id: u8,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatientDetailParam {
    pub page_num: u16,
    pub page_size:u16,
    pub status: Option<u16>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartOrStopDrip {
    pub device_id: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifyPresetAmount {
    pub device_id: u8,
    pub preset_amount: u16,
}

pub async fn fetch_patients() -> impl IntoResponse {
    match query_patient().await {
        Ok(patients) => {
            (StatusCode::OK, Json(patients)).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn sync_remote_device_data() -> impl IntoResponse {
    let url = "http://172.16.80.253:1024/".to_string();
    let client = HttpClient::new(url, None);
    let result = fetch_and_store_devices(&client).await;
    
    if let Ok(_) = result {
        info!("success to fetch and store device data");
        (StatusCode::OK, "success to fetch and store device data").into_response()
    } else if let Err(e) = result {
        error!("failed to fetch and store device data: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("failed to fetch and store device data: {}", e)).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error").into_response()
    }
}

pub async fn sync_remote_patient_data() -> impl IntoResponse {
    let url = "http://172.16.80.253:1024/".to_string();
    let client = HttpClient::new(url, None);
    let result = fetch_and_store_patients(&client).await;
    
    if let Ok(_) = result {
        info!("success to fetch and store patient data");
        (StatusCode::OK, "success to fetch and store patient data").into_response()
    } else if let Err(e) = result {
        error!("failed to fetch and store patient data: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("failed to fetch and store patient data: {}", e)).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error").into_response()
    }
}

pub async fn sync_remote_bed_data() -> impl IntoResponse {
    let url = "http://172.16.80.253:1024/".to_string();
    let client = HttpClient::new(url, None);
    let result = fetch_and_store_beds(&client).await;
    
    if let Ok(_) = result {
        info!("success to fetch and store bed data");
        (StatusCode::OK, "success to fetch and store bed data").into_response()
    } else if let Err(e) = result {
        error!("failed to fetch and store bed data: {}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("failed to fetch and store bed data: {}", e)).into_response()
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error").into_response()
    }
}

pub async fn fetch_devices() -> impl IntoResponse {
    match query_device().await {
        Ok(devices) => (StatusCode::OK, Json(devices)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
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

    (StatusCode::OK, Json(ApiResponse::<()>::new(0, "success".to_string(), None))).into_response()
}

pub async fn turn_off_device(Json(turn_off_device): Json<TurnOffDevice>) -> impl IntoResponse {
    publish_turn_off_device_cmd(TurnOffDeviceCmd::new(turn_off_device.device_id)).await;

    (StatusCode::OK, Json(ApiResponse::<()>::new(0, "success".to_string(), None))).into_response()
}

pub async fn start_drip(Json(start_drip): Json<StartOrStopDrip>) -> impl IntoResponse {
    publish_start_or_stop_drip_cmd(StartOrStopDripCmd::new(start_drip.device_id, -22)).await;

    (StatusCode::OK, Json(ApiResponse::<()>::new(0, "success".to_string(), None))).into_response()
}

pub async fn stop_drip(Json(stop_drip): Json<StartOrStopDrip>) -> impl IntoResponse {
    publish_start_or_stop_drip_cmd(StartOrStopDripCmd::new(stop_drip.device_id, -17)).await;

    (StatusCode::OK, Json(ApiResponse::<()>::new(0, "success".to_string(), None))).into_response()
}

pub async fn modify_preset_amount(Json(modify_preset_amount): Json<ModifyPresetAmount>) -> impl IntoResponse {
    publish_modify_preset_amount(ModifyPresetAmountCmd::new(modify_preset_amount.device_id, modify_preset_amount.preset_amount)).await;

    (StatusCode::OK, Json(ApiResponse::<()>::new(0, "success".to_string(), None))).into_response()
}


pub async fn patient_detail(Query(patient_detail): Query<PatientDetailParam>) -> (StatusCode, Json<Vec<PatientDetail>>) {
    match fetch_all_patient_page(patient_detail.page_num, patient_detail.page_size, patient_detail.status, patient_detail.name).await {
        Ok(patients) => (StatusCode::OK, Json(patients)),
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::new()))
        }
    }
}

