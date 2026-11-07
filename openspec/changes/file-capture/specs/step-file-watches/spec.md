## REMOVED Requirements

### Requirement: Recursive Filtered Step Watches

**Reason**: `WatchFile` is replaced by `CopyFiles`, `StartCapture` and `StopCapture`, and the recursion flag by rule depth.

**Migration**: Use the `file-capture` capability: a `Tree` source for filtered discovery and `CaptureMode` for capture behavior.

### Requirement: Live Stream Artifact Capture

**Reason**: Its behavior moves with the operation that provides it.

**Migration**: The `Stream Capture` requirement of the `file-capture` capability carries the same retention rules.
