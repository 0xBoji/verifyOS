use crate::agent_assets::AgentAssetLayout;
use crate::agents::{render_fix_prompt, render_pr_brief, render_pr_comment, CommandHints};
use crate::report::{render_agent_pack_markdown, AgentPack, AgentPackFormat};
use miette::{IntoDiagnostic, Result};
use std::path::Path;

pub fn write_next_steps_script(path: &Path, hints: &CommandHints) -> Result<()> {
    let Some(app_path) = hints.app_path.as_deref() else {
        return Err(miette::miette!(
            "`--shell-script` requires `--from-scan <path>` so voc can build the follow-up commands"
        ));
    };

    let profile = hints.profile.as_deref().unwrap_or("full");
    let agent_pack_dir = hints.agent_pack_dir.as_deref().unwrap_or(".verifyos-agent");

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).into_diagnostic()?;
    }

    let mut script = String::new();
    script.push_str("#!/usr/bin/env bash\nset -euo pipefail\n\n");
    script.push_str(&format!(
        "voc --app {} --profile {}\n",
        shell_quote(app_path),
        profile
    ));
    script.push_str(&format!(
        "voc --app {} --profile {} --format json > report.json\n",
        shell_quote(app_path),
        profile
    ));
    script.push_str(&format!(
        "voc --app {} --profile {} --agent-pack {} --agent-pack-format bundle\n",
        shell_quote(app_path),
        profile,
        shell_quote(agent_pack_dir)
    ));
    if let Some(output_dir) = hints.output_dir.as_deref() {
        let mut cmd = format!(
            "voc doctor --output-dir {} --fix --from-scan {} --profile {}",
            shell_quote(output_dir),
            shell_quote(app_path),
            profile
        );
        if let Some(baseline) = hints.baseline_path.as_deref() {
            cmd.push_str(&format!(" --baseline {}", shell_quote(baseline)));
        }
        if hints.pr_brief_path.is_some() {
            cmd.push_str(" --open-pr-brief");
        }
        if hints.pr_comment_path.is_some() {
            cmd.push_str(" --open-pr-comment");
        }
        script.push_str(&format!("{cmd}\n"));
    } else if let Some(baseline) = hints.baseline_path.as_deref() {
        script.push_str(&format!(
            "voc init --from-scan {} --profile {} --baseline {} --agent-pack-dir {} --write-commands --shell-script\n",
            shell_quote(app_path),
            profile,
            shell_quote(baseline),
            shell_quote(agent_pack_dir)
        ));
    } else {
        script.push_str(&format!(
            "voc init --from-scan {} --profile {} --agent-pack-dir {} --write-commands --shell-script\n",
            shell_quote(app_path),
            profile,
            shell_quote(agent_pack_dir)
        ));
    }

    std::fs::write(path, script).into_diagnostic()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path).into_diagnostic()?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms).into_diagnostic()?;
    }
    Ok(())
}

pub fn write_fix_prompt_file(path: &Path, pack: &AgentPack, hints: &CommandHints) -> Result<()> {
    write_text_asset(path, &render_fix_prompt(pack, hints))
}

pub fn write_pr_brief_file(path: &Path, pack: &AgentPack, hints: &CommandHints) -> Result<()> {
    write_text_asset(path, &render_pr_brief(pack, hints))
}

pub fn write_pr_comment_file(path: &Path, pack: &AgentPack, hints: &CommandHints) -> Result<()> {
    write_text_asset(path, &render_pr_comment(pack, hints))
}

