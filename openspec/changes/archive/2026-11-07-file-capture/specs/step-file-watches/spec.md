## MODIFIED Requirements

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

## REMOVED Requirements

### Requirement: Recursive Filtered Step Watches

**Reason**: `WatchFile` is replaced by `CopyFiles`, `StartCapture` and `StopCapture`, and the recursion flag by rule depth.

**Migration**: Use the `file-capture` capability: a `Tree` source for filtered discovery and `CaptureMode` for capture behavior.

### Requirement: Live Stream Artifact Capture

**Reason**: Its behavior moves with the operation that provides it.

**Migration**: The `Stream Capture` requirement of the `file-capture` capability carries the same retention rules.
