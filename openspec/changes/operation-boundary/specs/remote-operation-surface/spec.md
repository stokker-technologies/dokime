## Purpose

Define what the Dokime host exposes remotely: only operations declared in Dokime's own protocol, including wrappers over in-process Harmos host operations, with SDKs generated from the embedded descriptors.

## ADDED Requirements

### Requirement: Product Wrappers for Used Harmos Operations

Dokime SHALL declare a wrapper for each generic Harmos host operation its clients use, calling the typed facade in-process. Unused operations SHALL NOT be wrapped. Plugin management/settings contracts and policy SHALL instead be Dokime-owned operations over Harmos activation and storage, not wrappers around library administration.

#### Scenario: Desktop Lists Plugins

- **WHEN** the desktop lists installed plugins
- **THEN** it calls the Dokime plugin management operation, which reads the product registry through Harmos storage operations

#### Scenario: Unwrapped Harmos Operation

- **WHEN** a client calls a Harmos host method directly on the Dokime listener
- **THEN** the host refuses the call and executes no handler

### Requirement: Host Build Identity

Product Status Get and Watch responses SHALL carry the answering host's source revision and publication channel alongside the library health and issued boundary. The runtime SHALL derive these values through the shared product build identity implementation. The semantic version SHALL remain in the status identity's application version.

#### Scenario: Client Identifies a Connected Host

- **WHEN** a client reads or watches product Status
- **THEN** its response includes the answering host's source revision and channel
- **AND** the command-line status and desktop connection details display that build
- **AND** Settings About displays the backend build beside the desktop build when they differ

### Requirement: Retained Log Reads

The product SHALL expose client-only Logs List with the library's page request, response and issued-boundary semantics, delegating to the library's in-process retained log reader. Logs Watch SHALL keep its opening boundary and later-event semantics without an opening snapshot.

#### Scenario: Client Pages Retained Logs

- **WHEN** a client requests product Logs List with page bounds and follows its issued continuation
- **THEN** the product returns retained redacted events at that read boundary through the library reader
- **AND** report-save and screenshot journeys use this product operation

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
