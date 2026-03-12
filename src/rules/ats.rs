use crate::rules::core::{
    AppStoreRule, ArtifactContext, RuleCategory, RuleError, RuleReport, RuleStatus, Severity,
};

pub struct AtsAuditRule;

impl AppStoreRule for AtsAuditRule {
    fn id(&self) -> &'static str {
        "RULE_ATS_AUDIT"
    }

    fn name(&self) -> &'static str {
        "ATS Exceptions Detected"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Ats
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Remove ATS exceptions or scope them to specific domains with justification."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let Some(ats_dict) = plist.get_dictionary("NSAppTransportSecurity") else {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        };

        let mut issues = Vec::new();

        if let Some(true) = ats_dict
            .get("NSAllowsArbitraryLoads")
            .and_then(|v| v.as_boolean())
        {
            issues.push("NSAllowsArbitraryLoads=true".to_string());
        }

        if let Some(true) = ats_dict
            .get("NSAllowsArbitraryLoadsInWebContent")
            .and_then(|v| v.as_boolean())
        {
            issues.push("NSAllowsArbitraryLoadsInWebContent=true".to_string());
        }

        if let Some(domains) = ats_dict
            .get("NSExceptionDomains")
            .and_then(|v| v.as_dictionary())
        {
            for (domain, config) in domains {
                if let Some(true) = config
                    .as_dictionary()
                    .and_then(|d| d.get("NSExceptionAllowsInsecureHTTPLoads"))
                    .and_then(|v| v.as_boolean())
                {
                    issues.push(format!("NSExceptionAllowsInsecureHTTPLoads for {domain}"));
                }
            }
        }

        if issues.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("ATS exceptions detected".to_string()),
            evidence: Some(issues.join("; ")),
        })
    }
}

pub struct AtsExceptionsGranularityRule;

impl AppStoreRule for AtsExceptionsGranularityRule {
    fn id(&self) -> &'static str {
        "RULE_ATS_GRANULARITY"
    }

    fn name(&self) -> &'static str {
        "ATS Exceptions Too Broad"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Ats
    }

    fn severity(&self) -> Severity {
        Severity::Warning
    }

    fn recommendation(&self) -> &'static str {
        "Avoid global ATS relaxations; scope exceptions to specific domains without IncludesSubdomains unless required."
    }

    fn evaluate(&self, artifact: &ArtifactContext) -> Result<RuleReport, RuleError> {
        let Some(plist) = artifact.info_plist else {
            return Ok(RuleReport {
                status: RuleStatus::Skip,
                message: Some("Info.plist not found".to_string()),
                evidence: None,
            });
        };

        let Some(ats_dict) = plist.get_dictionary("NSAppTransportSecurity") else {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: None,
                evidence: None,
            });
        };

        let mut issues = Vec::new();

        if is_true(ats_dict, "NSAllowsArbitraryLoads") {
            issues.push("NSAllowsArbitraryLoads=true".to_string());
        }

        if is_true(ats_dict, "NSAllowsArbitraryLoadsInWebContent") {
            issues.push("NSAllowsArbitraryLoadsInWebContent=true".to_string());
        }

        if is_true(ats_dict, "NSAllowsArbitraryLoadsForMedia") {
            issues.push("NSAllowsArbitraryLoadsForMedia=true".to_string());
        }

        if is_true(ats_dict, "NSAllowsArbitraryLoadsForWebContent") {
            issues.push("NSAllowsArbitraryLoadsForWebContent=true".to_string());
        }

        if let Some(domains) = ats_dict
            .get("NSExceptionDomains")
            .and_then(|v| v.as_dictionary())
        {
            for (domain, config) in domains {
                let Some(domain_dict) = config.as_dictionary() else {
                    continue;
                };

                if is_true(domain_dict, "NSIncludesSubdomains") {
                    issues.push(format!("NSIncludesSubdomains=true for {domain}"));
                }

                if is_true(domain_dict, "NSExceptionAllowsInsecureHTTPLoads") {
                    issues.push(format!("NSExceptionAllowsInsecureHTTPLoads for {domain}"));
                }

                if is_true(domain_dict, "NSExceptionRequiresForwardSecrecy") == false
                    && domain_dict.contains_key("NSExceptionRequiresForwardSecrecy")
                {
                    issues.push(format!(
                        "NSExceptionRequiresForwardSecrecy=false for {domain}"
                    ));
                }

                if is_true(domain_dict, "NSRequiresCertificateTransparency") == false
                    && domain_dict.contains_key("NSRequiresCertificateTransparency")
                {
                    issues.push(format!(
                        "NSRequiresCertificateTransparency=false for {domain}"
                    ));
                }
            }
        }

        if issues.is_empty() {
            return Ok(RuleReport {
                status: RuleStatus::Pass,
                message: Some("ATS exceptions look scoped".to_string()),
                evidence: None,
            });
        }

        Ok(RuleReport {
            status: RuleStatus::Fail,
            message: Some("ATS exceptions are overly broad".to_string()),
            evidence: Some(issues.join(" | ")),
        })
    }
}

fn is_true(dict: &plist::Dictionary, key: &str) -> bool {
    dict.get(key).and_then(|v| v.as_boolean()) == Some(true)
}
