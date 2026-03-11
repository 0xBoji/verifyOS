use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

pub struct MissingPrivacyManifestRule;

impl AppStoreRule for MissingPrivacyManifestRule {
    fn id(&self) -> &'static str {
        "RULE_PRIVACY_MANIFEST"
    }

    fn name(&self) -> &'static str {
        "Missing Privacy Manifest"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Privacy
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn recommendation(&self) -> &'static str {
        "Add a PrivacyInfo.xcprivacy file to the app bundle."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let manifest_path = artifact.app_bundle_path.join("PrivacyInfo.xcprivacy");
        if !manifest_path.exists() {
            return Ok(RuleReport {
                status: RuleStatus::Fail,
                message: Some("Missing PrivacyInfo.xcprivacy".to_string()),
                evidence: Some(format!("Not found at {}", manifest_path.display())),
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Pass,
            message: None,
            evidence: None,
        })
    }
}
