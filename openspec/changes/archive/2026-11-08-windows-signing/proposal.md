## Why

Enterprise deployment requires Windows publisher verification and optional machine-wide installation while retaining unsigned fork builds and the existing updater trust contract.

## What Changes

- Sign staged Windows executables and the completed installer before minisign, with Azure Artifact Signing or a secret-backed PFX, RFC 3161 timestamps and verification.
- Preserve the installer manifest and current-user default; add explicit machine-wide NSIS selection.
- Document managed deployment, network requirements, Linux verification and exact workflow wiring.

## Capabilities

### New Capabilities

- `windows-deployment`: Optional verified Authenticode and explicit machine-wide deployment.

### Modified Capabilities

None.

## Impact

Only packaging, its tests, the sign-windows composite action and deployment documentation change. Release/nightly workflow integration belongs to the orchestrator. No certificate purchase, release, push or repository signing is authorized.

## Authorization

The orchestrator approved this scope and the pre-minisign signing hook on 2026-11-08. Existing restored OpenSpec changes remain inactive.
