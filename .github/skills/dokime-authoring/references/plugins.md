# Dokime Plugin

A Dokime plugin is a native process launched and supervised by the host over piped stdio.

Embed the product runtime through the product entry library by default. Declare its handler catalog in the plugin's `const CATALOGS`; the plugin owns suites, templates, observers, listeners and its own records and streams.

When crash isolation or a native library requires a separate process, host that runtime as a sidecar on the same machine with the same operation contracts. The plugin requires those operations and consumes that product's portable contract crates, beside its Dokime authoring dependency. Dokime data contracts live in `dokime-domain`; its operation declarations live in `dokime-operations` without the `runtime` feature.

Stream row revisions derive automatically from their schema. Changing fields needs
only a rebuild and plugin Reload, with no author version bump or backend restart.
Saved rows retain their definitions. Documents and queries select stream ids and
columns; absent columns contribute no values and incompatible columns are named
on affected bindings without discarding readable evidence. Record revisions also
derive from their declared fields; never author a record schema version. Rebuild
plugins against the matching Harmos SDK. Plugin-owned records, stored objects and
system facts retain their stored definitions, including when the plugin is absent;
unknown plugin objects use `ObjectRetention::All`. Only unreadable Dokime-owned
persistent state can require a fresh root.

Declare records, streams, prompts and settings once in their domain owner. Dokime declares each operation, its policy and family catalog in `dokime-operations`; its `runtime`-gated implementation dispatches through host-provided resources. Plugin-provided operations follow their own operation families and one definition per operation. Public visibility permits another provider to satisfy a requirement; ambiguous providers are refused.

A template lists its calls in its `requires` catalog. Its plugin's own `Private` or `Public` operations can answer without a plugin-level requirement. Calls to another plugin require a `Public` operation and a declared plugin requirement. `Host` operations cannot satisfy a template call.

Use `ctx.operation(request).await?` on every authoring surface. Await returns
`Response<T>`: read it through Deref, or call `into_owned()` to move the value.
Use `.start().await?` to retain a scoped instance. Built-ins such as StartCapture and
RunProcess use that method without a template `requires` entry. Redundant built-in requirements are compile errors.
The operation macro owns default timeouts, delays and retries. A builder option overrides one
call; retry choices come from settings, parameters or configuration and require
declared replay safety when enabled. Every attempt is recorded and attempt
deadlines are never raised. Import `OperationHost` on carrier hosts; there is no
request-taking `ctx.call`, template `ctx.host()`, builder `.run()` or `.id()`.
Keep the no-argument handler `ctx.call()` metadata accessor and the Dokime host
plugin. Rebuild carrier artifacts together for operation envelope v2.

Contributions name only populated catalogs: `Suites`, `Records`, `Evidence`, `RunStart` and `Settings`. Template records, prompts and streams belong to their template declarations; plugin contributions cover evidence outside those declarations. Observers publish evidence; listeners receive their declared record types.

The plugin declaration implements `dokime::Plugin`; resource ownership implements the shared `dokime::harmos::Lifecycle` separately. Its async `prepare(config, &mut Setup<Self>)` registers shared resources and prepares suites with `dokime::execution::prepare::<Self, Suites>(setup)`. `initialize` activates those resources, followed by serving, `stop` and `cleanup`. Mutable resources use `Arc<tokio::sync::Mutex<T>>` or `Arc<tokio::sync::RwLock<T>>`. Observers acquire measurements through operations using their execution context. Process observers require `#[dokime::observer(interval_ms = …)]` (a constant expression of at least 1 ms). They may implement `fn interval(resources: &ExecutionContext) -> Result<Duration, Error>` beside `observe` to override the declared initial interval; otherwise the declaration applies. Harmos resolves the interval once at activation before initialization and sampling. Manual generic observers declare `Observer::INTERVAL` and inherit the same fallback. Host observer defaults and settings defaults share constants in `settings.rs`. Cadence is absent from static observer inventories. Template-local observer functions keep their separate `interval_ms` declaration.

## Operation Boundaries

Templates and observers import only operations, `domain`, Dokime and the standard
library; they never import driver or library crates. Wrap driver or library work
in an operation and import that operation. Steps sequence operations and may use
returned values for records and criteria; criteria decide from captured evidence.

