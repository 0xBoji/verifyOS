use crate::parsers::plist_reader::InfoPlist;
use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

const USAGE_DESCRIPTION_KEYS: &[&str] = &[
    "NSCameraUsageDescription",
    "NSMicrophoneUsageDescription",
    "NSPhotoLibraryUsageDescription",
    "NSPhotoLibraryAddUsageDescription",
    "NSLocationWhenInUseUsageDescription",
    "NSLocationAlwaysAndWhenInUseUsageDescription",
    "NSLocationAlwaysUsageDescription",
    "NSBluetoothAlwaysUsageDescription",
    "NSBluetoothPeripheralUsageDescription",
    "NSFaceIDUsageDescription",
    "NSCalendarsUsageDescription",
    "NSRemindersUsageDescription",
    "NSContactsUsageDescription",
    "NSSpeechRecognitionUsageDescription",
    "NSMotionUsageDescription",
    "NSAppleMusicUsageDescription",
    "NSHealthShareUsageDescription",
    "NSHealthUpdateUsageDescription",
];

pub struct UsageDescriptionsRule;

impl AppStoreRule for UsageDescriptionsRule {
    fn id(&self) -> &'static str {
        "RULE_USAGE_DESCRIPTIONS"
    }

    fn name(&self) -> &'static str {
        "Missing Usage Description Keys"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Privacy
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Add NS*UsageDescription keys required by your app's feature usage."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let missing: Vec<&str> = USAGE_DESCRIPTION_KEYS
            .iter()
            .copied()
            .filter(|key| !plist.has_key(key))
            .collect();

        if missing.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("One or more NS*UsageDescription keys are missing".to_string()),
            evidence: Some(format!("Missing keys: {}", missing.join(", "))),
        })
    }
}

pub struct UsageDescriptionsValueRule;

impl AppStoreRule for UsageDescriptionsValueRule {
    fn id(&self) -> &'static str {
        "RULE_USAGE_DESCRIPTIONS_EMPTY"
    }

    fn name(&self) -> &'static str {
        "Empty Usage Description Values"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Privacy
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Ensure NS*UsageDescription values are non-empty and user-facing."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let empty: Vec<&str> = USAGE_DESCRIPTION_KEYS
            .iter()
            .copied()
            .filter(|key| is_empty_string(plist, key))
            .collect();

        if empty.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Usage description values are empty".to_string()),
            evidence: Some(format!("Empty keys: {}", empty.join(", "))),
        })
    }
}

pub struct InfoPlistRequiredKeysRule;

impl AppStoreRule for InfoPlistRequiredKeysRule {
    fn id(&self) -> &'static str {
        "RULE_INFO_PLIST_REQUIRED_KEYS"
    }

    fn name(&self) -> &'static str {
        "Missing Required Info.plist Keys"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Metadata
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Add required Info.plist keys for your app's functionality."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let mut missing = Vec::new();
        if !plist.has_key("LSApplicationQueriesSchemes") {
            missing.push("LSApplicationQueriesSchemes");
        }
        if !plist.has_key("UIRequiredDeviceCapabilities") {
            missing.push("UIRequiredDeviceCapabilities");
        }

        if missing.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Missing required Info.plist keys".to_string()),
            evidence: Some(format!("Missing keys: {}", missing.join(", "))),
        })
    }
}

pub struct InfoPlistCapabilitiesRule;

impl AppStoreRule for InfoPlistCapabilitiesRule {
    fn id(&self) -> &'static str {
        "RULE_INFO_PLIST_CAPABILITIES_EMPTY"
    }

    fn name(&self) -> &'static str {
        "Empty Info.plist Capability Lists"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Metadata
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Remove empty arrays or populate capability keys with valid values."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let mut empty = Vec::new();

        if is_empty_array(plist, "LSApplicationQueriesSchemes") {
            empty.push("LSApplicationQueriesSchemes");
        }

        if is_empty_array(plist, "UIRequiredDeviceCapabilities") {
            empty.push("UIRequiredDeviceCapabilities");
        }

        if empty.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("Capability keys are present but empty".to_string()),
            evidence: Some(format!("Empty keys: {}", empty.join(", "))),
        })
    }
}

fn is_empty_string(plist: &InfoPlist, key: &str) -> bool {
    match plist.get_string(key) {
        Some(value) => value.trim().is_empty(),
        None => false,
    }
}

fn is_empty_array(plist: &InfoPlist, key: &str) -> bool {
    match plist.get_value(key) {
        Some(value) => value.as_array().map(|arr| arr.is_empty()).unwrap_or(false),
        None => false,
    }
}
