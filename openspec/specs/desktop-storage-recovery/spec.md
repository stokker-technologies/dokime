# desktop-storage-recovery Specification

## Purpose
Recover failed desktop storage through a confirmed backup and reset while preserving installed plugins and credentials.

## Requirements

### Requirement: Confirm Startup Recovery

The desktop SHALL offer Back Up And Reset after a local startup failure and SHALL
exclude BackendRunning. Confirmation SHALL name the root and archive and explain
removal after backup. Only confirmation SHALL start backup and reset.

#### Scenario: Corrupt Local Storage

- **WHEN** Start Local Backend fails decoding or opening storage
- **THEN** the connection screen offers confirmed recovery

#### Scenario: Occupied Folder

- **WHEN** another backend owns the root and reports BackendRunning
- **THEN** the recovery button is absent and native recovery refuses access

#### Scenario: Backup Failure

- **WHEN** backup cannot complete
- **THEN** no root contents are deleted

### Requirement: Preserve Installations And Secrets

Recovery SHALL retain installed bundles, enabled state and valid settings, preserve
readable node identity and secret catalog, and leave credentials untouched. Bundled
installation SHALL follow normal startup. A refused bundle SHALL remain visible with
its failure without blocking startup or reset. Readable metadata SHALL exclude removed
and orphan bundles; unreadable metadata SHALL allow recovery of bundle files.

#### Scenario: Installed Bundle

- **WHEN** a successful reset replaces execution storage
- **THEN** installed bundles and enabled state survive and valid settings are retained

#### Scenario: Refused Bundle

- **WHEN** the new backend refuses a recovered bundle
- **THEN** startup completes and the plugin inventory exposes its failure

#### Scenario: Removed And Orphan Bundles

- **WHEN** readable metadata records an installation as removed or does not reference a bundle file
- **THEN** automatic recovery excludes that bundle

### Requirement: Report Recovery

Recovery SHALL report stopping, archiving, deleting, reinstalling and starting
through the existing reset steps and expose the completed archive path.

#### Scenario: Completed Backup

- **WHEN** backup succeeds, regardless of a subsequent restart failure
- **THEN** the operator can read the archive path
