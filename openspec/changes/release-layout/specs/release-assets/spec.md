## ADDED Requirements

### Requirement: One Release Layout

A stable release and a Nightly SHALL publish the same asset layout: installers, one archive per
platform for the command-line tool, the distribution and the plugins, one protocol archive, one
compliance archive, one index `release.json` and `SHA256SUMS`.

#### Scenario: Asset List Matches the Layout

- **WHEN** a release or a Nightly finishes publication
- **THEN** every published asset name matches an entry of the layout and no entry is missing

### Requirement: Compliance Archive

Bills of materials and notices SHALL be published only inside
`dokime-compliance-<version>.zip`, with bills of materials under `artifacts/` and
`crates/<platform>/`.

#### Scenario: No Loose Bills of Materials

- **WHEN** a release is published
- **THEN** no asset name ends in `.cdx.json` and the compliance archive holds one bill of materials per shipped file and per source crate and platform

### Requirement: One Release Index

The updater, the download index and the plugin feed SHALL read `release.json`.

#### Scenario: Consumers Read One File

- **WHEN** the desktop checks for an update or lists plugins against a published release
- **THEN** it reads `release.json` and the release has no `latest.json`, `downloads.json` or `plugins.json`

### Requirement: Nightly Reports

Nightly SHALL attach `dokime-reports-<version>.zip` with benchmark results and coverage, and
SHALL summarize them in its release notes. A failed benchmark or coverage job SHALL NOT prevent
publication.

#### Scenario: Results Are Published

- **WHEN** a Nightly completes its benchmark and coverage jobs
- **THEN** the reports archive is attached and the notes show coverage per language, benchmarks beyond their threshold and suite results

#### Scenario: Failed Suite Does Not Block

- **WHEN** a benchmark or coverage job fails in a Nightly
- **THEN** the Nightly is still published and its notes name the failed job
