# verifyOS Backend (Rust)

This service provides a clean, versioned HTTP API that accepts an `.ipa` or `.app` upload and returns a normalized scan report.

## Quick Start

From the repo root:

```bash
cargo run --manifest-path apps/backend/Cargo.toml
```

The API listens on `http://127.0.0.1:7070` by default.

Auth (optional):

```bash
export REQUIRE_AUTH=true
export AUTH_DEV_MODE=true
```

Example request:

```bash
curl -X POST http://127.0.0.1:7070/api/v1/scan \
  -F "bundle=@/path/to/YourApp.ipa" \
  -F "profile=full"
```

Add include/exclude rules, baseline suppression, and alternate formats:

```bash
curl -X POST http://127.0.0.1:7070/api/v1/scan \
  -F "bundle=@/path/to/YourApp.ipa" \
  -F "profile=basic" \
  -F "include=RULE_PRIVACY_MANIFEST,RULE_USAGE_DESCRIPTIONS" \
  -F "exclude=RULE_PRIVATE_API" \
  -F "baseline=@/path/to/baseline.json" \
  -F "format=markdown"
```

Auth flow (dev mode prints code in response):

```bash
curl -X POST http://127.0.0.1:7070/api/v1/auth/start \
  -H "Content-Type: application/json" \
  -d '{"email":"you@company.com"}'

curl -X POST http://127.0.0.1:7070/api/v1/auth/verify \
  -H "Content-Type: application/json" \
  -d '{"email":"you@company.com","code":"ABC123"}'
```

Include project context (zip a `.xcodeproj` or `.xcworkspace`):

```bash
zip -r YourProject.zip YourProject.xcodeproj

curl -X POST http://127.0.0.1:7070/api/v1/scan \
  -F "bundle=@/path/to/YourApp.ipa" \
  -F "project=@/path/to/YourProject.zip" \
  -F "profile=full"
```

Download a full agent handoff bundle:

```bash
curl -X POST http://127.0.0.1:7070/api/v1/handoff \
  -F "bundle=@/path/to/YourApp.ipa" \
  -F "project=@/path/to/YourProject.zip" \
  -F "profile=full" \
  -o verifyos-handoff.zip

unzip -q verifyos-handoff.zip
bash apply-handoff.sh /path/to/project/root
```

## API

`POST /api/v1/scan`

- multipart `bundle` file field (required)
- `profile` form field: `basic` or `full` (optional)
- `include` form field: comma-separated rule IDs (optional)
- `exclude` form field: comma-separated rule IDs (optional)
- `baseline` file field: JSON report from a previous run (optional)
- `format` form field: `json`, `sarif`, or `markdown` (optional)
- `project` zip field with `.xcodeproj` or `.xcworkspace` (optional)

Response: JSON report (same shape as `voc --format json`) or text output for `sarif`/`markdown`.

`POST /api/v1/auth/start`

- JSON body: `{ "email": "you@company.com" }`

Response: `{ "status": "sent", "dev_code": "ABC123" }` (dev mode only).

`POST /api/v1/auth/verify`

- JSON body: `{ "email": "you@company.com", "code": "ABC123" }`

Response: `{ "token": "...", "email": "...", "expires_in_seconds": 86400 }`.

`POST /api/v1/handoff`

- multipart `bundle` file field (required)
- `profile` form field: `basic` or `full` (optional)
- `include` form field: comma-separated rule IDs (optional)
- `exclude` form field: comma-separated rule IDs (optional)
- `baseline` file field: JSON report from a previous run (optional)
- `project` zip field with `.xcodeproj` or `.xcworkspace` (optional)

Response: `verifyos-handoff.zip` containing `.verifyos/`, `AGENTS.md`, and `apply-handoff.sh`.

## Notes

- This module depends on the root `verifyOS` crate (`verifyos-cli`).
- It is initialized as a standalone crate for a future Cargo workspace split.
