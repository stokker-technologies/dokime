# Feature Drop Specification

## Purpose
Define the native execution, persistence, reporting and operator workflows delivered together across Dokime interfaces.

## Requirements

### Requirement: Single Plugin Dependency Surface

Native Dokime plugin consumers SHALL enable only `dokime/plugin-sidecar` for their authoring surface. That feature SHALL enable native runtime support and derives; `plugin` SHALL alias it. Entry-library consumers SHALL resolve Harmos APIs and generated implementation paths through Dokime re-exports, without a direct Harmos dependency.

#### Scenario: Independent Plugin Consumer

- **WHEN** a fresh starter or renamed entry-library consumer enables `plugin-sidecar` without declaring Harmos
- **THEN** plugin declarations and generated code compile, and native plugin packaging can read the resulting catalog.

### Requirement: Coherent Operator Workflows

The desktop SHALL implement every numbered feature-drop requirement using consistent live and frozen-report components where their contracts agree.

#### Scenario: Execution with Prompts and Frozen Evidence

- **WHEN** an operator runs a planned cycle, answers execution prompts and opens its report
- **THEN** controls reflect the correct lifecycle, prompts belong to their execution, every planned member is present, and labels, units, markers and plots retain their meaning.

### Requirement: Bounded Native Delivery and Runtime Work

Native delivery and runtime updates SHALL avoid redundant full payload/state work while preserving durability, cancellation and wire limits.

#### Scenario: Large Payload and Idle Open Cycle

- **WHEN** the operator installs a large bundle, exports evidence, opens a report or leaves a cycle open
- **THEN** transfers respect message budgets, report delivery avoids base64 duplication, and GUI refresh follows meaningful changes instead of repeated full-history polling.

### Requirement: Measured Native Lifecycle

The drop SHALL include the requested system benchmarks and native shutdown evidence without replacing required correctness checks.

#### Scenario: Closing Owned and Attached Desktops

- **WHEN** the operator closes an idle or running desktop
- **THEN** an owned host drains within the documented shutdown budget, force termination is bounded, reports are not rematerialized, and an attached remote host remains alive.
