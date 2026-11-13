## ADDED Requirements

### Requirement: Product SDK Entry Point

The Rust SDK at `sdk/rust` SHALL be `dokime`, exposing plugin authoring and its client half through `dokime::prelude::*`, depending only on protocol within the product graph and compiling no host. Harmos SHALL use the same arrangement as `harmos`.

#### Scenario: Author Imports the Product

- **WHEN** a Rust author imports the product prelude
- **THEN** plugin and client APIs are available without runtime or storage dependencies

### Requirement: Protocol Ownership

Proto files SHALL own contracts. Protocol crates SHALL contain generated definitions and light helpers. SDKs SHALL implement documented cross-language semantics. Dokime SHALL have no domain crate; Harmos domain SHALL retain only native state, recovery and storage semantics that are not wire mirrors.

#### Scenario: Definition Is Shared

- **WHEN** an operation or SDK needs a wire definition
- **THEN** it uses generated protocol types rather than a duplicate domain model

### Requirement: Direct Host Operations

Every host operation SHALL co-locate declaration, gRPC mapping, Rust callee policy and handler in one file in operations, accepting and returning protocol messages through narrow resources without a runtime dependency. Harmos SHALL accept generated protobuf types with descriptors and bounded decoding.

#### Scenario: Handler Is Registered

- **WHEN** runtime registers an operation
- **THEN** the host exposes its mapping directly without `hosted!`, `Host`/`Serve` forwarding or a per-service adapter

### Requirement: Runtime Host Ownership

Runtime SHALL own execution management, transactions, state, cycle transitions, effects, plugin entries, startup and listeners. Transactions, state and cycle transitions SHALL stay together. CLI and desktop SHALL embed the runtime library; remote access SHALL use listener configuration and one client authorization hook.

#### Scenario: Host Starts

- **WHEN** CLI or desktop starts a host
- **THEN** runtime composes resources, registers handlers and starts listeners without a server crate

### Requirement: Server Behavior Preservation

Logs, support bundles, transfers, paging, watches, plot deltas, diagnostics and redaction SHALL remain ordinary operations after the server crate is deleted. Pairing persistence, certificate rotation and shutdown SHALL remain implemented.

#### Scenario: Client Uses a Migrated Operation

- **WHEN** a client resumes a transfer or watch after migration
- **THEN** its bounds, cursor, completion and authorization semantics remain enforced

### Requirement: Layer Direction and Removal

Host dependencies SHALL flow from protocol through storage, operations and runtime to CLI/desktop. The separate domain, server and client crates, dead conversions and operations contract-only role SHALL be removed after consumer migration. `dokime-tracing` SHALL remain small development support.

#### Scenario: Migration Completes

- **WHEN** wave 5 closes
- **THEN** no SDK depends on host layers, no operations depend on runtime and no superseded crate remains
