# Retire Installer Inspection

## Why

The owner decided on 2026-10-30 to use Tauri/NSIS installer defaults and retire native installer inspection, including `cargo dokime installer inspect`. The inspector forced non-solid compression and duplicated the bundler's responsibility.

## What Changes

- Restore the stock `SetCompressor /SOLID "{{compression}}"` behavior and Tauri's default compression setting. Preserve Dokime's display label, identifier separation and CLI PATH hooks.
- Delete native installer parsing, the inspect CLI command, post-build pipeline inspection, inspector-only dependencies and tests, and obsolete guidance.
- Preserve input verification, staged artwork customization, signing and signature verification, startup identity, and the `installer.json` inventory consumed by bundled-plugin startup.
- Update the organization-installers specification and archive this change with OpenSpec, retaining the archive at the owner's explicit request.

## Capabilities

### Modified Capabilities

- `organization-installers`: Native assembly uses bundler defaults without a package inspection command or post-build content inspection.

## Impact

Owners are packaging, cargo-dokime, the Python packaging adapter, installer tests, product guidance and OpenSpec. README claims and Dokime skills will be reviewed; unaffected text stays unchanged. Proof is limited to workspace check, Rust formatting, remaining installer unit tests and strict OpenSpec validation. No installer build, packaging, push or PR is authorized.
