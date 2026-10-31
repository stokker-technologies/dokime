# Evidence Selection Specification

## Purpose
Define execution and cycle membership for canonical records and telemetry samples, including durable capture links, indexed reads, live watches and scoped evidence exports.

## Requirements

### Requirement: Produced Or Captured Evidence
Execution and cycle selections SHALL include evidence produced by that owner or captured by its executions, preserving the producer scope and one stored payload.

#### Scenario: Observer Capture
- **WHEN** execution A captures a plugin-wide record or sample
- **THEN** A and its cycle can read and watch it, while an unrelated execution cannot

#### Scenario: Shared Capture
- **WHEN** A and B capture the same evidence
- **THEN** each can query it once through indexed membership without duplicating payloads

#### Scenario: Existing Evidence Consumers
- **WHEN** reports, exports, Python iterators or checkpoints select an execution
- **THEN** they retain captured evidence and existing report context behavior