Keep operations thin and fast. Observers read through operations; return batches
when the acquisition rate needs them so fast streams remain viable.

Each `operations/<domain>/<operation>.rs` is a self-contained vertical slice:
request, response, policy, handler and private helpers live together. It may
import other operations, `domain`, Dokime, std and the driver or library it uses.
`domain/` holds shared data only; do not create `domain/operations/`. Operations
called by other plugins keep their handler and driver imports behind the
`runtime` feature in that same file. Requests and responses contain no driver
types: responses are plugin-owned, serializable wrappers over driver results,
and driver errors become `Refusal`.

Resources belong to the plugin instance, never a template or `domain`. Prepare
and retain them on the plugin through `Lifecycle`, register the same shared state
with `Setup::resource`, and retrieve it in handlers with `ctx.resource::<T>()`.
Use the existing mechanism; if it cannot reach the needed plugin-owned state,
stop and report rather than inventing an API.

Operation calls form a linear tree without cycles: higher-level operations call
lower-level operations, never the reverse. These are authoring conventions, not
a Hub rule or project check.

## Hand-Authored Steps

A hand-authored step reads as a list of operations. Each action is a Title Case
comment, a `let operation` binding and one `ctx.operation`, with a blank line between
actions:

```rust
#[dokime::step(label = "Prepare")]
async fn preparation(&mut self, ctx: &mut StepContext<'_>) -> Result<()> {
    // Enable Remote Control
    let operation = SetChamberSettings::from([("RemoteControl", "true")]);
    ctx.operation(operation).await?;

    // Start Chamber Service
    let operation = StartChamberService {};
    ctx.operation(operation).await?;

    // Wait Until Chamber Is Ready
    let operation = WaitUntilChamberReady {};
    ctx.operation(operation).await?;

    // Read Chamber Temperature
    let operation = ReadChamber {};
    let reading = ctx.operation(operation).await?;

    Ok(())
}
```

For traceability, fetch the data before building and publishing each section:

```rust
#[dokime::step(label = "Traceability")]
async fn traceability(&mut self, ctx: &mut StepContext<'_>) -> Result<()> {
    // Get Diagnostics
    let operation = GetDiagnostics {};
    let diagnostics = ctx.operation(operation).await?;

    // === Device Under Test === //
    let mut section = TraceabilitySection::new("Device Under Test");

    let serial_number = TraceabilityRow::text("Serial Number", &diagnostics.serial_number);
    section.row(serial_number);

    let firmware_version = TraceabilityRow::text("Firmware Version", &diagnostics.firmware_version);
    section.row(firmware_version);

    ctx.traceability().section(section);

    // === Test Bench === //
    let mut section = TraceabilitySection::new("Test Bench");

    let chamber_id = TraceabilityRow::text("Chamber ID", &diagnostics.chamber_id);
    section.row(chamber_id);

    let controller_version = TraceabilityRow::text("Controller Version", &diagnostics.controller_version);
    section.row(controller_version);

    ctx.traceability().section(section);

    Ok(())
}
```

Traceability constructors take the label first, like documents. The ID derives
from that label; `.id(...)` overrides it. Sections use a separate
`section.id("dut");` statement; rows support `row.id("serial_number")`.
Use explicit IDs for label-independent edit keys. The starter pins `bench` and
`target_c` because a later step updates that row. GUI stated groups are scoped
by snapshot identity, and reports retain captured identifiers; neither compares
authored section IDs across executions.

Fetch first, then use the result: call the operation once at the top, name the
result after what it holds, and only read that result in the rest of the step.
Use `?` by default for operation failures in a step.

- Use two comment levels: `// Title Case` labels an action;
  `// === Title Case === //` opens a group of actions.
- Name before use: build a value on one line and hand it over on the next;
  never nest construction in the call that consumes it.
- A variable and its response field share one name. Traceability IDs derive from
  labels using document normalization unless explicitly set with `.id(...)`.
  Pin an ID when edits or external consumers require identity across label changes.
- Make blocks read alike: create a section, add its rows, then publish it at the
  end of that group. Reuse the name `section` in each group.
