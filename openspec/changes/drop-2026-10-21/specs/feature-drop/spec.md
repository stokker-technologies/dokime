## ADDED Requirements

### Requirement: Coherent Operator Workflows

The desktop SHALL implement every numbered feature-drop requirement using consistent live and frozen-report components where their contracts agree.

#### Scenario: Execution With Prompts And Frozen Evidence

- **WHEN** an operator runs a planned cycle, answers execution prompts and opens its report
- **THEN** controls reflect the correct lifecycle, prompts belong to their execution, every planned member is present, and labels, units, markers and plots retain their meaning.

### Requirement: Bounded Native Delivery And Runtime Work

Native delivery and runtime updates SHALL avoid redundant full payload/state work while preserving durability, cancellation and wire limits.

#### Scenario: Large Payload And Idle Open Cycle

- **WHEN** the operator installs a large bundle, exports evidence, opens a report or leaves a cycle open
- **THEN** transfers respect message budgets, report delivery avoids base64 duplication, and GUI refresh follows meaningful changes instead of repeated full-history polling.

### Requirement: Measured Native Lifecycle

The drop SHALL include the requested system benchmarks and native shutdown evidence without replacing required correctness checks.

#### Scenario: Closing Owned And Attached Desktops

- **WHEN** the operator closes an idle or running desktop
- **THEN** an owned host drains within the documented shutdown budget, force termination is bounded, reports are not rematerialized, and an attached remote host remains alive.

### Requirement: Observable And Faithful File Capture

Watchers SHALL expose readiness after their baseline, preserve rotation segments, and separate selection from byte capture. All remains default; Latest selects the newest baseline match deterministically and follows its rotation; New captures only post-baseline discoveries from byte zero.

#### Scenario: Device Starts After Arming

- **WHEN** a caller awaits readiness and then starts a producer
- **THEN** capture is armed even for an explicitly empty baseline and no startup sleep is needed.

#### Scenario: Rotation And FTP Resume

- **WHEN** a selected file rotates or FTP cannot resume required extraction
- **THEN** distinct segments remain preserved without cumulative single-file rejection, and unsupported extraction returns a clear failure/gap while retaining captured evidence.

#### Scenario: Absent Source With Opt-In Waiting

- **WHEN** an opted-in source appears within the deadline, remains absent past it, or is cancelled while waiting
- **THEN** the watcher respectively arms, times out, or cancels; strict defaults and permission/invalid-path errors remain immediate. Unavailable FTP roots fail immediately because status 550 cannot distinguish absence from denial; waiting applies to missing FTP files beneath accessible roots and local roots/files.

### Requirement: Organization Packs And Trusted Release Sources

Organization Packs SHALL provide appearance and organizational update defaults without changing compiled app identity or signing trust. One resolved policy SHALL support disabled, manual and automatic checks through direct manifest, GitHub Releases and GitLab Releases sources; installation remains explicit.

#### Scenario: Invalid Pack Or Source Switch

- **WHEN** an operator stages an invalid policy pack or switches a named update source
- **THEN** the last valid configuration survives invalid input with a visible error, source switching preserves signing keys, and disabled policy performs no update network activity.

#### Scenario: Authenticated App And Plugin Releases

- **WHEN** an organization declares named connections and app/plugin release sources
- **THEN** public sources work without login, configured credentials remain host/account/client-scoped and do not cross origins, compatible signed app assets are verified, and plugin discovery/staging retains explicit consent, install and restart ownership.

### Requirement: Host-Owned Shared Secret Storage

The host SHALL provide freely managed stable named secrets shared through plugin lookup, while app authentication tokens and signing keys remain separate. Organization Packs SHALL contain neither secret values nor plugin secret binding declarations.

#### Scenario: Shared Lookup And Management

- **WHEN** multiple plugins look up a name that is added, replaced or deleted
- **THEN** future lookups see its current value or a clear missing-secret error, existing required-secret checks fail before device work where available, and settings show the owning host without exposing values or claiming exhaustive usage.

#### Scenario: Encrypted Transfer And Conflict Preview

- **WHEN** an operator exports selected secrets and imports them with explicit conflict choices
- **THEN** authenticated password encryption protects the transfer, wrong passwords/tampering/invalid or oversized input write nothing, and accepted replace/skip choices publish atomically with no raw values in logs, reports or evidence.

#### Scenario: Secure Backend Requirement

- **WHEN** an operator attempts a new shared-secret write or import without OS keyring storage
- **THEN** the host reports the secure-storage requirement and preserves existing LocalFile data for explicit secure adoption without silent deletion or rewriting.
