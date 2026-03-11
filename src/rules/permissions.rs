use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

pub struct CameraUsageDescriptionRule;

impl AppStoreRule for CameraUsageDescriptionRule {
    fn id(&self) -> &'static str {
        "RULE_CAMERA_USAGE"
    }

    fn name(&self) -> &'static str {
        "Missing Camera Usage Description"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Permissions
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }

    fn recommendation(&self) -> &'static str {
        "Add NSCameraUsageDescription to Info.plist with a user-facing reason."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        if let Some(plist) = artifact.info_plist {
            if !plist.has_key("NSCameraUsageDescription") {
                return Ok(RuleReport {
                    status: RuleStatus::Fail,
                    message: Some("Missing NSCameraUsageDescription".to_string()),
                    evidence: Some("Info.plist has no NSCameraUsageDescription".to_string()),
                });
            }
        }

        Ok(RuleReport {
            status: RuleStatus::Pass,
            message: None,
            evidence: None,
        })
    }
}
