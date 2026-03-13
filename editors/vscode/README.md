# verifyOS for VS Code

Catch App Store submission risks earlier with a clean VS Code experience for `Info.plist` and `PrivacyInfo.xcprivacy`.

`verifyOS` keeps diagnostics local, highlights findings in the **Problems** pane, and adds a lightweight **Action Center** in the sidebar so you can:

- see whether the language server is running
- scan the current bundle on demand
- generate a handoff bundle for AI agents
- jump straight to the verifyOS output log

## What you get

- Real App Store-focused diagnostics powered by the Rust `voc` engine
- A sidebar icon and Action Center instead of a purely invisible LSP
- Zero-config startup on supported Marketplace builds via a bundled `voc` binary
- A fallback `verifyOS.path` setting when you want to bring your own CLI install

## How it works

When you open `Info.plist` or `PrivacyInfo.xcprivacy`, verifyOS runs in the background and publishes diagnostics into VS Code's standard UI:

- red or yellow underlines in the editor
- the **Problems** panel (`Cmd/Ctrl + Shift + M`)
- the verifyOS sidebar for quick actions

This extension does not upload your app bundle anywhere. The rule engine stays inside the local Rust binary.

## Quick start

1. Install the `verifyOS` extension.
2. Open a workspace that contains an `.app` bundle with `Info.plist` or `PrivacyInfo.xcprivacy`.
3. Click the **verifyOS** icon in the activity bar.
4. Open one of those files to wake diagnostics and check the **Problems** pane.

If your build does not bundle `voc`, install it yourself:

```bash
cargo install verifyos-cli
```

## Action Center

The sidebar includes:

- **Language server status** so you can tell whether verifyOS is running
- **Scan current bundle** to run `voc` against the active `.app`
- **Generate handoff bundle** to create `.verifyos/` outputs for AI agent workflows
- **Open Problems**, **Show Output**, and **Restart language server**

## Settings

### `verifyOS.useBundledBinary`

Prefer the bundled `voc` binary that ships inside the extension when one is available for the current platform.

### `verifyOS.path`

Fallback path to the `voc` binary. Use an absolute path if `voc` is not on `PATH`.

### `verifyOS.profile`

Rule profile passed to `voc lsp`.

- `basic`
- `full`

### `verifyOS.outputDir`

Workspace-relative folder where **Generate handoff bundle** writes `.verifyos` outputs.

### `verifyOS.trace.server`

Trace level for the language server.

- `off`
- `messages`
- `verbose`

## Notes

- The extension is most useful when you are editing files inside a built `.app` bundle.
- Diagnostics are surfaced through VS Code's native Problems UI, so you may not see a large custom window even while verifyOS is working.
- Maintainer-oriented packaging and publishing details live in the main repository docs.
