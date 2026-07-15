# Security Policy

## Supported versions

Only the latest commit on `main` is supported. Builds are internal beta software until they are signed and the release checklist in the README is complete.

## Reporting

Report vulnerabilities privately to the repository owner. Do not include private images, full local paths, usernames, signing material, or other secrets. Include the affected version, minimal reproduction steps, and expected impact.

## Security boundaries

The app processes untrusted local image files. Tauri capabilities are restricted to the main window, event/path APIs, window show/focus, and the folder/file dialog. Arbitrary frontend filesystem permissions are not granted. Local paths are opened through a validated Rust command.

Release signing secrets must never be exposed to pull-request workflows. Public releases require protected secrets, manual approval, signature verification, and timestamping.
