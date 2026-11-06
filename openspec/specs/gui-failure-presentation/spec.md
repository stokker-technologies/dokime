# GUI Failure Presentation Specification

## Purpose
Preserve complete diagnostic context in one normalized GUI failure record and present each occurrence consistently through inline cards, notifications and deduplicated console reporting.

## Requirements

### Requirement: One Failure Stem
The GUI SHALL construct every presented failure through `failureOf(error, title)`, extending NotificationFailure with title, identity and available client Error.stack while preserving root-first causes, field problems, operation/request/invocation IDs and redacted backend diagnostics.

#### Scenario: Rich Native Or Client Error
- **WHEN** an error carries canonical diagnostics or a nested client cause chain
- **THEN** normalization retains the full context and available stack in one serializable record
- **AND** repeated normalization of the same occurrence returns that record unchanged

### Requirement: Inline Failure Card
An unavailable view SHALL show ErrorCard with a destructive border/icon distinct from focus colors, title, root message, optional Try Again, collapsed Technical Details and Copy of the complete record. PaneNotice SHALL be limited to loading and empty states.

#### Scenario: Failed View And Retry
- **WHEN** a view cannot load data
- **THEN** ErrorCard renders its failure record, exposes diagnostics only on expansion and invokes its provided retry callback
- **AND** Copy includes the complete failure record
- **AND** the view does not publish a toast

### Requirement: Central Routing And Console Reporting
One routing owner SHALL send view failures to ErrorCard and console, failed user actions to toast/history/console, and background startup/plugin/stream failures to notification/toast/console. Every leaf SHALL receive the normalized record unchanged. The console reporter SHALL emit a collapsed error group containing the complete technical context exactly once per failure identity.

#### Scenario: Shared Failure Across Renders And Leaves
- **WHEN** one occurrence is presented repeatedly or reaches multiple leaves
- **THEN** the console emits one group with title, message, causes, IDs, backend diagnostics/backtrace and client stack
- **AND** new failed attempts with distinct identities are reported independently

#### Scenario: Action Or Background Failure
- **WHEN** a user action or background operation fails
- **THEN** the toast and notification history retain the same normalized failure and their details dialog exposes its diagnostics and client stack
- **AND** existing remediation actions remain available

#### Scenario: Notification Storage Fails
- **WHEN** notification history cannot be read or written
- **THEN** the failure reaches the console once and available presentation without recursive history writes
