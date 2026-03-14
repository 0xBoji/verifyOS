use miette::Diagnostic;
use std::path::Path;
use thiserror::Error;
use xcodeproj::XCodeProject;

#[derive(Debug, Error, Diagnostic)]
pub enum XcodeError {
    #[error("Failed to read Xcode project at {path}")]
    ReadError { path: String, description: String },
}

pub struct XcodeProject {
    pub inner: XCodeProject,
}

impl XcodeProject {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, XcodeError> {
        let path = path.as_ref();

        let inner = XCodeProject::new(path).map_err(|e| XcodeError::ReadError {
            path: path.display().to_string(),
            description: format!("{e:?}"),
        })?;

        Ok(Self { inner })
    }

    pub fn target_names(&self) -> Vec<String> {
        self.inner
            .targets()
            .iter()
            .filter_map(|t| t.name.map(|s| s.to_string()))
            .collect()
    }
}
