use crate::app::{ScanError, ScanService};
use crate::domain::{ScanProfileInput, ScanRequest};
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use std::io::Write;
use tempfile::NamedTempFile;
use tracing::info;

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn scan_bundle(
    State(service): State<ScanService>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut request = ScanRequest { profile: None };
    let mut temp_file: Option<NamedTempFile> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        if name == "profile" {
            if let Ok(value) = field.text().await {
                request.profile = match value.to_lowercase().as_str() {
                    "basic" => Some(ScanProfileInput::Basic),
                    "full" => Some(ScanProfileInput::Full),
                    _ => None,
                };
            }
            continue;
        }

        if name == "bundle" {
            let mut file = match NamedTempFile::new() {
                Ok(file) => file,
                Err(err) => return to_error(err).into_response(),
            };
            let bytes = match field.bytes().await {
                Ok(bytes) => bytes,
                Err(err) => return to_error(err).into_response(),
            };
            if let Err(err) = file.write_all(&bytes) {
                return to_error(err).into_response();
            }
            temp_file = Some(file);
        }
    }

    let Some(bundle) = temp_file else {
        return (
            StatusCode::BAD_REQUEST,
            json!({ "error": "missing bundle file field" }),
        )
            .into_response();
    };

    info!("running scan for uploaded bundle");
    match service.run_scan(request, bundle.path()) {
        Ok(result) => (
            StatusCode::OK,
            serde_json::to_value(result).unwrap_or_default(),
        )
            .into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, error_body(err)).into_response(),
    }
}

fn error_body(err: ScanError) -> serde_json::Value {
    json!({ "error": err.to_string() })
}

fn to_error(err: impl std::fmt::Display) -> (StatusCode, serde_json::Value) {
    (StatusCode::BAD_REQUEST, json!({ "error": err.to_string() }))
}
