# Plugin Authoring

## ADDED Requirements

### Requirement: Label-First Traceability

Traceability sections and rows SHALL derive their identifiers from labels using document normalization unless the author explicitly sets an identifier. Constructors SHALL accept labels first, with no ID-first aliases. Existing serialized contracts SHALL remain unchanged.

#### Scenario: Derived Identity

- **WHEN** an author constructs a section or typed row with a label
- **THEN** its identifier matches document normalization of that label and its data remains intact

#### Scenario: Stable Edit Identity

- **WHEN** an explicit section or row ID is reused with another label
- **THEN** replacement preserves ordering and unrelated captured content

### Requirement: Canonical Runnable Examples

Every maintained example and sample plugin SHALL preserve its behavior while demonstrating canonical linear authoring, named criterion checks, document statements, operation ownership and presentation conventions.

#### Scenario: Example Review

- **WHEN** an author reads a template or observer
- **THEN** driver work is owned by operations and actions read top to bottom with named requests and Title Case comments

#### Scenario: Example Execution

- **WHEN** the focused sample and starter proofs execute
- **THEN** their existing evidence, outcomes, report content and cleanup behavior remain available
