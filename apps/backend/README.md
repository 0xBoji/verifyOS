# verifyOS Backend (Rust)

This service provides a clean, versioned HTTP API that accepts an `.ipa` or `.app` upload and returns a normalized scan report.

## API

`POST /api/v1/scan`

- multipart `bundle` file field (required)
- `profile` form field: `basic` or `full` (optional)

Response: JSON report (same shape as `voc --format json`).

## Notes

- This module depends on the root `verifyos-cli` crate.
- It is initialized as a standalone crate for a future Cargo workspace split.
