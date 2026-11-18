## ADDED Requirements

### Requirement: Class-Based Authoring API

Python SHALL compose `Plugin`, `Suite` and `TestTemplate` classes listing their
contents. Operations SHALL be plain functions with request and response types
from annotations. Steps, observers, criteria, records, streams and documents
SHALL retain the same concepts as the other SDKs.

#### Scenario: Equivalent Definition

- **WHEN** Python and Rust authors declare the same template, parameters and streams
- **THEN** their typed catalogs, materialized cases and document behaviour match

#### Scenario: Invalid Declaration

- **WHEN** a declaration has duplicate identifiers or unresolved fields
- **THEN** plugin startup fails naming the declaration before execution

### Requirement: Python Import Homes

Python SHALL expose client APIs, `Plugin`, `Suite` and `ResultStatus` from
`dokime_sdk`, template declarations from `dokime_sdk.templates`, and operation
declarations from `dokime_sdk.operations`. Each submodule SHALL own a separate
`Context`; the root SHALL export none. Examples SHALL import SDK symbols only
from these three homes.

#### Scenario: Separate Handler Contexts

- **WHEN** an author imports a template handler context and an operation handler context
- **THEN** each context comes from its own SDK submodule and neither is exported at the root

### Requirement: Class-Derived Identity

Plugin, suite and template identifiers SHALL derive from class names by stripping
`Plugin`, `Suite` and `TestTemplate` and converting to snake case. Labels SHALL
derive from class names unless overridden by a `label` attribute.

#### Scenario: Derived Suite and Template

- **WHEN** `CaptureSuite` contains `ConfirmationTestTemplate`
- **THEN** their local identifiers are `capture` and `confirmation`, and the qualified template identifier is `capture.confirmation`

### Requirement: Plugin Identity Conflict

A Python remote plugin SHALL obey the same connected identity and operation uniqueness rules as managed plugins.

#### Scenario: Same ID as Installed

- **WHEN** a remote entry connects with an identity already connected
- **THEN** that connection is refused with a named conflict while unrelated entries remain available

## MODIFIED Requirements

### Requirement: Authoritative Deadlines

Python operations SHALL obey declared operation policies enforced on the host clock.
Unset policies SHALL mean no deadline, zero cancellation grace and no retries.

#### Scenario: Deadline Reached

- **WHEN** the declared deadline expires
- **THEN** the host cancels the invocation and reports its deadline failure

#### Scenario: Ignored Cancellation

- **WHEN** a callback ignores cancellation after its declared policy expires
- **THEN** the invocation fails without preventing the host from finishing cleanup and closing its execution
