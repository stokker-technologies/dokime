# Python Plugin Authoring Specification

## Purpose

Define native admission and execution of Python declarations, callbacks and column batches through the embedded provider.

## Requirements

### Requirement: Authoritative Deadlines
Python callbacks SHALL obey the same operation policies as Rust plugins.

#### Scenario: Deadline Reached
- **WHEN** an operation's deadline passes
- **THEN** its Python task is cancelled and the operation reports the deadline failure

#### Scenario: Ignored Cancellation
- **WHEN** a callback ignores cancellation
- **THEN** the failure names the callback, and shutdown still completes within its policy
