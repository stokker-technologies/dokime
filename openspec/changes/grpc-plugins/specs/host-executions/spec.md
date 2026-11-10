## ADDED Requirements

### Requirement: Managed and Remote Plugins

Dokime SHALL load a plugin either as a bundle it installs and supervises or as a URL the user adds, and SHALL treat both the same once connected.

#### Scenario: Remote Plugin Added

- **WHEN** the user or a client adds a plugin URL whose server is reachable
- **THEN** the entry shows ready with the plugin's id, version and catalog digest, and its templates are available

#### Scenario: Remote Plugin Unreachable

- **WHEN** a remote plugin's server is not reachable
- **THEN** the entry stays in the list as unavailable with the reason, and Dokime reconnects without user action when the server returns

### Requirement: Plugin Materializes the Test Case

The host SHALL expand a template's parameters into parameter sets and SHALL obtain each test case from the plugin's pure `materialize` operation.

#### Scenario: Case From Parameters

- **WHEN** the host materializes a template for one parameter set
- **THEN** the plugin returns a test case with its resolved parameters, steps, criteria and streams, without touching an instrument

#### Scenario: Catalog Changed

- **WHEN** the plugin's catalog digest differs from the one a test case was materialized from
- **THEN** the host materializes the case again before executing it

### Requirement: Host Runs the Execution

The host SHALL create the execution and SHALL invoke the plugin's initialize, step, criterion and cleanup operations itself.

#### Scenario: Order

- **WHEN** an execution runs
- **THEN** the host calls initialize, each step in order, each criterion, then cleanup

#### Scenario: Cleanup After Failure

- **WHEN** a step fails or the execution is cancelled after initialize succeeded
- **THEN** the host still calls cleanup

#### Scenario: Retryable Step

- **WHEN** a step declared with retries fails an attempt
- **THEN** the host calls it again until it succeeds or the retries are used up

#### Scenario: Connection Dropped

- **WHEN** the connection drops during a step and the host reconnects to the same plugin process
- **THEN** only that step attempt failed and the execution continues under the step's retry rule

#### Scenario: Plugin Restarted

- **WHEN** the plugin process restarts during an execution
- **THEN** the host seals the execution and its cycles

### Requirement: Host Operations Through the Context

A step or criterion SHALL reach Dokime only by calling host operations through its context.

#### Scenario: Record

- **WHEN** a step publishes a record
- **THEN** the call returns after the host stored it, and a later criterion reads it

#### Scenario: Statistics

- **WHEN** a criterion asks for a stream's statistics for one step
- **THEN** the host returns statistics computed over that step's samples of the execution

#### Scenario: Settings

- **WHEN** a plugin's settings are saved while it is connected
- **THEN** the plugin keeps running and its next settings read returns the saved values

#### Scenario: Verdict

- **WHEN** every criterion returns pass
- **THEN** the execution passes, and it fails when any criterion returns fail

### Requirement: Host-Opened Execution Streams

The host SHALL open the streams a test case names before initialize and close them after cleanup, and SHALL attach their samples to the execution in arrival order.

#### Scenario: Observer

- **WHEN** a test case names a stream backed by an observer with an interval
- **THEN** the execution receives one sample per interval while it runs

#### Scenario: Samples During a Disconnect

- **WHEN** the connection drops and returns to the same process
- **THEN** samples produced in between are absent and the stream continues

### Requirement: SDKs Provide and Drive

Each language SDK SHALL let an author serve a plugin and drive Dokime through `dokime.v1` from the same package.

#### Scenario: Script Runs Its Own Suite

- **WHEN** a Python script serves a suite, adds its URL to a running Dokime, materializes cases and starts a cycle
- **THEN** the cycle executes the suite's steps and the script observes its progress

#### Scenario: Definitions Exported

- **WHEN** an author runs the proto export command for `dokime.v1` or for one plugin
- **THEN** it writes the `.proto` files the author's own toolchain generates code from

#### Scenario: Generated Client

- **WHEN** an operation served to clients is added to `dokime.v1`
- **THEN** every language's client exposes it after regeneration, and a check fails while the stubs are stale

### Requirement: Runtime-Free Authoring

A plugin SHALL build against the SDK for its language without compiling the Dokime host.

#### Scenario: Plugin Dependency Tree

- **WHEN** a generated Rust starter plugin is resolved
- **THEN** its dependency tree does not include `dokime-runtime`

### Requirement: One Command-Line Tool

Project creation, plugin packaging, installer packaging and proto export SHALL live in the `dokime` tool, and `cargo-dokime` SHALL no longer exist.

#### Scenario: New Project

- **WHEN** a developer runs `dokime new my-plugin --lang python`
- **THEN** it creates an ordinary Python project that runs without a Rust toolchain

#### Scenario: Unsupported Language

- **WHEN** `--lang` names a language with no SDK
- **THEN** the command fails, lists the supported languages and creates nothing
