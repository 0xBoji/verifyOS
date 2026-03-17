---
description: How to manage and monitor GitHub Actions CI/CD workflows
---

// turbo-all
# CI/CD Workflows Management

This workflow covers the automated pipelines defined in `.github/workflows/`.

## Core Pipelines

### 1. Rust CI (`rust.yml`)
- **Purpose**: Runs on every push and PR to main branches.
- **Actions**:
  - `cargo fmt`: Checks code formatting.
  - `cargo clippy`: Runs linting with strict warnings.
  - `cargo test`: Executes unit and integration tests across Ubuntu, macOS, and Windows.

### 2. VS Code Extension (`vscode-extension.yml`)
- **Purpose**: Automates the building, packaging, and publishing of the VS Code extension.
- **Actions**:
  - Builds bundled binaries for multiple platforms (Linux, Windows, macOS x64/arm64).
  - Packages the `.vsix` extension.
  - Publishes to VS Code Marketplace and Open VSX when a version tag (e.g., `v*`) is pushed.

### 3. Release Automation (`release-plz.yml` / `release.yml`)
- **Purpose**: Manages crate releases and automated changelog generation using `release-plz`.

## Monitoring and Debugging

1. **Check Status**:
   View the **Actions** tab on GitHub to see the status of all runs.

2. **Local Linting (Pre-commit)**:
   It is recommended to run these commands before pushing to avoid CI failures:
   `cargo fmt --all`
   `cargo clippy --all-targets --all-features -- -D warnings`
   `cargo test`

3. **Manual Trigger**:
   The `VS Code Extension` workflow can be triggered manually via `workflow_dispatch` if needed.
