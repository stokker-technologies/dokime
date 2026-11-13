## ADDED Requirements

### Requirement: Pure Remote Interface

The Python package SHALL be pure Python over generated gRPC clients and the Harmos SDK,
with plugin and client halves and no embedded backend or native extension.

#### Scenario: Behaviour Lives in the Host

- **WHEN** a Python client starts a cycle or reads evidence
- **THEN** the running Dokime host applies the operation through `dokime.v1`

### Requirement: Typed Pure Python Exports

Every public export SHALL provide Python type information and the package SHALL ship
`py.typed`; generated messages and stubs SHALL remain consistent with authoritative proto inputs.

#### Scenario: Protocol Drift

- **WHEN** an operation changes without regenerating Python client messages and stubs
- **THEN** the generated drift check fails

## MODIFIED Requirements

### Requirement: Client Parity

The package SHALL provide the `python-client` capabilities through generated `dokime.v1`
clients and thin conveniences, with flat APIs and explicit resource close.

#### Scenario: Existing Lifecycle

- **WHEN** a script drives cycles, finite evidence queries or dataframes
- **THEN** these use the remote host without loading Rust or starting a backend

#### Scenario: Superseded Package

- **WHEN** the SDK migration completes
- **THEN** the old separate Python client package and native client wrapper build/test/release wiring are absent

### Requirement: Python System Lane

The Python system story SHALL run in Nightly on required platforms, outside pull requests.

#### Scenario: End to End Story

- **WHEN** a script serves a remote plugin, adds its URL, connects through pairing and runs a prompt/stream execution
- **THEN** a later host reads the sealed report without the plugin

## REMOVED Requirements

### Requirement: Thin Interface Layer

**Reason**: The PyO3 interface is superseded by Pure Remote Interface.
**Migration**: Use generated gRPC clients and the language's Harmos SDK.

### Requirement: Complete Type Stubs

**Reason**: Native extension stubtest is superseded by Typed Pure Python Exports.
**Migration**: Generate message/client typing and check ordinary Python exports.

### Requirement: One Wheel per Machine

**Reason**: The package no longer contains a native extension.
**Migration**: Build one platform-independent pure Python wheel without Rust.
