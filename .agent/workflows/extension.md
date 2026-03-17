---
description: How to build and publish the verifyOS VS Code Extension
---

// turbo-all
# VS Code Extension Management Workflow

This workflow covers development and publishing for the verifyOS-vscode extension located in `editors/vscode`.

## Local Development

1. Navigate to the extension directory:
   `cd editors/vscode`

2. Install dependencies:
   `npm install`

3. Compile the extension:
   `npm run compile`

4. To test, open the directory in VS Code and press `F5` to launch the Extension Development Host.

## Packaging and Publishing

1. Packaging the extension (`.vsix`):
   `npm run package`

2. Publishing to VS Code Marketplace:
   `npm run publish:vsce`

3. Publishing to Open VSX:
   `npm run publish:ovsx`
