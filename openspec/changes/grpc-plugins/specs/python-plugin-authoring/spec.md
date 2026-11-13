## MODIFIED Requirements

### Requirement: Decorator API

Python SHALL offer decorators and value composition for suites, templates, steps, observers,
criteria, records, streams, documents and operations with the same concepts as the other SDKs.

#### Scenario: Equivalent Definition

- **WHEN** Python and Rust authors declare the same template, parameters and streams
- **THEN** their typed catalogs, materialized cases and document behaviour match

#### Scenario: Invalid Declaration

- **WHEN** a declaration has duplicate identifiers or unresolved fields
- **THEN** plugin startup fails naming the declaration before execution

### Requirement: Authoritative Deadlines

Python operations SHALL obey declared operation policies enforced on the host clock.
Unset policies SHALL mean no deadline, zero cancellation grace and no retries.

#### Scenario: Deadline Reached

- **WHEN** the declared deadline expires
- **THEN** the host cancels the invocation and reports its deadline failure

#### Scenario: Ignored Cancellation

- **WHEN** a callback ignores cancellation after its declared policy expires
- **THEN** the invocation fails without preventing the host from finishing cleanup and closing its execution

### Requirement: Plugin Identity Conflict

A Python remote plugin SHALL obey the same connected identity and operation uniqueness rules as managed plugins.

#### Scenario: Same ID as Installed

- **WHEN** a remote entry connects with an identity already connected
- **THEN** that connection is refused with a named conflict while unrelated entries remain available

## REMOVED Requirements

### Requirement: Rust Calls Python

**Reason**: The embedded Python provider and callback bridge are removed.
**Migration**: Serve ordinary unary plugin operations over Harmos gRPC.

### Requirement: Batch Stream Publication

**Reason**: Live plugin streams send one sample per message with gRPC backpressure.
**Migration**: Publish live samples through declared streams; later bulk imports use their own columnar admission.
