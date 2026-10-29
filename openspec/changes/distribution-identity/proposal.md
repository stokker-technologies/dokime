# Distribution Identity

## Why

Branded distributions reuse unchanged Dokime binaries, so every installation stores its data under the compiled Tauri identifier (`%LOCALAPPDATA%\com.stokker.dokime\…`). A company distribution (for example Hilti) therefore shares directories with plain Dokime and cannot be installed beside it. Organization packs override appearance only; `organization-installers` deliberately kept storage identity compiled.

## What Changes

Locked by the owner on 2026-10-29; deferred (not scheduled yet).

- The distributor's installer manifest declares the distribution identifier (`[installer] identifier = "com.hilti.dokime"`). It becomes the installation's storage and install identity.
- `cargo dokime installer package` writes the identifier into an installed distribution file beside the executable and uses it for the NSIS install folder and uninstall registry keys and for DEB/RPM package names.
- The desktop resolves every directory from the installed distribution identity before anything else starts: application, data, plugins, logs and the WebView2 profile. CLI and server defaults use the same lookup.
- Without a declared identifier, the compiled identifier applies (plain Dokime).
- Supersedes the `organization-installers` decision that storage identity stays compiled; organization packs themselves stay appearance-only.

## Capabilities

### New Capabilities

- `distribution-identity`: install-time storage and install identity for branded distributions.

## Impact

`dokime-packaging` (manifest, payload identity validation, NSIS/DEB/RPM assembly), the desktop bootstrap (`app/organization.rs`, `app/logging.rs`, WebView data directory), CLI and server path defaults, the product guide's Native Installers and Organization sections, and native lifecycle tests.