- Use standard style-guide Title Case for comments and every human-facing label:
  capitalize the first and last word and every major word; lowercase articles,
  coordinating conjunctions and short prepositions in between (`a`, `an`, `the`,
  `and`, `but`, `or`, `nor`, `for`, `of`, `in`, `on`, `at`, `to`, `by`). This includes
  steps, criteria, records, parameters, streams, sections and rows. For example,
  `Number of Rows` and `Wait Until Chamber Is Ready`. Declaration IDs keep their
  established syntax; label-derived traceability IDs use dot-separated lowercase
  components, for example `Serial Number` becomes `serial.number`.
- Use `Group / Action` naming in comments and labels, such as
  `// Check / Number of Rows` and `Setup / Bench`. Descriptions stay sentences.
- A template is the orchestration unit: read top to bottom as one linear flow.
  Repeat steps rather than extract a helper. Keep indentation shallow and loop
  bodies a plain sequence of steps.
- Build documents one statement at a time: create a plot, configure it, then
  attach it.
- Suffix field names with their unit (`duration_s`, `distance_m`); separate
  attributed fields with a blank line.
- Use the same type names in every template file: `Template`, `Parameters`,
  `Row`, `Required`, `Streams` and `Prompts`. Catalog variants match their
  operation names.
- Cleanup attempts everything and logs a warning for each failure.

Use `Plot::xy(label)` for explicit X and Y axes; style defaults to `Line`.
Choose `PlotStyle::Points` for density-grid reduction, or `Line`, `LinePoints`
or `Step` for the ordered envelope.

Plot references are plot-owned Y-axis values in stored units. Use
`plot.reference_band("Limits", lower, upper)`,
`plot.reference_tolerance("Target ± Band", center, tolerance)` and
`plot.reference_line("Limit", value)` with literals or expression handles.
`plot.scale(...)` scales them with the data in live views and frozen reports.
Do not associate references with a quantity or select an X reference axis.

Canonical examples contain no placeholders.

A criterion fetches its data, computes one named check per
`// Check / Title Case` comment, then combines the names in `Ok(...)`:

```rust
#[dokime::criterion(label = "All Measurements Successful")]
fn measurements_ok(ctx: &CriterionContext<'_>) -> Result<bool> {
    let rows = ctx.samples::<Row>();

    // Check / Number of Rows
    let number_ok = rows.len() == ctx.parameters().samples as usize;

    // Check / Measurements Successful
    let measurements_ok = rows.iter().all(|row| row.distance_m.is_some());

    Ok(number_ok && measurements_ok)
}
```

## Layout

```text
src/
  lib.rs                        Plugin Instance And Lifecycle
  domain/                       Shared Data Only
  operations/
    mod.rs                      Provided Operation Catalog
    <domain>/
      mod.rs                    Operation Modules
      <operation>.rs            Request, Response, Policy, Handler And Helpers
  observers/
  listeners/
  records/
  streams/
  settings/
  suites/
    mod.rs                      Suites Catalog
    <suite>/
      mod.rs                    Suite Declaration
      <template>.rs             Template Declaration
```

Use unprefixed suite names and only populated modules. Keep the selected
repository skeleton and all its section markers. Keep declarations explicit,
with no prelude modules or placeholder suites.

The existing host sidecar keeps request definitions in `host/<domain>/mod.rs`:
request structs, ids, policies, responses and contract tests. `host/controls`
shares native state capture, inverse transactions and backend journal callbacks
for radio, adapter and power writes. Construct `RadioSet` with
`plugin_host::domain::controls::{Radio, RadioPower}` (`Radio::Wifi`,
`Radio::Bluetooth`, `Radio::Cellular`; `RadioPower::On`, `RadioPower::Off`).
`RadioState.kind` shares `Radio`; inventory uses
`host::adapters::{AdapterKind, AdapterStatus}`. Native boundary parsers reject
unknown closed-set values with `plugin_host::Error::UnknownValue`; do not add
string request aliases. Runtime implementations
live in parallel `windows.rs` and `unix.rs` modules selected only by
`#[cfg(all(feature = "runtime", target_os = "windows"))] mod windows;` and
`#[cfg(all(feature = "runtime", unix))] mod unix;`. These module gates describe the existing host layout. `host/catalog.rs` assembles collection and stream declarations;
`host/operations.rs` assembles Public queries and controls on both platforms.
`host/observers.rs` selects runtime observers, including stateful native
notification ownership; cancelling its demand activation drops registrations.
Preparation registers mutable collector resources as `Arc<tokio::sync::Mutex<T>>`;
operation policies own collection deadlines and configuration owns probe parameters.


