## Purpose

Ensure every Dokime persistence need reaches storage only through Harmos storage operations, with Dokime's registered backend as the only code that performs physical storage I/O.

## ADDED Requirements

### Requirement: Backend Is the Only Physical Storage Owner

Dokime SHALL implement and register its storage backend with Harmos. Only that backend SHALL read or write the storage root's SQLite catalog, Arrow files and recovery data; every other Dokime component SHALL use Harmos storage operations.

#### Scenario: Operation Reads Execution State

- **WHEN** a host operation needs an execution's stored state
- **THEN** it calls a Harmos storage operation and never opens the product store directly

### Requirement: Product Transitions Use Compound Mutation

Execution, case, cycle, seal and report transitions that change several stored values SHALL commit through one Harmos compound mutation, keeping the transition rules in Dokime.

#### Scenario: Seal With Report Publication

- **WHEN** an execution is sealed and its report is published
- **THEN** the seal, report metadata and related state commit together or not at all

### Requirement: Evidence Ingestion Through Operations

Records and samples SHALL be appended through Harmos append operations under the trusted execution scope, acknowledging a sample batch only after its durable receipt.

#### Scenario: Sample Acknowledgement

- **WHEN** a plugin delivers a sample batch for a running execution
- **THEN** Dokime appends it through the storage operation and acknowledges after the receipt reports durable completion

### Requirement: Watches Through Row Watches

Every Dokime watch over stored data SHALL observe storage through Harmos row watches with issued cuts, projecting product responses from the rows received.

#### Scenario: Execution Watch Resynchronizes

- **WHEN** a watch falls behind and Harmos reports a resynchronization
- **THEN** the Dokime watch emits `resync_required` with its last issued cut and terminates, and the client lists a fresh image before reopening

### Requirement: Deletion Through Scoped Deletion

Deleting an execution or case SHALL use Harmos scoped deletion after Dokime's seal and ownership checks, removing its objects, records, samples, previews and files together.

#### Scenario: Delete an Unsealed Execution

- **WHEN** a client deletes an execution that Dokime policy permits deleting
- **THEN** a later read finds none of its stored data and retained cuts keep reading it until released

### Requirement: Startup and Maintenance Through Operations

Root admission, credential journals, root claims, desktop archive and disposal and command-line doctor copies SHALL use Harmos storage operations on the storage-only runtime, not direct file access.

#### Scenario: Doctor Copy

- **WHEN** the command-line doctor collects a storage copy before starting a host
- **THEN** it requests a Harmos maintenance copy and the source root remains usable

### Requirement: Incompatible Roots Are Refused

When stored shapes change, Dokime SHALL refuse an incompatible existing root with the action to choose a fresh root and SHALL NOT migrate, reset or delete it.

#### Scenario: Root From an Earlier Shape

- **WHEN** the host opens a root written with an incompatible stored shape
- **THEN** it refuses with the fresh-root action and leaves the root unchanged
