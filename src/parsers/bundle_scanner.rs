use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum BundleScanError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct BundleTarget {
    pub bundle_path: PathBuf,
    pub display_name: String,
}

pub fn find_nested_bundles(app_bundle_path: &Path) -> Result<Vec<BundleTarget>, BundleScanError> {
    let mut bundles = Vec::new();

    let frameworks = app_bundle_path.join("Frameworks");
    collect_bundles_in_dir(&frameworks, &mut bundles)?;

    let plugins = app_bundle_path.join("PlugIns");
    collect_bundles_in_dir(&plugins, &mut bundles)?;

    let extensions = app_bundle_path.join("Extensions");
    collect_bundles_in_dir(&extensions, &mut bundles)?;

    bundles.sort_by(|a, b| a.bundle_path.cmp(&b.bundle_path));
    bundles.dedup_by(|a, b| a.bundle_path == b.bundle_path);

    Ok(bundles)
}

fn collect_bundles_in_dir(
    dir: &Path,
    bundles: &mut Vec<BundleTarget>,
) -> Result<(), BundleScanError> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("app")
            || path.extension().and_then(|e| e.to_str()) == Some("appex")
            || path.extension().and_then(|e| e.to_str()) == Some("framework")
        {
            bundles.push(BundleTarget {
                display_name: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                bundle_path: path.clone(),
            });
        }

        if path.is_dir() {
            collect_bundles_in_dir(&path, bundles)?;
        }
    }

    Ok(())
}
