use crate::parsers::zip_extractor::extract_ipa;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SizeReport {
    pub app_path: String,
    pub total_bytes: u64,
    pub categories: Vec<CategorySummary>,
    pub top_files: Vec<SizeEntry>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CategorySummary {
    pub category: String,
    pub bytes: u64,
    pub file_count: usize,
    pub percent_of_total: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SizeEntry {
    pub path: String,
    pub category: String,
    pub bytes: u64,
}

pub fn analyze_app_size(path: &Path, top_n: usize) -> Result<SizeReport, miette::Report> {
    let extension = path.extension().and_then(|ext| ext.to_str());
    match extension {
        Some("ipa") => {
            let extracted = extract_ipa(path).map_err(|err| {
                miette::miette!("Failed to extract IPA {}: {}", path.display(), err)
            })?;
            let app_bundle = extracted
                .get_app_bundle_path()
                .map_err(|err| {
                    miette::miette!(
                        "Failed to inspect extracted IPA payload for {}: {}",
                        path.display(),
                        err
                    )
                })?
                .ok_or_else(|| {
                    miette::miette!(
                        "No .app bundle found inside extracted IPA {}",
                        path.display()
                    )
                })?;
            analyze_app_bundle(&app_bundle, top_n)
        }
        Some("app") | None => analyze_app_bundle(path, top_n),
        Some(other) => Err(miette::miette!(
            "Unsupported app artifact `{}`. Expected .ipa or .app",
            other
        )),
    }
}

pub fn analyze_app_bundle(
    app_bundle_path: &Path,
    top_n: usize,
) -> Result<SizeReport, miette::Report> {
    if !app_bundle_path.exists() {
        return Err(miette::miette!(
            "App bundle does not exist: {}",
            app_bundle_path.display()
        ));
    }

    let mut entries = Vec::new();
    collect_files(app_bundle_path, app_bundle_path, &mut entries)?;
    entries.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.path.cmp(&b.path)));

    let total_bytes = entries.iter().map(|item| item.bytes).sum::<u64>();
    let categories = summarize_categories(&entries, total_bytes);
    let top_files = entries.into_iter().take(top_n).collect();

    Ok(SizeReport {
        app_path: app_bundle_path.display().to_string(),
        total_bytes,
        categories,
        top_files,
    })
}

fn collect_files(
    root: &Path,
    current: &Path,
    entries: &mut Vec<SizeEntry>,
) -> Result<(), miette::Report> {
    for entry in fs::read_dir(current)
        .map_err(|err| miette::miette!("Failed to read {}: {}", current.display(), err))?
    {
        let entry = entry
            .map_err(|err| miette::miette!("Failed to walk {}: {}", current.display(), err))?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(root, &path, entries)?;
            continue;
        }

        let metadata = entry
            .metadata()
            .map_err(|err| miette::miette!("Failed to stat {}: {}", path.display(), err))?;
        let relative = normalized_relative_path(root, &path);
        entries.push(SizeEntry {
            path: relative,
            category: classify_path(root, &path).to_string(),
            bytes: metadata.len(),
        });
    }

    Ok(())
}

fn summarize_categories(entries: &[SizeEntry], total_bytes: u64) -> Vec<CategorySummary> {
    let mut by_category: BTreeMap<String, (u64, usize)> = BTreeMap::new();
    for entry in entries {
        let row = by_category.entry(entry.category.clone()).or_insert((0, 0));
        row.0 += entry.bytes;
        row.1 += 1;
    }

    let mut categories: Vec<CategorySummary> = by_category
        .into_iter()
        .map(|(category, (bytes, file_count))| CategorySummary {
            category,
            bytes,
            file_count,
            percent_of_total: if total_bytes == 0 {
                0.0
            } else {
                ((bytes as f64 / total_bytes as f64) * 1000.0).round() / 10.0
            },
        })
        .collect();
    categories.sort_by(|a, b| {
        b.bytes
            .cmp(&a.bytes)
            .then_with(|| a.category.cmp(&b.category))
    });
    categories
}

fn classify_path(root: &Path, path: &Path) -> &'static str {
    let relative = path.strip_prefix(root).unwrap_or(path);
    let parts: Vec<String> = relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy().to_string())
        .collect();

    if parts.iter().any(|part| part == "Frameworks") {
        return "framework";
    }
    if parts
        .iter()
        .any(|part| part == "PlugIns" || part == "Extensions")
    {
        return "extension";
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if matches!(
        file_name,
        "Info.plist" | "PkgInfo" | "embedded.mobileprovision" | "PrivacyInfo.xcprivacy"
    ) || matches!(extension.as_str(), "plist" | "strings" | "mobileprovision")
    {
        return "metadata";
    }

    if matches!(
        extension.as_str(),
        "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "webp"
            | "heic"
            | "car"
            | "pdf"
            | "json"
            | "ttf"
            | "otf"
            | "mp3"
            | "mp4"
            | "wav"
            | "storyboardc"
            | "nib"
    ) || file_name == "Assets.car"
    {
        return "asset";
    }

    if extension.is_empty() || matches!(extension.as_str(), "dylib" | "metallib") {
        return "binary";
    }

    "resource"
}

fn normalized_relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|part| part.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<String>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::analyze_app_bundle;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn analyzes_bundle_size_breakdown() {
        let dir = tempdir().expect("temp dir");
        let app = dir.path().join("Demo.app");
        fs::create_dir_all(app.join("Frameworks/Foo.framework")).expect("framework dir");
        fs::create_dir_all(app.join("PlugIns/Share.appex")).expect("appex dir");
        fs::write(app.join("Demo"), vec![0u8; 10]).expect("write binary");
        fs::write(app.join("Assets.car"), vec![0u8; 20]).expect("write asset");
        fs::write(app.join("Frameworks/Foo.framework/Foo"), vec![0u8; 30])
            .expect("write framework");
        fs::write(app.join("PlugIns/Share.appex/Share"), vec![0u8; 15]).expect("write appex");
        fs::write(app.join("Info.plist"), vec![0u8; 5]).expect("write plist");

        let report = analyze_app_bundle(&app, 3).expect("analyze bundle");

        assert_eq!(report.total_bytes, 80);
        assert_eq!(report.top_files.len(), 3);
        assert_eq!(report.top_files[0].path, "Frameworks/Foo.framework/Foo");
        assert_eq!(report.top_files[0].category, "framework");
        assert_eq!(report.categories[0].category, "framework");
        assert_eq!(report.categories[0].bytes, 30);
        assert!(report
            .categories
            .iter()
            .any(|item| item.category == "asset"));
        assert!(report
            .categories
            .iter()
            .any(|item| item.category == "binary"));
        assert!(report
            .categories
            .iter()
            .any(|item| item.category == "extension"));
        assert!(report
            .categories
            .iter()
            .any(|item| item.category == "metadata"));
    }
}
