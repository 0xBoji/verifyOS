# verifyOS VS Code Extension

This extension is intentionally thin. It starts the existing `voc lsp` binary so the Rust CLI remains the source of truth for diagnostics.

When `voc lsp` publishes findings for `Info.plist` or `PrivacyInfo.xcprivacy`, they appear in VS Code's Problems pane like any other language server diagnostic.

## Requirements

- `voc` installed and available on `PATH`, or a custom path set via `verifyOS.path`
- A workspace containing `Info.plist`, `.plist`, or `.xcprivacy` files

## Development

```bash
cd editors/vscode
npm ci
npm run compile
```

Press `F5` in VS Code to launch an Extension Development Host.

## Packaging and publishing

```bash
cd editors/vscode
npm ci
npm run package
```

The repository also includes `.github/workflows/vscode-extension.yml`, which packages a `.vsix` on tags and can publish to the VS Code Marketplace and Open VSX when `VSCE_PAT` and `OVSX_PAT` are configured.
