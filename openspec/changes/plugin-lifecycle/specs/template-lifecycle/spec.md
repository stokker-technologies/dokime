## Purpose

Define the phases of a test case template execution: pure materialization, initialize, steps and criteria, and a finalize that always follows an attempted initialize and receives the termination reason in every SDK.

## ADDED Requirements

### Requirement: Template Phase Order

The host SHALL drive a template execution as materialize → initialize → steps → criteria → finalize. Materialize SHALL have no execution scope. Sampling SHALL start after initialize and stop before finalize.

#### Scenario: Successful Execution

- **WHEN** a case with steps and criteria runs to completion
- **THEN** the host calls initialize, each step in order, each criterion, then finalize with the completed termination

### Requirement: Finalize After Attempted Initialize

The host SHALL call finalize exactly once for every execution whose initialize was attempted or whose template declares no initialize, including when initialize failed, timed out or was cancelled during the call.

#### Scenario: Initialize Fails

- **WHEN** a template's initialize returns an error
- **THEN** the host calls finalize with the failed termination and records the initialize failure in the execution's outcome

#### Scenario: Step Fails

- **WHEN** a step fails after a successful initialize
- **THEN** the host still calls finalize with the failed termination

### Requirement: Termination in Every SDK

The finalize context SHALL expose the termination reason in the Rust, Python, C# and C++ SDKs.

#### Scenario: Cancelled Execution in C++

- **WHEN** a C++ template's execution is cancelled
- **THEN** its finalize reads the cancelled termination from its context

### Requirement: One Finalize Name

The template release phase SHALL be named finalize on the wire (operation, request, response and template definition reference) and in all four SDKs, with idiomatic casing only. No cleanup phase SHALL remain.

#### Scenario: Template Definition

- **WHEN** a plugin reports a template with a release hook
- **THEN** the template definition references its finalize operation and the host invokes it as finalize
