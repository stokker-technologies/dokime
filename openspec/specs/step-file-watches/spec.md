# Step File Watches Specification

## Purpose
Define bounded, recursive file discovery and capture for step watches while preserving relative paths, ordered filtering and safe artifact names.

## Requirements

### Requirement: Recursive Filtered Step Watches

WatchFile SHALL expose recursive discovery and ordered root-relative FileRule globs through its existing request. Directory requests SHALL preserve relative paths below output, reject a single artifact override, and enforce file, byte and polling bounds. Empty rules with recursion disabled SHALL retain single-file behavior; recursive requests without rules SHALL include all files.

#### Scenario: A Log Tree Grows After Arming
- **WHEN** a Stream watch arms over existing files and a new subfolder later appears
- **THEN** existing files contribute only bytes after the baseline and new files contribute bytes from zero under output/relative-path in both native host and harness

#### Scenario: Archives Are Mirrored Selectively
- **WHEN** a recursive Mirror watch includes **/*.zip
- **THEN** only matching files are captured and changed archives replace their mirrored bytes

#### Scenario: Resource Limits Remain Enforced
- **WHEN** a directory watch exceeds its requested file or byte cap
- **THEN** it fails explicitly and retains captured evidence

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

### Requirement: Live Stream Artifact Capture

Native host and harness Stream watches SHALL omit files that existed at arming and contributed no bytes. They SHALL retain new files from byte zero, including genuinely empty new files. Watch outcomes and reports SHALL include only retained artifacts.

#### Scenario: Untouched Baseline
- **WHEN** a recursive Stream watch seals without changes to its existing files
- **THEN** no artifacts are retained and the outcome contains no artifacts

#### Scenario: Appended And New Files
- **WHEN** an existing file is appended and a new subfolder file appears after arming
- **THEN** one mirrored artifact contains only the appended bytes and the new file contains all bytes from zero

### Requirement: Bounded Live Artifact Reads

Captured Stream files SHALL appear live with growing size and bounded readable content before sealing, then expose finished state using the existing execution-file follower. Viewer reads SHALL preserve existing budgets and SHALL NOT depend on viewer consumption to progress capture.

#### Scenario: Live Growth And Completion
- **WHEN** a captured file grows while its watch remains active
- **THEN** the live list and follower expose its committed bytes and increasing size, and sealing finishes the same file identity

#### Scenario: New Empty File
- **WHEN** an empty source file is created after arming
- **THEN** the watch retains it as a real empty artifact

#### Scenario: Capture Fails After A Live Read
- **WHEN** artifact content capture fails after a viewer has started following it
- **THEN** its follower terminates as abandoned and reports retain the failed artifact outcome without finished content

#### Scenario: Uploaded Media Attribution
- **WHEN** a step upload declares its media type or uses the default octet-stream type
- **THEN** live listing, bounded reads, follow metadata and finished evidence retain that original type
