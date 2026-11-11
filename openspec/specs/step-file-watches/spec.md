# Step File Watches Specification

## Purpose
Define the artifact names, stored paths and bounded live reads shared by captured files and step uploads.

## Requirements

### Requirement: Safe Ordinary Artifact Extensions

Artifacts SHALL accept ordinary extensions including .diag and .hpl30 with application/octet-stream fallback for unknown types. Both paths SHALL reject traversal, absolute paths and portable reserved device names.

#### Scenario: Diagnostic Files Are Snapshotted
- **WHEN** a step snapshots .diag and .hpl30 files into the artifact root
- **THEN** both native host and harness retain their original names and bytes

#### Scenario: Unsafe Names Are Refused
- **WHEN** an artifact name contains traversal, an absolute path or a reserved device segment
- **THEN** host and harness reject the request

### Requirement: Mirrored Artifact Paths

Artifacts SHALL retain UUID identities internally while storing and displaying `artifacts/<requested path>`, including step uploads. Same-execution collisions SHALL reserve the first free `.N` suffix before the final extension starting at 2, in admission order. Replacement and copy-truncate segments SHALL preserve their existing segment names. Paths SHALL retain traversal and portable reserved-name validation. Development storage SHALL remain v1 without migrations or automatic resets.

#### Scenario: Two Uploads Request One Path
- **WHEN** two artifacts request `logs/device.log` in one execution
- **THEN** their stored paths are `artifacts/logs/device.log` and `artifacts/logs/device.2.log`, with distinct internal UUIDs

### Requirement: Bounded Live Artifact Reads

Captured Stream files SHALL appear live with growing size and bounded readable content before sealing, then expose finished state using the existing execution-file follower. Viewer reads SHALL preserve existing budgets and SHALL NOT depend on viewer consumption to progress capture.

#### Scenario: Live Growth and Completion
- **WHEN** a captured file grows while its capture remains active
- **THEN** the live list and follower expose its committed bytes and increasing size, and sealing finishes the same file identity

#### Scenario: New Empty File
- **WHEN** an empty source file is created after the baseline
- **THEN** the capture retains it as a real empty artifact

#### Scenario: Capture Fails After a Live Read
- **WHEN** artifact content capture fails after a viewer has started following it
- **THEN** its follower terminates as abandoned and reports retain the failed artifact outcome without finished content

#### Scenario: Uploaded Media Attribution
- **WHEN** a step upload declares its media type or uses the default octet-stream type
- **THEN** live listing, bounded reads, follow metadata and finished evidence retain that original type
