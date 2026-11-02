# step-traceability Specification

## Purpose
Define step-owned traceability edits through the execution context, using the existing durable evidence pipeline and preserving read-side contracts.

## Requirements

### Requirement: Steps Own Dynamic Traceability

The system SHALL let steps add and replace traceability sections and rows through their execution context. It SHALL NOT expose a template traceability lifecycle hook or mutation from criteria, initialization, cleanup or observers.

#### Scenario: Discovery During A Step

- **WHEN** a step discovers device identity and submits a section through its context
- **THEN** live observation and subsequent captured reports expose that section through existing traceability projections

#### Scenario: Editing Captured Identity

- **WHEN** steps submit repeated section identifiers or replace a row in an existing section
- **THEN** edits apply in delivery order, the latest value wins, and unrelated sections and rows remain intact

#### Scenario: Refused Edit

- **WHEN** a step references a missing section, submits duplicate row identifiers or claims a host-derived section identifier
- **THEN** the edit is logged and refused without changing previously captured traceability

### Requirement: Existing Evidence And Storage Ownership

The system SHALL use the document-style execution index and buffered evidence pipeline with existing durable traceability transactions and projections. Development storage SHALL remain v1 and existing data SHALL never be silently reset or deleted.

#### Scenario: Multiple Edits In One Delivery

- **WHEN** several edits to one section share an evidence batch
- **THEN** durable state and testing reports retain the final edit without duplicate sections

#### Scenario: Captured Report Stability

- **WHEN** a report has been sealed from captured evidence
- **THEN** later live edits do not rewrite its captured traceability
