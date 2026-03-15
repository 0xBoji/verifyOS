use crate::app::{ScanError, ScanService};
use crate::domain::{ScanProfileInput, ScanRequest};
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use tracing::info;
use zip::ZipArchive;

pub async fn health() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn scan_bundle(
    State(service): State<ScanService>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut request = ScanRequest { profile: None };
    let mut temp_file: Option<NamedTempFile> = None;
    let mut project_file: Option<NamedTempFile> = None;
    let mut project_path: Option<PathBuf> = None;
    let mut project_dir: Option<tempfile::TempDir> = None;

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
            continue;
        }

        if name == "project" {
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
            project_file = Some(file);
        }
    }

    let Some(bundle) = temp_file else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "missing bundle file field" })),
        )
            .into_response();
    };

    if let Some(project_file) = project_file {
        let path = project_file.path();
        if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
            if ext.eq_ignore_ascii_case("zip") {
                match extract_project_zip(path) {
                    Ok((dir, project)) => {
                        project_dir = Some(dir);
                        project_path = project;
                    }
                    Err(err) => return to_error(err).into_response(),
                }
            } else if ext.eq_ignore_ascii_case("xcodeproj")
                || ext.eq_ignore_ascii_case("xcworkspace")
            {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Upload .xcodeproj/.xcworkspace as a .zip archive."
                    })),
                )
                    .into_response();
            }
        }
    }

    info!("running scan for uploaded bundle");
    let _keep_project_dir_alive = project_dir;
    match service.run_scan(request, bundle.path(), project_path.as_deref()) {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(error_body(err))).into_response(),
    }
}

fn error_body(err: ScanError) -> serde_json::Value {
    json!({ "error": err.to_string() })
}

fn to_error(err: impl std::fmt::Display) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({ "error": err.to_string() })),
    )
}

fn extract_project_zip(
    path: &Path,
) -> Result<(tempfile::TempDir, Option<PathBuf>), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let file = std::fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let out_path = dir.path().join(entry.name());
        if entry.name().ends_with('/') {
            std::fs::create_dir_all(&out_path)?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut outfile = std::fs::File::create(&out_path)?;
        std::io::copy(&mut entry, &mut outfile)?;
    }

    let project = find_project_path(dir.path());
    Ok((dir, project))
}

fn find_project_path(root: &Path) -> Option<PathBuf> {
    let mut queue = vec![root.to_path_buf()];
    let mut project = None;
    while let Some(dir) = queue.pop() {
        let entries = std::fs::read_dir(&dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("xcworkspace"))
                {
                    return Some(path);
                }
                if project.is_none()
                    && path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("xcodeproj"))
                {
                    project = Some(path.clone());
                }
                queue.push(path);
            }
        }
    }
    project
}
