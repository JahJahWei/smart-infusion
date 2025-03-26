use serde::{Deserialize, Serialize};
use crate::repository::{query_bed, query_device, query_infusions};
use crate::{db::get_db, repository::query_patient};
use crate::http_client::HttpClient;
use axum::{
    response::IntoResponse,
    http::StatusCode,
    Json,
    extract::Query
};

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page_num: Option<u32>,
    pub page_size: Option<u32>,
    pub status: Option<u8>,
}


pub async fn fetch_external_data() -> impl IntoResponse {
    let http_client = HttpClient::new("http://172.16.80.253:1024/".to_string());
    
    match http_client.fetch_and_store_patients().await {
        Ok(_) => {
            let response = format!("成功从API获取并存储患者数据");
            (StatusCode::OK, response).into_response()
        },
        Err(e) => {
            let error_msg = format!("获取API数据失败: {}", e);
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

pub async fn sync_devices() -> impl IntoResponse {
    let http_client = HttpClient::new("http://172.16.80.253:1024/".to_string());
    match http_client.fetch_and_store_devices().await {
        Ok(devices) => {
            (StatusCode::OK, Json(devices)).into_response()
        }
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn fetch_devices() -> impl IntoResponse {
    match query_device().await {
        Ok(devices) => (StatusCode::OK, Json(devices)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
    }
}

pub async fn sync_beds() -> impl IntoResponse {
    let http_client = HttpClient::new("http://172.16.80.253:1024/".to_string());
    match http_client.fetch_and_store_beds().await {
        Ok(beds) => (StatusCode::OK, Json(beds)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
    }
}

pub async fn fetch_beds() -> impl IntoResponse {
    match query_bed().await {
        Ok(beds) => (StatusCode::OK, Json(beds)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
    }
}

pub async fn fetch_infusions() -> impl IntoResponse {
    match query_infusions().await {
        Ok(infusions) => (StatusCode::OK, Json(infusions)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
    }
}

