## ADDED Requirements

### Requirement: Single Plugin Dependency Surface

Native Dokime plugin consumers SHALL enable only `dokime/plugin-sidecar` for their authoring surface. That feature SHALL enable native runtime support and derives; `plugin` SHALL alias it. Entry-library consumers SHALL resolve Harmos APIs and generated implementation paths through Dokime re-exports, without a direct Harmos dependency.

#### Scenario: Independent Plugin Consumer

- **WHEN** a fresh starter or renamed entry-library consumer enables `plugin-sidecar` without declaring Harmos
- **THEN** plugin declarations and generated code compile, and native plugin packaging can read the resulting catalog.

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
