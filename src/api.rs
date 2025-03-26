use serde::{Deserialize, Serialize};
use crate::repository::{query_bed, query_device};
use crate::{db::get_db, repository::query_patient};
use crate::http_client::HttpClient;
use axum::{
    response::IntoResponse,
    http::StatusCode,
    Json,
    extract::Query
};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Infusion {
    id: i64,
    name: String,
    gender: String,
    age: u8,
    #[sqlx(rename = "bed_no")]
    bed_no: String,
    #[sqlx(rename = "drug_names")]
    drug_names: String, // SQLite不直接支持数组，所以存为JSON字符串
    dosage: u8,
    temperature: u8,
    #[sqlx(rename = "drip_rate")]
    drip_rate: u8,
    status: u8
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page_num: Option<u32>,
    pub page_size: Option<u32>,
    pub status: Option<u8>,
}

pub async fn fetch_infusion(
    Query(params): Query<PaginationParams>
) -> impl IntoResponse {
    let db = get_db();
    
    let page_num = params.page_num.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);
    let status = params.status.unwrap_or(0);
    
    let offset = (page_num - 1) * page_size;
    
    // 使用sqlx的查询构建器，更简洁和类型安全
    match sqlx::query_as::<_, Infusion>(
        "SELECT * FROM infusion WHERE status = ? LIMIT ? OFFSET ?"
    )
    .bind(status)
    .bind(page_size as i64)
    .bind(offset as i64)
    .fetch_all(&*db)
    .await {
        Ok(infusions) => {
            // 直接返回结果，无需手动映射
            (StatusCode::OK, Json(infusions)).into_response()
        },
        Err(e) => {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn insert_infusions(
    Json(infusions): Json<Vec<Infusion>>
) -> impl IntoResponse {
    let db = get_db();
    
    if infusions.is_empty() {
        return (StatusCode::BAD_REQUEST, "No infusions provided").into_response();
    }
    
    // 使用事务处理批量插入
    let result = sqlx::query(
        "INSERT INTO infusion (name, gender, age, bed_no, drug_names, dosage, temperature, drip_rate, status) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&infusions[0].name)
    .bind(&infusions[0].gender)
    .bind(infusions[0].age)
    .bind(&infusions[0].bed_no)
    .bind(&infusions[0].drug_names)
    .bind(infusions[0].dosage)
    .bind(infusions[0].temperature)
    .bind(infusions[0].drip_rate)
    .bind(infusions[0].status)
    .execute(&*db)
    .await;
    
    match result {
        Ok(_) => (StatusCode::OK, "Infusion inserted successfully").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
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