## Run Effects and Start Evidence

`dokime::RunStart<Capture>` registers a default-constructed Host operation returning `dokime::operations::effects::StartEvidence`. The backend calls each enabled provider synchronously before any execution step, qualifies the route with its provider owner, and records the structured JSON snapshot in that execution's evidence. A refused capture refuses execution start. Keep capture read-only.

An operation that changes external state requires `ReadEnvironment`, `PrepareEffect` and `ConfirmEffect` from `dokime::operations::effects`. Read and validate the native prior state, prepare an owner-qualified encoded inverse durably before changing it, then confirm with the actual native completion timestamp. Empty tickets mean a standalone change outside an execution. The backend restores execution effects in reverse order after pass, failure or cancellation and restores persisted pending inverses before startup returns. Failed inverses remain pending and refuse startup; never reset storage to discard them. Native embedding owners use `RuntimeHandle::begin_external_change` and `complete_external_change` with the same journal.

Cancellation must retain native work and its serialization gate until it settles. A plugin retains its generation `Supervisor` during initialization and copies the serving execution scope for journal callbacks, so cancellation of the parent operation cannot strand its completion timestamp. Backend cleanup closes write admission before restoring; it keeps execution capture open through restoration. An inverse is a Host operation callable by the backend, not another plugin. Internal storage remains v1; an incompatible development store requires a fresh root without deleting the old one.


## Host CPU Load Control

`plugin_host::host::cpu::load` owns `LoadHold`, `LoadTarget`, `LoadSummary`,
`LoadControl` and `CpuLoadSettings`. The locked Public operation is
`host.cpu.load.hold`; preserve existing `host.cpu.load` as the capacity/load query.
Use finite `LoadTarget::Percent(value)` in `(0,100]` or positive finite
`LoadTarget::Flops(value)`; `threads: Option<u32>` selects logical cores when absent; explicit counts must fit `1..=logical_cores`.
One shared CPU sampling owner serves telemetry and control; consumers never refresh
CPU deltas. Reuse the calibrated kernel/capacity rather than adding another peak probe.
Controller evidence uses distinct `host.cpu.load.control`, structured units and
`Label [unit]` labels. Percent feedback counts the whole host; FLOP/s counts only
this generator. Timing and gains belong to the CPU Load Control settings section.

Start a suite handle with `ctx.operation(LoadHold { /* request fields */ }).start().await?` (the
scoped equivalent of the runtime builder’s `.start()`), hold, then `.cancel().await?` for a typed
`LoadSummary { target, mean, error_rms, duration_s, samples }`. Step/run cleanup cancels unjoined handles. Stopping workers is restoration,
so no durable effect inverse is needed. The handler uses Harmos cooperative
cancellation and returns the final summary within its configured policy grace.
Use the host CPU control stream as evidence in the consuming template. Combine
its row with the collected streams in one `Evidence` contribution; collection
operations remain in their executable catalog.

## Shared Secrets

```rust
#[dokime::section(label = "Cloud")]
pub struct Cloud {
    pub api_token: dokime::plugin::Secret,
    pub optional_token: Option<dokime::plugin::Secret>,
}
```

These fields name `api_token` and `optional_token` in shared Secret Storage.
The host resolves declarations before startup: missing required identifiers return
`UnavailableSecret(identifier)`; optional missing identifiers decode as `None`.
Add the exact missing identifier and restart Dokime or retry the plugin. There is
no binding, environment source or secret default. Manual `SecretSetting`
declarations likewise use their `id` as the store identifier. Rebuild consumers of
removed `SecretReference` or per-plugin update APIs against the matching SDK.

## Plugin Settings

Declare bounded operational values with `#[dokime::settings]` and labeled
`#[dokime::section]` groups, then contribute `dokime::Settings<Settings>`.
Each numeric setting has a description, validation bounds and units in square
brackets in its label. Resolve once with `config.settings::<Settings>()` in
`prepare` and register that snapshot as a resource. Saved updates apply on restart
or plugin Retry. Host collection cadence is declared in `settings.rs` and read by
each process observer's runtime interval method. Operation policies remain static
Harmos declarations; replay windows remain static publication contracts.
