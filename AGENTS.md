# AGENTS.md

# Rust / verifyOS-cli

Guidelines for developing the `verifyOS-cli` App Store rejection risk scanner.

## Core Principles

- **"Shift Left" Validation**: Catch App Store Connect rejections as early as possible on the developer's local machine or CI runner.
- **Local Analysis**: Do not transmit app bundles to external servers. All analysis must be performed locally using Rust.
- **Zero Placeholders**: Implement actual business logic for App Store rules. If a rule is requested, implement the core engine logic or use established patterns.

## Code Style & Conventions

- **Inlined Variables**: Always inline variables into `format!` strings (`format!("{var}")`) where possible.
- **Clippy Compliance**: 
    - Always collapse `if` statements when possible.
    - Use method references over closures when possible (`.map(|x| x.foo())` -> `.map(X::foo)`).
- **Match Statements**: Ensure `match` statements are exhaustive. Avoid wildcard arms (`_`) if the set of variants is known and stable.
- **Modules**:
    - Target module files under 500 lines of code.
    - Follow the established directory structure:
        - `core/`: Orchestration and top-level pipeline logic (Engine).
        - `parsers/`: File format handles (`zip`, `plist`, `macho`).
        - `rules/`: Trait-based validation logic (`AppStoreRule`).

## Rule Engine & Traits

- **Trait Usage**: All validation rules must implement the `AppStoreRule` trait found in `src/rules/core.rs`.
- **Context Awareness**: Use the `ArtifactContext` to access `Info.plist` data and the app bundle path during evaluation.
- **Error Handling**: 
    - Use `miette` for user-facing diagnostics. 
    - Use `thiserror` for internal rule-specific errors (`EntitlementsError`, `PlistError`).
    - Use absolute `verifyos_cli::` paths for cross-module imports.

## Data Processing Rules

- **IPAs/App Bundles**: Treat `.ipa` files as ZIP archives. Extract to temporary directories and clean up after analysis.
- **Info.plist**: Use the `plist` crate for robust parsing. Map keys to structured `InfoPlist` structs in `src/parsers/plist_reader.rs`.
- **Mach-O**: Use `goblin` or `apple-codesign` for deep binary inspection (entitlements, architectures).

## Testing

- **Integration Tests**: Place major test flows in `tests/cli_test.rs` and `tests/rules_test.rs`.
- **Fixtures**: Use the `.ipa` and `.app` fixtures in `examples/` and `tests/fixtures/`.
- **Path Resolution**: Use `CARGO_MANIFEST_DIR` base paths to locate test fixtures reliably across different environments.

## Tooling

- **Cargo**: Standard commands: `cargo fmt`, `cargo clippy`, `cargo test`.
- **Release**: Follow conventional commits (`feat:`, `fix:`, `chore:`, `docs:`) to facilitate the `release-plz` workflow.

---

## Memory System: cass-memory

The Cass Memory System (cm) is a tool for giving agents an effective memory based on the ability to quickly search across previous coding agent sessions across an array of different coding agent tools (e.g., Claude Code, Codex, Gemini-CLI, Cursor, etc.) and projects (and even across multiple machines, optionally) and then reflect on what they find and learn in new sessions to draw out useful lessons and takeaways; these lessons are then stored and can be queried and retrieved later, much like how human memory works.

The `cm onboard` command guides you through analyzing historical sessions and extracting valuable rules.

### Quick Start

```bash
# 1. Check status and see recommendations
cm onboard status

# 2. Get sessions to analyze (filtered by gaps in your playbook)
cm onboard sample --fill-gaps

# 3. Read a session with rich context
cm onboard read /path/to/session.jsonl --template

# 4. Add extracted rules (one at a time or batch)
cm playbook add "Your rule content" --category "debugging"
# Or batch add:
cm playbook add --file rules.json

# 5. Mark session as processed
cm onboard mark-done /path/to/session.jsonl
```

Before starting complex tasks, retrieve relevant context:

```bash
cm context "<task description>" --json
```

This returns:
- **relevantBullets**: Rules that may help with your task
- **antiPatterns**: Pitfalls to avoid
- **historySnippets**: Past sessions that solved similar problems
- **suggestedCassQueries**: Searches for deeper investigation

### Protocol

1. **START**: Run `cm context "<task>" --json` before non-trivial work
2. **WORK**: Reference rule IDs when following them (e.g., "Following b-8f3a2c...")
3. **FEEDBACK**: Leave inline comments when rules help/hurt:
   - `// [cass: helpful b-xyz] - reason`
   - `// [cass: harmful b-xyz] - reason`
4. **END**: Just finish your work. Learning happens automatically.

### Key Flags

| Flag | Purpose |
|------|---------|
| `--json` | Machine-readable JSON output (required!) |
| `--limit N` | Cap number of rules returned |
| `--no-history` | Skip historical snippets for faster response |

stdout = data only, stderr = diagnostics. Exit 0 = success.

