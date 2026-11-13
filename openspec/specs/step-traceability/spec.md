# step-traceability Specification

## Purpose
Define step-owned traceability edits through the execution context, using the existing durable evidence pipeline and preserving read-side contracts.

## Requirements

### Requirement: Steps Own Dynamic Traceability

The system SHALL let steps add and replace traceability sections and rows through their execution context. It SHALL NOT expose a template traceability lifecycle hook or mutation from criteria, initialization, cleanup or observers.

#### Scenario: Discovery During a Step

- **WHEN** a step discovers device identity and submits a section through its context
- **THEN** live observation and subsequent captured reports expose that section through existing traceability projections

#### Scenario: Editing Captured Identity

- **WHEN** steps submit repeated section identifiers or replace a row in an existing section
- **THEN** edits apply in delivery order, the latest value wins, and unrelated sections and rows remain intact

#### Scenario: Refused Edit

- **WHEN** a step references a missing section, submits duplicate row identifiers or claims a host-derived section identifier
- **THEN** the edit is logged and refused without changing previously captured traceability