pub fn write_agent_pack(
    path: &Path,
    agent_pack: &AgentPack,
    format: AgentPackFormat,
) -> Result<()> {
    match format {
        AgentPackFormat::Json => {
            let json = serde_json::to_string_pretty(agent_pack).into_diagnostic()?;
            std::fs::write(path, json).into_diagnostic()?;
        }
        AgentPackFormat::Markdown => {
            let markdown = render_agent_pack_markdown(agent_pack);
            std::fs::write(path, markdown).into_diagnostic()?;
        }
        AgentPackFormat::Bundle => {
            std::fs::create_dir_all(path).into_diagnostic()?;
            let json_path = path.join("agent-pack.json");
            let markdown_path = path.join("agent-pack.md");
            let json = serde_json::to_string_pretty(agent_pack).into_diagnostic()?;
            let markdown = render_agent_pack_markdown(agent_pack);
            std::fs::write(json_path, json).into_diagnostic()?;
            std::fs::write(markdown_path, markdown).into_diagnostic()?;
        }
    }

    Ok(())
}

pub fn load_agent_pack(path: &Path) -> Option<AgentPack> {
    if !path.exists() {
        return None;
    }

    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn empty_agent_pack() -> AgentPack {
    AgentPack {
        generated_at_unix: 0,
        total_findings: 0,
        findings: Vec::new(),
    }
}

pub fn infer_existing_command_hints(layout: &AgentAssetLayout) -> CommandHints {
    let mut hints = CommandHints {
        output_dir: Some(layout.output_dir.display().to_string()),
        app_path: None,
        baseline_path: None,
        agent_pack_dir: Some(layout.agent_bundle_dir.display().to_string()),
        profile: None,
        shell_script: layout.next_steps_script_path.exists(),
        fix_prompt_path: Some(layout.fix_prompt_path.display().to_string()),
        repair_plan_path: layout
            .repair_plan_path
            .exists()
            .then(|| layout.repair_plan_path.display().to_string()),
        pr_brief_path: layout
            .pr_brief_path
            .exists()
            .then(|| layout.pr_brief_path.display().to_string()),
        pr_comment_path: layout
            .pr_comment_path
            .exists()
            .then(|| layout.pr_comment_path.display().to_string()),
    };

    for command in collect_existing_voc_commands(layout) {
        let tokens = split_shell_words(&command);
        if tokens.first().map(String::as_str) != Some("voc") {
            continue;
        }

        let mut index = 1;
        while index < tokens.len() {
            match tokens[index].as_str() {
                "--app" | "--from-scan" => {
                    if hints.app_path.is_none() {
                        hints.app_path = tokens.get(index + 1).cloned();
                    }
                    index += 1;
                }
                "--profile" => {
                    if hints.profile.is_none() {
                        hints.profile = tokens.get(index + 1).cloned();
                    }
                    index += 1;
                }
                "--baseline" => {
                    if hints.baseline_path.is_none() {
                        hints.baseline_path = tokens.get(index + 1).cloned();
                    }
                    index += 1;
                }
                "--shell-script" => {
                    hints.shell_script = true;
                }
                "--open-pr-brief" => {
                    hints.pr_brief_path = Some(layout.pr_brief_path.display().to_string());
                }
                "--open-pr-comment" => {
                    hints.pr_comment_path = Some(layout.pr_comment_path.display().to_string());
                }
                _ => {}
            }
            index += 1;
        }
    }

    hints
}

fn write_text_asset(path: &Path, contents: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).into_diagnostic()?;
    }
    std::fs::write(path, contents).into_diagnostic()?;
    Ok(())
}

fn collect_existing_voc_commands(layout: &AgentAssetLayout) -> Vec<String> {
    let mut commands = Vec::new();

    if let Ok(contents) = std::fs::read_to_string(&layout.agents_path) {
        commands.extend(
            contents
                .lines()
                .map(str::trim)
                .filter(|line| line.starts_with("voc "))
                .map(str::to_string),
        );
    }

    if let Ok(contents) = std::fs::read_to_string(&layout.next_steps_script_path) {
        commands.extend(
            contents
                .lines()
                .map(str::trim)
                .filter(|line| line.starts_with("voc "))
                .map(str::to_string),
        );
    }

    commands
}

fn split_shell_words(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single_quote = false;

    for ch in input.chars() {
        match ch {
            '\'' => in_single_quote = !in_single_quote,
            ' ' | '\t' if !in_single_quote => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "/._-".contains(ch))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}