## Repo Rules

- Keep commits small. Prefer one behavioral change per commit.
- Split work into separate commits when it improves reviewability:
  - `feat:` for behavior
  - `test:` for meaningful test additions or coverage changes
  - `docs:` for README, architecture, examples, or agent-playbook updates
  - `chore(ci):` for workflow or automation changes
- Do not bundle unrelated code, tests, docs, and CI changes into one large commit unless they must ship together for correctness.
- Before every push, run:
  - `cargo fmt --all -- --check`
  - `cargo test`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- Keep conventional commit messages concise and scope-specific.

<!-- verifyos-cli:agents:start -->
## verifyOS-cli

Use `voc` before large iOS submission changes or release builds.

### Recommended Workflow

1. Run `voc --app <path-to-.ipa-or-.app> --profile basic` for a quick gate.
2. Run `voc --app <path-to-.ipa-or-.app> --profile full --agent-pack ./.verifyos-agent --agent-pack-format bundle` before release or when an AI agent will patch findings.
3. Read `./.verifyos-agent/agent-pack.md` first, then patch the highest-priority scopes.
4. Re-run `voc` after each fix batch until the pack is clean.

### AI Agent Rules

- Prefer `voc --profile basic` during fast inner loops and `voc --profile full` before shipping.
- When findings exist, generate an agent bundle with `voc --agent-pack ./.verifyos-agent --agent-pack-format bundle`.
- Fix `high` priority findings before `medium` and `low`.
- Treat `Info.plist`, `entitlements`, `ats-config`, and `bundle-resources` as the main fix scopes.
- Re-run `voc` after edits and compare against the previous agent pack to confirm findings were actually removed.

### Rule Inventory

| Rule ID | Name | Category | Severity | Default Profiles |
| --- | --- | --- | --- | --- |
| `RULE_ATS_AUDIT` | ATS Exceptions Detected | `Ats` | `Warning` | `basic, full` |
| `RULE_ATS_GRANULARITY` | ATS Exceptions Too Broad | `Ats` | `Warning` | `basic, full` |
| `RULE_BUNDLE_METADATA_CONSISTENCY` | Bundle Metadata Consistency | `Metadata` | `Warning` | `full` |
| `RULE_BUNDLE_RESOURCE_LEAKAGE` | Sensitive Files in Bundle | `Bundling` | `Error` | `full` |
| `RULE_CAMERA_USAGE` | Missing Camera Usage Description | `Permissions` | `Error` | `basic, full` |
| `RULE_DEVICE_CAPABILITIES_AUDIT` | UIRequiredDeviceCapabilities Audit | `Metadata` | `Warning` | `full` |
| `RULE_EMBEDDED_TEAM_ID_MISMATCH` | Embedded Team ID Mismatch | `Signing` | `Error` | `basic, full` |
| `RULE_ENTITLEMENTS_MISMATCH` | Debug Entitlements Present | `Entitlements` | `Error` | `basic, full` |
| `RULE_ENTITLEMENTS_PROVISIONING_MISMATCH` | Entitlements vs Provisioning Mismatch | `Entitlements` | `Error` | `basic, full` |
| `RULE_EXPORT_COMPLIANCE` | Export Compliance Declaration | `Metadata` | `Warning` | `full` |
| `RULE_EXTENSION_ENTITLEMENTS_COMPAT` | Extension Entitlements Compatibility | `Entitlements` | `Warning` | `full` |
| `RULE_INFO_PLIST_CAPABILITIES_EMPTY` | Empty Info.plist Capability Lists | `Metadata` | `Warning` | `full` |
| `RULE_INFO_PLIST_REQUIRED_KEYS` | Missing Required Info.plist Keys | `Metadata` | `Warning` | `full` |
| `RULE_INFO_PLIST_VERSIONING` | Info.plist Versioning Consistency | `Metadata` | `Warning` | `full` |
| `RULE_LSAPPLICATIONQUERIES_SCHEMES_AUDIT` | LSApplicationQueriesSchemes Audit | `Metadata` | `Warning` | `full` |
| `RULE_PRIVACY_MANIFEST` | Missing Privacy Manifest | `Privacy` | `Error` | `basic, full` |
| `RULE_PRIVACY_MANIFEST_COMPLETENESS` | Privacy Manifest Completeness | `Privacy` | `Warning` | `full` |
| `RULE_PRIVACY_SDK_CROSSCHECK` | Privacy Manifest vs SDK Usage | `Privacy` | `Warning` | `full` |
| `RULE_PRIVATE_API` | Private API Usage Detected | `ThirdParty` | `Warning` | `full` |
| `RULE_USAGE_DESCRIPTIONS` | Missing Usage Description Keys | `Privacy` | `Warning` | `basic, full` |
| `RULE_USAGE_DESCRIPTIONS_EMPTY` | Empty Usage Description Values | `Privacy` | `Warning` | `basic, full` |

<!-- verifyos-cli:agents:end -->
