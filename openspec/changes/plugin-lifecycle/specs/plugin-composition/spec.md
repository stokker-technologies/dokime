## Purpose

Define how a Dokime plugin composes its product kinds on the library registrar: module and suite catalogs, template-owned prompts and records, inert registration, plugin finalize and the lifetime of declarations versus execution instances.

## ADDED Requirements

### Requirement: Dokime Kinds Register Through the Library Registrar

Suites, templates, prompts and record contracts SHALL register through the same register phase and registrar as library declarations, in Rust, Python, C# and C++, and SHALL reach the host in Dokime's declaration section of the frozen declarations.

#### Scenario: Register a Suite Catalog

- **WHEN** an author's register function registers the suites catalog
- **THEN** the host lists every suite and template of that catalog with their operations, prompts and records

### Requirement: Inert Registration

Dokime registration SHALL declare only. Device construction, connections, prompts and any other side effect SHALL run in initialize, never while declarations are composed.

#### Scenario: Manual Device Operations

- **WHEN** a plugin declares operations that drive a device handle
- **THEN** listing the plugin's declarations opens no device, and the handle is opened by initialize

### Requirement: Module and Suite Catalogs

Each module that contributes declarations SHALL expose them through catalog (Catalog in C#). A suite's catalog SHALL include its templates and the prompts, records, observers and streams those templates own.

#### Scenario: Suite Folder

- **WHEN** the root register function registers a suite folder's catalog
- **THEN** every template of the suite and every prompt, record, observer and stream it owns are declared once

### Requirement: Template-Owned Prompts and Records

Templates SHALL declare prompts and record contracts in all four SDKs. Registering a template SHALL aggregate them into the plugin's frozen declarations, refusing duplicate identities, and the host SHALL NOT restrict their use to the declaring template.

#### Scenario: Template Record Contract

- **WHEN** a template declares a record contract and its step appends that record
- **THEN** the record is admitted against the contract from the plugin's frozen declarations

### Requirement: Declarations Outlive Executions

Declarations SHALL be immutable for the activation. Observer instances, stream demands, execution resources and bench state SHALL be created per execution and released when it ends.

#### Scenario: Two Executions of One Template

- **WHEN** the same template runs twice in sequence
- **THEN** both executions see identical declarations and the second starts with no state from the first

### Requirement: Plugin Finalize Releases Execution Resources First

On plugin finalize the SDK SHALL release every remaining execution resource and then run the author's finalize. The plugin SHALL have no other stop hook.

#### Scenario: Plugin Stopped During an Execution

- **WHEN** the host stops a plugin while one of its executions holds resources
- **THEN** in-flight calls are cancelled, the SDK releases the execution resources and the author's finalize runs afterwards
