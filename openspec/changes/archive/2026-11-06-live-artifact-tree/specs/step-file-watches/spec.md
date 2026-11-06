## ADDED Requirements

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
