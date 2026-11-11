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

- **WHEN** the manager drives a TestCase run for an execution
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
- **THEN** the manager interrupts the active run, closes its execution and finalizes it and its affected cycles without touching import drafts

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

- **WHEN** at least one criterion is evaluated and every evaluated criterion returns pass
- **THEN** the execution has an evaluated Passed outcome, and an evaluated failure yields Failed

### Requirement: Host-Opened Execution Streams

The host SHALL open the streams a test case names before initialize and close them after cleanup, and SHALL attach their samples to the execution in arrival order.

#### Scenario: Observer

- **WHEN** a test case names a stream backed by an observer with an interval
- **THEN** the execution receives one sample per interval while it runs

#### Scenario: Samples During a Disconnect

- **WHEN** the connection drops and returns to the same process
- **THEN** samples produced in between are absent and the stream continues

### Requirement: SDKs Provide and Drive

Each language SDK SHALL let an author serve a plugin and drive Dokime through `dokime.v1` from the same package. Python SHALL offer flat value composition and explicit resource close without requiring a context manager.

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


### Requirement: Evidence Container Before Host Adoption

Before manager construction, Execution SHALL contain identity, cycle, origin, frozen schemas, evidence, attributed outcome and seal, with open/closed/sealed states and no step knowledge. Origin SHALL freeze PluginRun with TestCase, plugin/template/digest/start time or Import with its definition/manifest. There SHALL be no separate execution-definition type.

#### Scenario: Frozen Origin

- **WHEN** the host creates an execution
- **THEN** its origin freezes the corresponding TestCase or Import without a fabricated plugin/case for imports

### Requirement: Ephemeral Run State

A run SHALL be execution-manager live state for a TestCase only: queued, initializing, stepping, evaluating, cleaning up, then finished/cancelled/interrupted. It SHALL NOT be a stored object or API type. Cancel SHALL address an execution and SHALL be valid only while a run drives it.

#### Scenario: Run Ends

- **WHEN** the live run finishes, is cancelled or is interrupted
- **THEN** the manager releases its live state while execution evidence remains

#### Scenario: Cancel Import

- **WHEN** a caller cancels an import execution with no live run
- **THEN** the backend refuses the unsupported operation

### Requirement: Origin-Specific Recovery

Interrupted-run recovery SHALL close the affected executions and complete ordinary finalization without treating import drafts as interrupted runs. Plugin restart SHALL interrupt its active runs and finalize their executions and affected cycles; import drafts SHALL be excluded.

#### Scenario: Recover Host

- **WHEN** the host restarts with interrupted runs and import drafts
- **THEN** only the interrupted runs have executions closed by run recovery

### Requirement: Frozen Execution Schemas

Every execution SHALL freeze all schemas it uses from the plugin catalog for a run or the script-local import catalog for an import. Import catalogs SHALL NOT register as plugins or overwrite existing definitions.

#### Scenario: Catalog Changes

- **WHEN** a plugin or script later changes a schema
- **THEN** existing execution evidence retains its frozen schema

### Requirement: Attributed Outcomes and Cycle Verdicts

Outcome SHALL distinguish not evaluated, evaluated (Passed, Failed and existing evaluated kinds), and importer-declared. Imports SHALL NOT have criteria. Cycle headlines SHALL fold only evaluated results; summaries/reports SHALL count declared and unevaluated members separately. A cycle with no evaluated result SHALL NOT be Passed.

#### Scenario: Declared-Only Cycle

- **WHEN** a cycle has only declared Passed or unevaluated members
- **THEN** its headline is not Passed and those member counts remain separate

#### Scenario: Mixed Cycle

- **WHEN** a cycle has evaluated and declared results
- **THEN** only evaluated outcomes determine the headline and reports show declarations separately

### Requirement: Backend Execution Capabilities

The backend SHALL state supported start, rerun, cancel, inspect and export operations for each execution origin/state; desktop SHALL show only those. Imports SHALL NOT inherit start/rerun controls; cancel SHALL require a driving live run.

#### Scenario: Desktop Opens Import

- **WHEN** desktop displays an imported execution
- **THEN** only backend-supported actions are offered and no plugin rerun is attempted

### Requirement: Report Rendering and Publication

Runs and imports SHALL separate rendering a document at a fixed execution snapshot into returned temporary output from durable report publication. Rendering SHALL NOT persist or publish reports/documents. Publication SHALL freeze the selected document/report with the execution under its durability contract.

#### Scenario: Render Without Publish

- **WHEN** a caller renders a snapshot and document
- **THEN** it receives output without durable writes, publication or lifecycle mutation

#### Scenario: Render Fails

- **WHEN** rendering fails before final report publication
- **THEN** no new report is published or seal invented
