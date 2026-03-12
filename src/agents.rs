use crate::profiles::{rule_inventory, RuleInventoryItem};
use std::path::Path;

const MANAGED_START: &str = "<!-- verifyos-cli:agents:start -->";
const MANAGED_END: &str = "<!-- verifyos-cli:agents:end -->";

pub fn write_agents_file(path: &Path) -> Result<(), miette::Report> {
    let existing = if path.exists() {
        Some(std::fs::read_to_string(path).map_err(|err| {
            miette::miette!(
                "Failed to read existing AGENTS.md at {}: {}",
                path.display(),
                err
            )
        })?)
    } else {
        None
    };

    let managed_block = build_managed_block();
    let next = merge_agents_content(existing.as_deref(), &managed_block);
    std::fs::write(path, next)
        .map_err(|err| miette::miette!("Failed to write AGENTS.md at {}: {}", path.display(), err))
}

pub fn merge_agents_content(existing: Option<&str>, managed_block: &str) -> String {
    match existing {
        None => format!("# AGENTS.md\n\n{}", managed_block),
        Some(content) => {
            if let Some((start, end)) = managed_block_range(content) {
                let mut next = String::new();
                next.push_str(&content[..start]);
                if !next.ends_with('\n') {
                    next.push('\n');
                }
                next.push_str(managed_block);
                let tail = &content[end..];
                if !tail.is_empty() && !tail.starts_with('\n') {
                    next.push('\n');
                }
                next.push_str(tail);
                next
            } else if content.trim().is_empty() {
                format!("# AGENTS.md\n\n{}", managed_block)
            } else {
                let mut next = content.trim_end().to_string();
                next.push_str("\n\n");
                next.push_str(managed_block);
                next.push('\n');
                next
            }
        }
    }
}

pub fn build_managed_block() -> String {
    let inventory = rule_inventory();
    let mut out = String::new();
    out.push_str(MANAGED_START);
    out.push('\n');
    out.push_str("## verifyOS-cli\n\n");
    out.push_str("Use `voc` before large iOS submission changes or release builds.\n\n");
    out.push_str("### Recommended Workflow\n\n");
    out.push_str("1. Run `voc --app <path-to-.ipa-or-.app> --profile basic` for a quick gate.\n");
    out.push_str("2. Run `voc --app <path-to-.ipa-or-.app> --profile full --agent-pack .verifyos-agent --agent-pack-format bundle` before release or when an AI agent will patch findings.\n");
    out.push_str(
        "3. Read `.verifyos-agent/agent-pack.md` first, then patch the highest-priority scopes.\n",
    );
    out.push_str("4. Re-run `voc` after each fix batch until the pack is clean.\n\n");
    out.push_str("### AI Agent Rules\n\n");
    out.push_str("- Prefer `voc --profile basic` during fast inner loops and `voc --profile full` before shipping.\n");
    out.push_str("- When findings exist, generate an agent bundle with `voc --agent-pack .verifyos-agent --agent-pack-format bundle`.\n");
    out.push_str("- Fix `high` priority findings before `medium` and `low`.\n");
    out.push_str("- Treat `Info.plist`, `entitlements`, `ats-config`, and `bundle-resources` as the main fix scopes.\n");
    out.push_str("- Re-run `voc` after edits and compare against the previous agent pack to confirm findings were actually removed.\n\n");
    out.push_str("### Rule Inventory\n\n");
    out.push_str("| Rule ID | Name | Category | Severity | Default Profiles |\n");
    out.push_str("| --- | --- | --- | --- | --- |\n");
    for item in inventory {
        out.push_str(&inventory_row(&item));
    }
    out.push('\n');
    out.push_str(MANAGED_END);
    out.push('\n');
    out
}

fn inventory_row(item: &RuleInventoryItem) -> String {
    format!(
        "| `{}` | {} | `{:?}` | `{:?}` | `{}` |\n",
        item.rule_id,
        item.name,
        item.category,
        item.severity,
        item.default_profiles.join(", ")
    )
}

fn managed_block_range(content: &str) -> Option<(usize, usize)> {
    let start = content.find(MANAGED_START)?;
    let end_marker = content.find(MANAGED_END)?;
    Some((start, end_marker + MANAGED_END.len()))
}

#[cfg(test)]
mod tests {
    use super::{build_managed_block, merge_agents_content};

    #[test]
    fn merge_agents_content_creates_new_file_when_missing() {
        let block = build_managed_block();
        let merged = merge_agents_content(None, &block);

        assert!(merged.starts_with("# AGENTS.md"));
        assert!(merged.contains("## verifyOS-cli"));
        assert!(merged.contains("RULE_PRIVACY_MANIFEST"));
    }

    #[test]
    fn merge_agents_content_replaces_existing_managed_block() {
        let block = build_managed_block();
        let existing = r#"# AGENTS.md

Custom note

<!-- verifyos-cli:agents:start -->
old block
<!-- verifyos-cli:agents:end -->

Keep this
"#;

        let merged = merge_agents_content(Some(existing), &block);

        assert!(merged.contains("Custom note"));
        assert!(merged.contains("Keep this"));
        assert!(!merged.contains("old block"));
        assert_eq!(
            merged.matches("<!-- verifyos-cli:agents:start -->").count(),
            1
        );
    }
}
