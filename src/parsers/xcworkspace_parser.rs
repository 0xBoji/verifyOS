use miette::Diagnostic;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum WorkspaceError {
    #[error("Failed to read Xcode workspace at {path}")]
    ReadError { path: String, description: String },
    #[error("Missing contents.xcworkspacedata in workspace {path}")]
    MissingContents { path: String },
}

#[derive(Debug, Clone)]
pub struct Xcworkspace {
    pub project_paths: Vec<PathBuf>,
}

impl Xcworkspace {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, WorkspaceError> {
        let path = path.as_ref();
        let contents_path = if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("xcworkspacedata"))
        {
            path.to_path_buf()
        } else {
            let workspace_root = path;
            let contents = workspace_root.join("contents.xcworkspacedata");
            if !contents.exists() {
                return Err(WorkspaceError::MissingContents {
                    path: workspace_root.display().to_string(),
                });
            }
            contents
        };

        let data =
            std::fs::read_to_string(&contents_path).map_err(|e| WorkspaceError::ReadError {
                path: contents_path.display().to_string(),
                description: format!("{e}"),
            })?;

        let workspace_dir = contents_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));

        let mut project_paths = Vec::new();
        for location in extract_locations(&data) {
            if let Some(path) = resolve_location(&workspace_dir, &location) {
                if path.extension().is_some_and(|ext| ext == "xcodeproj") {
                    project_paths.push(path);
                }
            }
        }

        Ok(Self { project_paths })
    }
}

fn extract_locations(data: &str) -> Vec<String> {
    let mut locations = Vec::new();
    let needle = "location=\"";
    let mut start = 0;
    while let Some(pos) = data[start..].find(needle) {
        let idx = start + pos + needle.len();
        if let Some(end) = data[idx..].find('"') {
            locations.push(data[idx..idx + end].to_string());
            start = idx + end + 1;
        } else {
            break;
        }
    }
    locations
}

fn resolve_location(workspace_dir: &Path, location: &str) -> Option<PathBuf> {
    if let Some(rest) = location.strip_prefix("group:") {
        Some(workspace_dir.join(rest))
    } else if let Some(rest) = location.strip_prefix("container:") {
        Some(workspace_dir.join(rest))
    } else if let Some(rest) = location.strip_prefix("absolute:") {
        Some(PathBuf::from(rest))
    } else if location.starts_with('/') {
        Some(PathBuf::from(location))
    } else {
        Some(workspace_dir.join(location))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parses_workspace_file_refs() {
        let dir = tempdir().expect("tempdir");
        let workspace_dir = dir.path().join("Demo.xcworkspace");
        fs::create_dir_all(&workspace_dir).expect("workspace dir");
        let contents = workspace_dir.join("contents.xcworkspacedata");
        fs::write(
            &contents,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Workspace version="1.0">
   <FileRef location="group:Demo.xcodeproj"></FileRef>
</Workspace>"#,
        )
        .expect("write contents");

        let workspace = Xcworkspace::from_path(&workspace_dir).expect("parse workspace");
        assert_eq!(workspace.project_paths.len(), 1);
        assert!(workspace.project_paths[0].ends_with("Demo.xcodeproj"));
    }
}
