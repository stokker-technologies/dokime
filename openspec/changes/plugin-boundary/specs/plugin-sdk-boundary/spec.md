## Purpose

Make Dokime the complete author-facing plugin API in every supported language while reusing shared carrier, generation and inspection machinery.

## ADDED Requirements

### Requirement: Dokime Is the Only Author API

Dokime SHALL own its plugin contract, four author SDKs and starters. Rust, Python, C# and C++ author APIs SHALL require only Dokime types, attributes, namespaces and errors, with automatic internal dependency resolution. No public signature or escape hatch SHALL expose a Harmos type. Import aliases SHALL NOT substitute for a product API.

#### Scenario: Installed SDK Consumer

- **WHEN** an author builds a plugin with independent operations, settings, windows and lifecycle hooks in any supported language
- **THEN** only Dokime author imports and package instructions are needed and all public values and failures have Dokime API paths

### Requirement: Shared Helpers Stay Internal

Dokime SDKs SHALL delegate carrier transport, cancellation and window framing to Harmos helpers. The product registrar and registration API SHALL compose product declarations internally without re-exporting Harmos author contracts or copying carrier engines.

#### Scenario: Register a Typed Suite

- **WHEN** an author registers a suite catalog with typed records
- **THEN** the Dokime registrar preserves one plugin type and passes frozen declarations to the carrier through internal translation

### Requirement: Generation and Inspection Have One Owner

Dokime SHALL use consolidated Harmos descriptor/codegen tooling with its richer renderer fixes preserved and SHALL use Harmos bounded metadata inspection. It SHALL retain no duplicate renderer or spawn/connect/Get/Stop probe, and SHALL keep product section interpretation and output configuration in Dokime.

#### Scenario: Bundle Metadata Inspection Is Cancelled

- **WHEN** Dokime cancels candidate inspection after a process is spawned
- **THEN** the Harmos process owner settles it under the inspection policy and Dokime has no separate teardown implementation

### Requirement: Product and Generic Conformance Are Separate

Dokime SHALL reuse Harmos generic conformance and own additional proof of its author API boundary, admission, installation, settings, suites/templates, evidence, secrets and execution resource lifecycle in all four SDKs.

#### Scenario: Author SDK Hides Carrier Types

- **WHEN** each installed-consumer fixture compiles and runs its product authoring flow
- **THEN** its API boundary and product behavior are proved separately from the reused carrier conformance rules
