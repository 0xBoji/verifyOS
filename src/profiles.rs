use crate::core::engine::Engine;
use crate::rules::ats::{AtsAuditRule, AtsExceptionsGranularityRule};
use crate::rules::bundle_leakage::BundleResourceLeakageRule;
use crate::rules::bundle_metadata::BundleMetadataConsistencyRule;
use crate::rules::entitlements::{EntitlementsMismatchRule, EntitlementsProvisioningMismatchRule};
use crate::rules::export_compliance::ExportComplianceRule;
use crate::rules::extensions::ExtensionEntitlementsCompatibilityRule;
use crate::rules::info_plist::{
    InfoPlistCapabilitiesRule, InfoPlistRequiredKeysRule, InfoPlistVersionConsistencyRule,
    LSApplicationQueriesSchemesAuditRule, UIRequiredDeviceCapabilitiesAuditRule,
    UsageDescriptionsRule, UsageDescriptionsValueRule,
};
use crate::rules::permissions::CameraUsageDescriptionRule;
use crate::rules::privacy::MissingPrivacyManifestRule;
use crate::rules::privacy_manifest::PrivacyManifestCompletenessRule;
use crate::rules::privacy_sdk::PrivacyManifestSdkCrossCheckRule;
use crate::rules::private_api::PrivateApiRule;
use crate::rules::signing::EmbeddedCodeSignatureTeamRule;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanProfile {
    Basic,
    Full,
}

pub fn register_rules(engine: &mut Engine, profile: ScanProfile) {
    match profile {
        ScanProfile::Basic => register_basic_rules(engine),
        ScanProfile::Full => register_full_rules(engine),
    }
}

fn register_basic_rules(engine: &mut Engine) {
    engine.register_rule(Box::new(MissingPrivacyManifestRule));
    engine.register_rule(Box::new(UsageDescriptionsRule));
    engine.register_rule(Box::new(UsageDescriptionsValueRule));
    engine.register_rule(Box::new(CameraUsageDescriptionRule));
    engine.register_rule(Box::new(AtsAuditRule));
    engine.register_rule(Box::new(AtsExceptionsGranularityRule));
    engine.register_rule(Box::new(EntitlementsMismatchRule));
    engine.register_rule(Box::new(EntitlementsProvisioningMismatchRule));
    engine.register_rule(Box::new(EmbeddedCodeSignatureTeamRule));
}

fn register_full_rules(engine: &mut Engine) {
    engine.register_rule(Box::new(MissingPrivacyManifestRule));
    engine.register_rule(Box::new(PrivacyManifestCompletenessRule));
    engine.register_rule(Box::new(PrivacyManifestSdkCrossCheckRule));
    engine.register_rule(Box::new(CameraUsageDescriptionRule));
    engine.register_rule(Box::new(UsageDescriptionsRule));
    engine.register_rule(Box::new(UsageDescriptionsValueRule));
    engine.register_rule(Box::new(InfoPlistRequiredKeysRule));
    engine.register_rule(Box::new(InfoPlistCapabilitiesRule));
    engine.register_rule(Box::new(LSApplicationQueriesSchemesAuditRule));
    engine.register_rule(Box::new(UIRequiredDeviceCapabilitiesAuditRule));
    engine.register_rule(Box::new(InfoPlistVersionConsistencyRule));
    engine.register_rule(Box::new(ExportComplianceRule));
    engine.register_rule(Box::new(AtsAuditRule));
    engine.register_rule(Box::new(AtsExceptionsGranularityRule));
    engine.register_rule(Box::new(EntitlementsMismatchRule));
    engine.register_rule(Box::new(EntitlementsProvisioningMismatchRule));
    engine.register_rule(Box::new(BundleMetadataConsistencyRule));
    engine.register_rule(Box::new(BundleResourceLeakageRule));
    engine.register_rule(Box::new(ExtensionEntitlementsCompatibilityRule));
    engine.register_rule(Box::new(PrivateApiRule));
    engine.register_rule(Box::new(EmbeddedCodeSignatureTeamRule));
}
