use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{http_client::HttpClient, repository::{insert_devices, Device}};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiDevice {
    dev_no: u8,
    mac: Option<String>,
}

const API_DEVICE_ENDPOINT: &str = "/api/devices";

impl From<ApiDevice> for Device {
    fn from(device: ApiDevice) -> Self {
        Device::new(device.dev_no, device.mac)
    }
}

pub async fn fetch_and_store_devices(http_client: &HttpClient) -> Result<()> {
    http_client.fetch_and_store::<ApiDevice, Device, _, _, sqlx::Error>(
        API_DEVICE_ENDPOINT,
        |devices| devices.into_iter().map(|device| device.into()).collect(),
        |devices| Box::pin(async move { insert_devices(devices).await }),
        "devices"
    ).await
}


