# Images Optimizer

Local desktop image compression and conversion for Windows. The app accepts JPEG and PNG files, can optimize the original format, and can create WebP and AVIF outputs.

> Status: internal unsigned beta. Do not publish the installer as a production release until Windows Authenticode signing and the manual failure tests below are complete.

## Supported platform and formats

- Windows 10 and Windows 11 only.
- Inputs: `.jpg`, `.jpeg`, `.png`.
- Outputs: optimized JPEG/PNG, WebP, and AVIF.
- JPEG, PNG, WebP, and AVIF quality is lossy. PNG output is quantized with pngquant and then optimized with Oxipng.
- Encoded outputs remove metadata. JPEG EXIF orientation is applied to pixels before metadata is removed.

## Data safety

Every output is written to a temporary file in the destination directory, flushed, decoded, checked for format and dimensions, and only then committed. Existing files default to a unique-name policy. Overwrite requires explicit confirmation and keeps a temporary backup until replacement succeeds.

The app plans every destination before parallel work starts. Same-stem collisions, collisions with selected sources, existing destinations, duplicate tasks, and unsafe nested custom output folders are rejected, skipped, or renamed according to the selected policy.

Completed safe outputs remain after cancellation. Work not yet started is reported as canceled; temporary files are removed.

## Limits

- 10,000 input files and 10 GiB total input per run.
- Maximum width or height: 50,000 pixels.
- At most four worker threads.
- External codec timeout: 120 seconds.
- Custom output and input directories cannot contain one another.
- Symlinks are not followed.
- Outputs that are not smaller are skipped.

## Privacy

Processing is local. The app does not upload images or analytics. Debug output must not be shared without reviewing file paths and error text.

Production logs contain operation IDs, counts, codec names, and error codes without source paths. Three 1 MB log files are retained in the platform application log directory. The result modal can copy a redacted diagnostic summary.

## Install

There is no public production installer yet. Draft GitHub releases are unsigned prerelease builds and may trigger Windows SmartScreen. Verify `SHA256SUMS.txt` before installing.

## Development

Requirements:

- Node.js 22+
- Yarn 1.22.22
- Rust 1.88 or newer stable toolchain
- Windows SDK and WebView2/Tauri build prerequisites

```powershell
yarn install --frozen-lockfile
yarn lint
yarn stylelint
yarn typecheck
yarn test
yarn generate

cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo build --release --manifest-path src-tauri/Cargo.toml

yarn tauri:build
```

Use `yarn tauri:dev` for local desktop development.

## Versioning and release

From a clean, up-to-date `main`, choose the SemVer increment:

```powershell
yarn release patch
yarn release minor
yarn release major
```

The command synchronizes package, Cargo, and Tauri versions, commits the bump, and atomically pushes `main` with the matching `vX.Y.Z` tag. The tag creates a draft, prerelease Windows build and SHA-256 checksums. Before a public production release:

1. Configure and protect a Windows code-signing certificate and timestamp service.
2. Verify Authenticode signatures in CI.
3. Smoke-test install, upgrade, uninstall, and Windows 10/11 startup.
4. Complete disk-full, locked-destination, cancellation, and keyboard-only tests.
5. Change the release workflow and this document only after those gates pass.

## Bundled software

The Windows build embeds pngquant 2.17.0 under GPL-3.0-or-later and Oxipng 10.0.0 under MIT. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for license and source details.

## Known limitations

- Windows installers are unsigned and beta-only.
- ICC profile preservation is not implemented.
- Metadata is removed rather than configurable.
- `cargo audit` ignores RUSTSEC-2026-0194 and RUSTSEC-2026-0195 because they occur only in the inactive Linux `wayland-scanner` build dependency. Remove the exceptions when upstream adopts `quick-xml` 0.41 or newer.
- Disk-full, locked-file, abrupt-process-kill, and packaged-app accessibility tests still require manual Windows validation.
- No screenshots are maintained in the repository yet.

Security reports: see [SECURITY.md](SECURITY.md). Contributions: see [CONTRIBUTING.md](CONTRIBUTING.md).
