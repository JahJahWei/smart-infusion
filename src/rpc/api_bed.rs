use serde::{Deserialize, Serialize};
use anyhow::Result;
use sqlx;
use crate::{http_client::HttpClient, repository::{insert_beds, Bed}};

const API_BED_ENDPOINT: &str = "/api/beds";
const BED_DATA_TYPE: &str = "beds";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiBed {
    bed_no: String,
    mac: Option<String>,
}
impl From<ApiBed> for Bed {
    fn from(bed: ApiBed) -> Self {
        Bed::new(bed.bed_no, bed.mac)
    }
}

pub async fn fetch_and_store_beds(http_client: &HttpClient) -> Result<()> {
    http_client.fetch_and_store::<ApiBed, Bed, _, _, sqlx::Error>(
        API_BED_ENDPOINT,
        |beds| beds.into_iter().map(|bed| bed.into()).collect(),
        |beds| Box::pin(async move { insert_beds(beds).await }),
        BED_DATA_TYPE
    ).await
}

