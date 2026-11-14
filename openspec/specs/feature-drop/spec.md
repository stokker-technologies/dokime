# Feature Drop Specification

## Purpose
Define the native execution, persistence, reporting and operator workflows delivered together across Dokime interfaces.

## Requirements

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
