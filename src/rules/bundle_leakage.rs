use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};
use std::path::Path;

pub struct BundleResourceLeakageRule;

impl AppStoreRule for BundleResourceLeakageRule {
    fn id(&self) -> &'static str {
        "RULE_BUNDLE_RESOURCE_LEAKAGE"
    }

    fn name(&self) -> &'static str {
        "Sensitive Files in Bundle"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Bundling
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn recommendation(&self) -> &'static str {
        "Remove certificates, provisioning profiles, or env files from the app bundle before submission."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let offenders = scan_bundle_for_sensitive_files(artifact.app_bundle_path, 80)?;

        if offenders.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("No sensitive files found in bundle".to_string()),
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Sensitive files found in bundle".to_string()),
            evidence: Some(offenders.join(" | ")),
        })
    }
}

fn scan_bundle_for_sensitive_files(
    app_bundle_path: &Path,
    limit: usize,
) -> Result<Vec<String>, RuleError> {
    let mut hits = Vec::new();
    let mut stack = vec![app_bundle_path.to_path_buf()];

    while let Some(path) = stack.pop() {
        let entries = match std::fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };
            let path = entry.path();

            if path.is_dir() {
                stack.push(path);
                continue;
            }

            if is_sensitive_path(&path) {
                let display = match path.strip_prefix(app_bundle_path) {
                    Ok(rel) => rel.display().to_string(),
                    Err(_) => path.display().to_string(),
                };
                hits.push(display);
                if hits.len() >= limit {
                    return Ok(hits);
                }
            }
        }
    }

    Ok(hits)
}

fn is_sensitive_path(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if name == ".env" || name.ends_with(".env") {
        return true;
    }

    if matches!(
        path.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()),
        Some(ext) if ext == "p12" || ext == "pfx" || ext == "pem" || ext == "key"
    ) {
        return true;
    }

    if name == "embedded.mobileprovision" {
        return true;
    }

    if name.ends_with(".mobileprovision") {
        return true;
    }

    if name.contains("secret") || name.contains("apikey") || name.contains("api_key") {
        return true;
    }

    false
}
