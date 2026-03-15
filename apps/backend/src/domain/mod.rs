use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanProfileInput {
    Basic,
    Full,
}

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub profile: Option<ScanProfileInput>,
}

#[derive(Debug, Serialize)]
pub struct ScanResponse {
    pub report: serde_json::Value,
    pub warnings: Vec<String>,
}
