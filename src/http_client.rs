use serde::de::DeserializeOwned;
use anyhow::Result;
use std::time::Duration;
use std::future::Future;
use std::pin::Pin;
use crate::rpc::{fetch_and_store_beds, fetch_and_store_devices, fetch_and_store_patients};
use tracing::{info, error};

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// HTTP client for making API requests
/// 
/// This struct provides functionality for making HTTP requests to an API endpoint.
/// It maintains a connection pool and handles timeouts for requests.
/// 
/// # Fields
/// 
/// * `client` - The underlying reqwest HTTP client instance
/// * `api_base_url` - The base URL for all API requests
/// * `timeout` - The maximum duration to wait for a response before timing out, default is 10 seconds
pub struct HttpClient {
    client: reqwest::Client,
    api_base_url: String,
}

impl HttpClient {
    pub fn new(api_base_url: String, timeout: Option<Duration>) -> Self {
        let timeout = timeout.unwrap_or(Duration::from_secs(10));

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_base_url,
        }
    }

    pub async fn fetch_api_data<T>(&self, endpoint: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.api_base_url, endpoint);
        let response = self.client.get(&url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "API request failed with status: {}", 
                response.status()
            ));
        }

        let api_response: T = response.json().await?;
        Ok(api_response)
    }

    pub async fn fetch_and_store<T, U, C, F, E>(
        &self,
        endpoint: &str,
        converter: C,
        store_operation: F,
        data_type: &str,
    ) -> Result<()> 
    where
        T: DeserializeOwned,
        F: FnOnce(Vec<U>) -> BoxFuture<'static, Result<(), E>>,
        C: FnOnce(Vec<T>) -> Vec<U>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let data = self.fetch_api_data::<Vec<T>>(endpoint).await?;
        info!("{} data parsed successfully", data_type);

        let converted_data = converter(data);
        let result = store_operation(converted_data).await;
        
        match result {
            Ok(_) => {
                info!("{} data stored successfully", data_type);
                Ok(())
            },
            Err(e) => {
                error!("{} data stored failed: {}", data_type, e);
                Err(anyhow::anyhow!("{}", e))
            }
        }
    }

    pub async fn fetch_all_data(&self) -> Result<()> {
        fetch_and_store_devices(self).await?;
        fetch_and_store_beds(self).await?;
        fetch_and_store_patients(self).await?;

        Ok(())
    }
    
}
