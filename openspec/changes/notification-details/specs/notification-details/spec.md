## ADDED Requirements

### Requirement: Accessible Notification Inspection

Every toast and retained notification SHALL open a themed details dialog by click or Enter/Space. Esc SHALL close it. Details SHALL include title, complete message, source/operation, absolute and relative time, severity, available typed reason, root-first causes, problems, operation/request/invocation identifiers and copyable diagnostics. Technical text SHALL be collapsed by default and SHALL NOT replace readable prose. Existing plugin remediation actions SHALL remain usable.

#### Scenario: Inspect And Copy A Failure

- **WHEN** an operator activates a failure notification
- **THEN** the dialog shows readable context with root causes before collapsed consequences, and Copy copies all details including collapsed diagnostics

#### Scenario: Reopen Retained Feedback

- **WHEN** an operator opens a notification from history after a reload
- **THEN** the same structured context remains available

#### Scenario: Keyboard Inspection

- **WHEN** an operator focuses a notification and presses Enter or Space
- **THEN** the dialog opens, and Esc closes it with focus restored

### Requirement: Preserve Aggregated Problems

Notification producers SHALL retain each problem and its cause instead of publishing only a count. Backend structured details SHALL use the canonical generated contracts, without Debug formatting or a parallel domain codec.

#### Scenario: Invalid Template Inputs

- **WHEN** template parameters or files have validation problems
- **THEN** the notification details list every affected field and its reason

#### Scenario: Native Failure Details

- **WHEN** a native RPC failure supplies structured rejection or captured error details
- **THEN** the notification retains its reason, issues and cause chain for inspection and copying
