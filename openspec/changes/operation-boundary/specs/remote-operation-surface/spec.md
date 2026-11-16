## Purpose

Define what the Dokime host exposes remotely: only operations declared in Dokime's own protocol, including wrappers over in-process Harmos host operations, with SDKs generated from the embedded descriptors.

## ADDED Requirements

### Requirement: Product Wrappers for Used Harmos Operations

Dokime SHALL declare a wrapper operation in its own protocol for each Harmos host operation a Dokime client uses, calling the Harmos operation in-process. Harmos host operations no client uses SHALL NOT be wrapped.

#### Scenario: Desktop Lists Plugins

- **WHEN** the desktop lists installed plugins
- **THEN** it calls the Dokime wrapper, which calls the Harmos plugin listing in-process

#### Scenario: Unwrapped Harmos Operation

- **WHEN** a client calls a Harmos host method directly on the Dokime listener
- **THEN** the host refuses the call and executes no handler

### Requirement: Wrapper Audience

Each Dokime wrapper SHALL declare its caller audience on its own method, and runtime admission SHALL refuse callers outside it, so plugins never reach client administration.

#### Scenario: Plugin Calls a Client Wrapper

- **WHEN** a plugin sends a child request to a client-only wrapper
- **THEN** the host refuses it with the wrong-caller failure before any handler runs

### Requirement: Generation Without a Composition File

Dokime SHALL have no composition file. The Rust, Python, C# and C++ SDK generators and runtime assembly SHALL derive packages and audiences from the embedded descriptors and Dokime's own method declarations.

#### Scenario: Regenerate SDKs

- **WHEN** the SDKs are regenerated from the embedded descriptors
- **THEN** each client surface contains Dokime operations and wrappers only, and no Harmos host resource
