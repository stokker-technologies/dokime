# Release Asset Requirements

## ADDED Requirements

### Requirement: Product Release Inputs Match The Generic Layout

Dokime SHALL declare native tool archives, per-target shipped host-plugin bundles, native installers and portable payload ZIPs under the locked generic layout. OBC, examples, Python distributions and standalone signature files SHALL NOT be release downloads.

#### Scenario: Stage Product Downloads

- **WHEN** matching native package inputs are staged for a release
- **THEN** tool names use Rust ABI targets and native archive suffixes, payload names begin with `dokime-payload-`, and plugins.json and upload.json describe the selected shipped plugin downloads

### Requirement: Published Installer Verification Uses The Embedded Signature

Dokime SHALL verify signed installer bytes against the compiled updater key using the signature embedded in latest.json when a standalone sidecar is not downloaded.

#### Scenario: Verify A Published Windows Installer

- **WHEN** the release download directory contains its installer and latest.json without an .exe.sig sidecar
- **THEN** verification checks the manifest version, exact asset URL name and cryptographic signature against that installer

### Requirement: Preserve Owner Validation Holds

Adoption SHALL preserve the existing Check/Test job holds and enabled Release / Dry, and SHALL use credential-free release rehearsal.

#### Scenario: Inspect The Adopted Workflow

- **WHEN** product configuration is adapted
- **THEN** Check/Test hold conditions remain byte-for-byte unchanged and Release / Dry stays enabled
