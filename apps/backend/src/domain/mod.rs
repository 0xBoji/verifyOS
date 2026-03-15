use serde::{Deserialize, Serialize};
use verifyos_cli::report::ReportData;

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
    pub report: ReportData,
    pub warnings: Vec<String>,
}
