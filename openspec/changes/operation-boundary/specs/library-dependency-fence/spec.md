## Purpose

Keep Dokime on the idiomatic Harmos surface: runtime building, operation calling, backend authoring and plugin authoring, without direct use of Harmos internal crates or managers.

## ADDED Requirements

### Requirement: Facade-Only Harmos Dependencies

Dokime manifests SHALL depend on the Harmos host facade and SDK crates only, with no direct dependency on the Harmos domain, query, storage, operations or protocol crates.

#### Scenario: Manifest Check

- **WHEN** a Dokime manifest declares a direct dependency on a Harmos internal crate
- **THEN** the dependency check fails naming the facade to use instead

### Requirement: Runtime Built Through the Builder

Dokime SHALL build its runtime by passing its backend pair, catalog and settings to the Harmos builder and SHALL call Harmos only through the typed in-process facade, retaining the runtime handle and never a Harmos manager.

#### Scenario: Plugin Activation

- **WHEN** an execution activates a plugin, binds its clock and starts its samples
- **THEN** Dokime calls the Harmos facade for each step, each call dispatching its in-process operation, and holds no plugin manager

### Requirement: Backend Through the Authoring Module

The Dokime backend SHALL implement the Harmos backend traits importing their contracts, catalogs and codecs only from the facade's backend authoring module.

#### Scenario: Backend Build

- **WHEN** the storage crate is built
- **THEN** it compiles against the authoring module without Harmos engine helpers
