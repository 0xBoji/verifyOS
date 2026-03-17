use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use zip::ZipArchive;

#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),
    #[error("Zip Error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Invalid IPA path: {0}")]
    InvalidPath(PathBuf),
}

pub struct ExtractedIpa {
    pub temp_dir: TempDir,
    pub payload_dir: PathBuf,
}

impl ExtractedIpa {
    pub fn get_app_bundle_path(&self) -> io::Result<Option<PathBuf>> {
        if !self.payload_dir.exists() {
            return Ok(None);
        }

        // Try direct root search first
        for entry in fs::read_dir(&self.payload_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("app") {
                return Ok(Some(path));
            }
        }

        // Fallback: recursive search
        let mut queue = vec![self.payload_dir.clone()];
        while let Some(dir) = queue.pop() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if path.extension().and_then(|e| e.to_str()) == Some("app") {
                            return Ok(Some(path));
                        }
                        queue.push(path);
                    }
                }
            }
        }

        Ok(None)
    }

    pub fn get_project_path(&self) -> io::Result<Option<PathBuf>> {
        let mut project = None;
        let mut queue = vec![self.payload_dir.clone()];

        while let Some(dir) = queue.pop() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let extension = path.extension().and_then(|e| e.to_str());
                        if extension == Some("xcworkspace") {
                            return Ok(Some(path));
                        }
                        if project.is_none() && extension == Some("xcodeproj") {
                            project = Some(path.clone());
                        }
                        queue.push(path);
                    }
                }
            }
        }

        Ok(project)
    }

    pub fn discover_targets(&self) -> io::Result<Vec<(PathBuf, String)>> {
        let mut targets = Vec::new();
        let mut queue = vec![self.payload_dir.clone()];

        while let Some(dir) = queue.pop() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let extension = path.extension().and_then(|e| e.to_str());
                        match extension {
                            Some("app") => targets.push((path.clone(), "app".to_string())),
                            Some("xcodeproj") => targets.push((path.clone(), "project".to_string())),
                            Some("xcworkspace") => {
                                targets.push((path.clone(), "workspace".to_string()))
                            }
                            _ => queue.push(path),
                        }
                    } else if path.extension().and_then(|e| e.to_str()) == Some("ipa") {
                        targets.push((path.clone(), "ipa".to_string()));
                    }
                }
            }
        }
        Ok(targets)
    }
}

pub fn extract_ipa<P: AsRef<Path>>(ipa_path: P) -> Result<ExtractedIpa, ExtractionError> {
    let path = ipa_path.as_ref();
    if !path.exists() {
        return Err(ExtractionError::InvalidPath(path.to_path_buf()));
    }
    let file = fs::File::open(path)?;
    let mut archive = ZipArchive::new(file)?;
    let temp_dir = tempfile::tempdir()?;
    let extract_path = temp_dir.path();

    archive.extract(extract_path)?;

    let mut payload_dir = extract_path.join("Payload");
    if !payload_dir.exists() {
        payload_dir = extract_path.to_path_buf();
    }

    Ok(ExtractedIpa {
        temp_dir,
        payload_dir,
    })
}
