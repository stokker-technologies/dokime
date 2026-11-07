## ADDED Requirements

### Requirement: Thin Interface Layer
The package SHALL be a PyO3 interface beside the GUI and the CLI, holding no domain
logic of its own.

#### Scenario: Behaviour Lives in Rust
- **WHEN** a Python call starts a cycle, reads evidence or runs a plugin operation
- **THEN** the behaviour runs in the existing Rust crates through a wrapper

### Requirement: Rust API Mirror
The public Python API SHALL mirror the Rust API's concepts, names and arguments,
deviating only where Python requires it.

#### Scenario: Documented Deviation
- **WHEN** a Python name or shape differs from its Rust counterpart
- **THEN** the design's mapping table records the difference and its reason

### Requirement: Complete Type Stubs
Every public export SHALL have a hand-written stub at its public module path in the
crate's `python/` folder, and the package SHALL ship `py.typed`.

#### Scenario: Stub Drift
- **WHEN** an export is added, removed or changed without updating its stub
- **THEN** stubtest against the built extension fails

### Requirement: Client Parity
The package SHALL provide every behaviour in the `python-client` capability, replacing
`dokime-client` and its generated wire code.

#### Scenario: Existing Lifecycle
- **WHEN** a script uses owned cycles, evidence iterators, finite queries or dataframes
- **THEN** each works through the Rust client with the documented semantics

#### Scenario: Superseded Package
- **WHEN** the change completes
- **THEN** `packages/dokime-client` and its build, test and release wiring are gone

### Requirement: One Wheel per Machine
Each supported platform SHALL build one abi3 wheel for all supported Python versions,
after the product build and outside default workspace builds.

#### Scenario: Ordinary Build
- **WHEN** a developer runs a build task that does not name Python
- **THEN** PyO3 and the extension are not compiled

#### Scenario: Supported Versions
- **WHEN** one platform's wheel installs on each supported Python version
- **THEN** it imports and passes the same acceptance tests

### Requirement: Python System Lane
A `test:python:system` lane SHALL prove the whole flow on every required platform, running in Nightly rather than on pull requests.

#### Scenario: End to End Story
- **WHEN** a script starts a backend with a Python plugin, exports a pairing file, attaches
  a client through it, runs an execution with a prompt and a stream, then exits
- **THEN** a new backend on that root reads the sealed report without the plugin

### Requirement: Simple Installation
Developers SHALL install the package with `uv add`, the language's idiomatic modern toolchain, and any
required command-line tools through mise.

#### Scenario: New Project
- **WHEN** a developer follows the install section in a new project
- **THEN** a few documented commands reach a building sample, with no manual downloads
