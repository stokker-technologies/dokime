# release-assets Specification

## Purpose
Define native distribution and tool archives, explicit shipping plugin inputs, signed installer verification and credential-free release validation.

## Requirements

### Requirement: Product Release Inputs Match The Generic Layout

Dokime SHALL declare native tool archives, per-kind shipping plugin archives, native installers and portable payload ZIPs under the locked generic layout. OBC, examples, Python distributions and standalone signature files SHALL NOT be release downloads.

#### Scenario: Stage Product Downloads

- **WHEN** matching native package inputs are staged for a release
- **THEN** tool names use Rust ABI targets and native archive suffixes, distribution input names begin with `dokime-distribution-`, and plugins.json and upload.json describe the selected shipped plugin downloads

### Requirement: Published Installer Verification Uses The Embedded Signature

Dokime SHALL verify signed installer bytes against the compiled updater key using the signature embedded in latest.json when a standalone sidecar is not downloaded.

#### Scenario: Verify A Published Windows Installer

- **WHEN** the release download directory contains its installer and latest.json without an .exe.sig sidecar
- **THEN** verification checks the manifest version, exact asset URL name and cryptographic signature against that installer

### Requirement: Preserve Required Native Validation

Release tooling SHALL preserve required Windows and Linux Check/Test coverage and the development Release / Dry job, and SHALL use credential-free release rehearsal.

#### Scenario: Inspect The Release Workflow

- **WHEN** product configuration is adapted
- **THEN** required native checks remain enabled, development release validation is retained and rehearsal does not upload artifacts
