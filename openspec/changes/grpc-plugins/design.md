# gRPC Plugins Design

## Library State and Ownership

The library pin is owned by the workspace `Cargo.toml`. Protocol generation
refreshes every manifest projection and the vendored protocol inventory from
that revision's archive, sharing the C++ archive checksum input. Generated
bindings remain tracked. Cargo and native SDK locks follow the same revision.

Registry removal, disable and host shutdown detach externally launched plugins;
only registry-spawned children receive Stop. External launchers retain process
exit ownership. Registration restart and remote authoring keep their existing
guards and assertions.

The host composes the library plugin state manager with managed storage recovery,
including State Get and Put operation registration. Plugin state capacity comes
from the product operation message budget. Recovery capture/replay use the
existing storage pending-byte budget and operation page capacity; snapshot
segments use the message budget and the snapshot cadence uses the storage flush
row policy. Snapshot schemas remain at v1. SDK contexts forward the library
StateHandle; local test contexts require a registered product host for durable
state. A stale write revision returns `state_revision_conflict`.

Remote address replacement remains blocked on a library capability. Product
`crates/dokime-operations/src/plugin_policy.rs` currently handles a changed URL
with Delete/Create, which removes the previous entry's durable state. The
library's `crates/harmos-runtime/src/plugin/manager/replacement.rs` restricts
Replace to managed bundles. Product Plugins/Create callers reach the former;
the library manager's Replace path reaches the latter. The clean contract is
one library-owned remote placement update preserving entry identity, settings
and state; retire the product Delete/Create path after that capability lands.
This adoption preserves one external listener across host restart and does not
claim durability across remote address replacement.

Native recovery compaction atomically removes journal entries covered by the
newest published snapshot and older snapshots, retaining the uncovered suffix
and any newer snapshot still awaiting publication. Without a published snapshot,
compaction changes nothing. Publication and compaction preserve the saved
frontier and require no migration or data reset.

The Rust SDK exposes the library Collection directly. The product scheduler in
`sdk/rust/src/stream.rs` remains owned by the host plugin integration worker;
replace that scheduler with the library collection when the host plugin lands.
Both it and the library's `sdk/rust/src/stream.rs` schedule observer windows.
Product Plugin/Observer composition calls the former and library Collection
publications call the latter. Delete the product scheduler after migrating its
consumers; this work does not edit its other owner's files.

The requested direct StateHandle and Collection exports supersede the earlier
Rust worker brief's blanket removal of library-facing APIs for these types only;
there is no product-owned mirror or compatibility layer. Work stays on
`feat/library-state-and-ownership` as this task directs, overriding the project
guidance's older integration branch name.

## Resuming

### Python Peer Operation References

Python callers import a typed `dokime_sdk.operations.Operation` value published
by the callee package, carrying its identifier and request/response types
(including generated protobuf messages). The existing call shape is
`await ctx.operation(reference)(request, policy=optional_override)`. There is no
author requirements list or second peer dispatcher. Before library activation,
the SDK freezes typed references reachable through handler globals, closures,
helper functions, referenced module/class members and reference collections,
including plugin hooks, operations, template hooks and stream handlers. Private
local calls stay local. Conflicting fingerprints fail construction. References
created or injected only after activation fail with a re-registration action;
runtime mutation cannot extend the frozen activation requirements.

The workspace pin `324ad01b0892b1ca6a3bce4bb9525b22974f606b` already supplies
Python Catalog requirements, child carriers and host-registry Public peer routing
with fingerprint admission, inherited scope, callee lanes/policies/deadlines and
cancellation. No library change is required for imported typed references.
The read-only library checkout's HEAD differs; these findings use the pinned
Git objects, not that checkout's current source. Runtime-created references must
match a frozen contract or rebuild and register the plugin again. The library
alone validates child requirements; the product projects its contract errors
into actionable SDK failures without a second admission check.

The execution host, typed catalog/origins, four SDKs, CLI and desktop runtime
embedding are present; tasks.md records remaining behavior and final proof.
The workspace no longer contains the old domain/client/server/loop crates.
Current source is authoritative; the historical wave prerequisites below do not
reopen deleted code or establish passing native proof.

## Ownership of the Remaining Replacement

operation-boundary owns storage access only through in-process Harmos operations,
canonical scopes/cuts, compound lifecycle/seal publication, columnar intake,
retained row watches, transforms, archive fidelity, Store deletion, the typed
facade and product remote wrappers. Its tasks also delete
`protocol/composition.toml` and current internal manager dependencies. Generic
storage/query transforms belong in Harmos; FFT and product plotting/document
presentation remain product-owned. Product clients use product wrappers; Harmos
host operations are not a second remote API in the target.

plugin-boundary owns Dokime plugin contracts, author SDKs, registrar/admission,
registry/settings, installation, bundle format and verification, generator reuse,
metadata inspection and complete protocol export closure. Harmos supplies generic
carrier/runtime helpers. plugin-lifecycle owns paired verbs/hooks, template-scoped
prompts/records and lifecycle catalog declarations. Plugin analyses have been
removed entirely. All generated protocol output remains tracked and is regenerated
through project tasks; build-time generation never authorizes untracking it.

operation-grammar owns resource naming, caller/visibility acceptance and final
inventory review. There is one executor-owned model/host/plugin layout, with caller
filtering independent of visibility. Private stays local and unwired, Protected is
host/declaring-plugin only, and Public peer calls route through the admitted host
registry with matching requirements, inherited scope and cancellation.

## The Flow

```text
Plugin (Managed | Remote)
  └─ catalog: dokime.suites, operations, streams

TestCaseTemplate ── expand (host) ──► parameter sets
parameter set ──── materialize (plugin, pure) ──► TestCase
TestCase ── run (host manager live state) ──► fill Execution
Import ──── import (later admission path) ──► fill Execution
Execution: evidence container, open → closed → sealed
Live run: queued → initializing → stepping → evaluating → cleaning up
          → finished | cancelled | interrupted
```

| Stage | Owner | Notes |
| --- | --- | --- |
| Template, parameter definitions, files, document | Plugin catalog | `dokime.suites` section; no steps or criteria |
| Expand dimensions, filters, overrides, case cap | Host | The planning operation owns expansion |
| `materialize(parameters)` | Plugin | Pure; the host may call it for preview and validation |
| Execution evidence, origin, outcome and seal | Host | Shared evidence model |
| Live run ordering, retries and cancel | Host | Execution manager state, never a stored run or API type |
| Step, criterion, initialize, cleanup bodies | Plugin | Ordinary operations |

## Plugins

```rust
pub enum Plugin {
    Managed(ManagedPlugin),   // bundle: installed, started, supervised by Dokime
    Remote(RemotePlugin),     // URL: the user runs it; Dokime connects
}
```

The variant decides only how the channel is obtained and who owns the
process. Nothing above it branches on the variant.

| | Entry Is | Removed By | States |
| --- | --- | --- | --- |
| Managed | An installed bundle | Uninstalling it | Starting, ready, failed |
| Remote | A URL | Removing it from the list | Connecting, ready, unavailable |

An entry exists until someone removes it; availability is a state. Adding and
removing a remote plugin are operations served to clients, so the settings page and a
script use the same path.

## The Test Case

```proto
syntax = "proto3";
package dokime.v1;

message Metadata {
  string identifier = 1;
  string label = 2;
  string description = 3;
}

message TestCase {
  reserved 1, 2, 3;
  reserved "template", "plugin", "digest";
  map<string, Value> parameters = 4;  // resolved
  repeated Step steps = 5;
  repeated Criterion criteria = 6;
  repeated string streams = 7;        // execution streams the host opens
}

message Step {
  Metadata metadata = 1;              // evidence keys on metadata.identifier
  string operation = 2;               // generated by the SDK, Protected
}

message Criterion {
  Metadata metadata = 1;
  string operation = 2;               // generated by the SDK, Protected
}
```

Definitions are one per verb: `TestCase` for run and `Import` for import.
They freeze into an execution through `Origin`; there is no separate
execution-definition type. `PluginRun` contains the full frozen TestCase,
plugin/template identity, materialization digest and plugin process start time.
`Import` is the frozen import manifest with sources/hashes, claimed importer,
SDK version, processing description and import time. Its detailed admission
contract belongs to `imported-executions`.

```proto
message Origin {
  oneof kind {
    PluginRun plugin_run = 1; // the TestCase, plugin and its start time
    Import import = 2;        // frozen import definition and source manifest
  }
}
message PluginRun {
  TestCase test_case = 1;
  string plugin = 2;
  string template = 3;
  bytes digest = 4;
  google.protobuf.Timestamp started_at = 5; // plugin process identity
}
// Import is the definition/manifest, not a live runtime object.
// Existing execution/call field numbers are assigned during implementation.
```

Preserve the remaining TestCase field numbers and reserve the removed identity
fields above. Plugin calls carry origin beside the full case; both references
must agree. Materialization routing names the template in MaterializeRequest;
the host captures identity/digest from the catalog. No fake case or plugin is
created for imports. These contracts replace the earlier origin sketch.

- Generated operation names only need to be stable within one catalog digest.
  The host materializes again when the digest changed before execution.
- A step's retry count is its operation's retry policy.
- `<template>.materialize`, `<template>.initialize` and `<template>.cleanup`
  have stable identifiers. The last two are optional.

## Execution

An execution is the evidence container: identity, cycle, origin, frozen schemas,
evidence, attributed outcome and seal. Its states are open, closed and sealed.
It knows nothing about steps; step/attempt evidence is interpreted through the
TestCase frozen in PluginRun origin. Every execution freezes its used schemas:
from the plugin catalog for a run, from a script's import-local catalog for an
import. Import catalogs never register plugins or overwrite definitions.

A run is only the execution manager's live state for a TestCase: queued,
initializing, stepping, evaluating, cleaning up, then finished, cancelled or
interrupted. It is not a stored object or API type and exists only while running.
Durable execution evidence and recovery markers do not create a persisted Run.
Build this model and migrate consumers before building the manager; this is
host-wave work, not a dependency supplied later by imported-executions.

```text
host: create open execution with PluginRun origin and frozen schemas
manager: queue its live run, open the test case's streams
manager: initialize → steps (with retries) → criteria → cleanup
manager: close streams and execution evidence, attribute outcome
host: render final report, then publish report and seal
manager: release live run state at finished/cancelled/interrupted
```

Cleanup remains required after successful initialization, including failure or
cancellation. Interactive Next, Run Step and Finish are decisions inside the
live loop. Cancel addresses the execution and is valid only while a run drives
it. SDK suite resources are created on the first operation and released after
cleanup; they are never serialized.

Outcome records how it was reached: not evaluated, evaluated (Passed, Failed and
the existing evaluated kinds), or declared by the importer. No criteria are
accepted on imports. Cycle headline verdicts fold only evaluated outcomes;
declared and unevaluated members are counted and shown separately in cycle
summaries and reports. A cycle with no evaluated result is never Passed. Empty
criterion sets cannot produce an evaluated pass merely by vacuous success.

The backend states each execution's supported start, rerun, cancel, inspect and
export operations for its origin and current state. Desktop shows only those.
Import drafts never enter the live run queue, are not cancel targets and do not
inherit start/rerun actions; their separate operations belong to imported-executions.

Recovery handles interrupted runs by closing their executions, retaining their
interrupted outcome and completing ordinary report/seal work as applicable. It
never closes or finalizes an import draft. Restarting a plugin interrupts its
active runs, closes their executions and seals them and their affected cycles
through normal finalization; drafts and unrelated executions are excluded.

| On reconnect, plugin process `started_at` is | Result |
| --- | --- |
| Unchanged | The step in flight failed; retry it if its policy allows |
| Changed | Interrupt the run, close its execution and finalize its affected cycles |

Rendering and publishing reports are separate for runs and imports. Rendering
accepts a fixed execution snapshot plus a document and returns temporary output
without durable writes or publication. Publication commits a selected report
with its frozen document/revision and seal under the host's durability contract.
Rendering failure publishes nothing. Reusing rendering does not invoke existing
operations that capture or publish reports as a side effect. Import previews,
revision conditions and draft-only visibility are specified by imported-executions.

## Authoring

Python authoring uses the class-based plugin, suite and template declarations in
[Python Project Shape and Client Surface](#python-project-shape-and-client-surface-locked-by-the-owner-2026-11-18).
Steps, criteria, streams and observers stay with their template.

```rust
use dokime::prelude::*;

#[derive(dokime::Message)]
struct Temperature { celsius: f64 }

async fn ramp(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Bench>()?;
    ctx.operation(SetTemperature { target: bench.limit }).await
}

async fn within_limit(ctx: &Context) -> Result<bool> {
    let stats = ctx.statistics("chamber.temperature", "celsius").step("hold").await?;
    Ok(stats.max <= ctx.get::<Bench>()?.limit)
}

fn main() -> Result<()> {
    let temperature = Stream::<Temperature>::new("chamber.temperature", "Chamber Temperature [°C]");
    let observer = Observer::new(&temperature, read_temperature).every_ms(1000);

    let template = Template::new("thermal.cycle", "Thermal Cycle")
        .initialize(initialize)
        .step(Step::new("Ramp to Target", ramp).retries(2))
        .step(Step::new("Hold", hold))
        .criterion(Criterion::new("Temperature Within Limit", within_limit))
        .cleanup(cleanup)
        .stream(temperature)
        .observer(observer);

    let suite = Suite::new("thermal", "Thermal").template(template);
    let plugin = Plugin::new("thermal", env!("CARGO_PKG_VERSION")).suite(suite);

    plugin.serve()
}
```

- Rust authoring is derives for data and ordinary values for everything
  else. A template, a suite and a plugin are built and passed by value;
  there are no attribute macros for them. C# and C++ use the same shape, and
  Python composes `Plugin`, `Suite` and `TestTemplate` classes and uses
  decorators for template handlers and plain operation functions.
- A mistake such as a duplicate identifier is reported when the plugin
  starts, not at compile time.
- Steps and criteria have one fixed signature per language: `ctx` in Rust,
  the instance and `ctx` in Python.
- The SDK derives `materialize` from the registered steps, criteria and
  streams. An author overrides it only when they depend on parameters:

```python
from dokime_sdk.templates import TestTemplate, materialize

class ThermalCycleTestTemplate(TestTemplate):
    @materialize
    def materialize(cls, case, parameters):
        for channel in parameters["channels"]:
            case.step(f"Measure Channel {channel}", cls.measure, channel=channel)
```

## Host Operations for Plugins

Called through `ctx`; each is a Public `dokime.v1` operation the plugin
declares as a requirement. The SDK methods are thin conveniences over
`ctx.operation`.

| `ctx` | Purpose |
| --- | --- |
| `prompt(...)` | Ask the operator and wait for the answer |
| `secret(identifier)` | Read Secret Storage |
| `settings()` | Read the plugin's saved settings when needed. Nothing is injected at start, so the applied-versus-saved comparison and its "unapplied" state are removed |
| `sleep(duration)` | Wait on the host's clock |
| `record(kind, ...)` | Publish a record; returns once the host stored it |
| `records(kind)` | Read the execution's records of a kind (`count`, `last`) |
| `statistics(stream, field=None, step=None)` | Statistics of one column of a stream so far, optionally for one step; the field may be omitted when the stream has one value column |
| `artifact(...)`, file capture | As today's host operations; effects are plugin state |
| `progress(...)`, `log(...)`, traceability, document parts | New small operations replacing evidence-page fields |

```proto
message Statistics {
  uint64 count = 1;
  double min = 2;
  double max = 3;
  double mean = 4;
  double stddev = 5;
  double median = 6;
  double p05 = 7;
  double p95 = 8;
  double p99 = 9;
  double first = 10;
  double last = 11;
}
```

The host computes statistics once per stream and time range and serves every
criterion from that. Queries are scoped to the execution through `ctx`, and
the host records which data each criterion read. The step argument is a
time-range filter using the step's start and end, which the host already
knows. Statistics cover streams only: a numeric series that a criterion
judges belongs in a stream, not in records.

## Streams, Observers and Records

| Declared In | Opened | Closed |
| --- | --- | --- |
| The catalog (plugin-wide, such as host telemetry) | When the plugin connects | When it disconnects |
| The `TestCase` (belongs to an execution) | Before `initialize` | After `cleanup` |

- The plugin sends one sample per message; the host stamps arrival and
  ingests it. Samples lost during a disconnect are accepted. The fastest
  stream in use today publishes every 200 ms, so this is enough; an SDK may
  later group samples under the same authoring API.
- An observer's interval is a declared default or comes from the plugin's
  settings, as today.
- An observer is a function and an interval. The host sends a tick each
  interval and the SDK answers it with the function's value, stamped with
  the tick's time. A stream without an observer is a generator that yields
  samples when the device produces them.
- The host owns orchestration time and `ctx.sleep`, allowing deterministic
  accelerated live-run tests. Evidence separately retains source time, host
  receipt time and durable position; each stream declares its axis. Host arrival
  ordering does not erase source time. Imported UTC normalization belongs to
  `imported-executions`; the host is not the only evidence clock.
- Records are not a stream. A verdict may rest on them, so each is an
  acknowledged host operation on the step's own call.
- Removed: stream leases, publication admission, sample and record delivery
  into a running template.

## Calling Other Plugins

A step calls an instrument operation that another plugin serves, in any
language, as a typed call routed by the host (Harmos `grpc-plugins`, Calls
Between Plugins). The definitions come from one command, which emits `.proto`
files only; each language's own toolchain generates code from them.

```sh
dokime proto export --out proto/                    # dokime.v1
dokime proto export --plugin thermal --out proto/   # one plugin's operations
```

```python
from thermal.v1 import SetTemperature

ctx.operation(SetTemperature(target=85))
```

The export never reads the data folder. It takes the definitions from a
running Dokime as a client, from a plugin URL through `Describe`, or from a
bundle path. Only `dokime server` and `dokime doctor storage` open the data
root, and they find it from the distribution identity beside the executable.

Plugins connect independently and trust each other. A call to a plugin that
is not connected is refused naming it.

## Execution State

Each language has exactly one way to keep state for an execution.

| Language | State Lives In | Why |
| --- | --- | --- |
| Rust | The resource store. The suite type has no fields and steps take only `ctx` | Observers run beside steps and criteria run without the instance, so neither can borrow fields |
| Python | The template instance (`self.chamber = ...`), which the SDK hands to steps, observers and criteria. There is no `ctx.set` or `ctx.get` | Attributes are the natural form and can be added at any time |

```rust
struct Bench { charger: Charger, limit: f64 }

async fn initialize(ctx: &Context) -> Result<()> {
    let charger = Charger::open(ctx.parameter("port")?).await?;
    ctx.set(Bench { charger, limit: ctx.parameter("limit")? });
    Ok(())
}

async fn ramp(ctx: &Context) -> Result<()> {
    let bench = ctx.get::<Bench>()?;
    bench.charger.set(bench.limit).await
}

async fn cleanup(ctx: &Context) -> Result<()> {
    if let Ok(bench) = ctx.get::<Bench>() {         // may never have been set
        ctx.operation(ReleaseCharger { device_id: bench.charger.device_id }).await?;
    }
    Ok(())
}
```

- The Rust store has two verbs. `ctx.set(value)`
  stores a value by its type and replaces an existing one, so a retried step
  is safe. `ctx.get::<T>()` returns it or a named missing-resource error.
- The store is visible to initialize, steps, observers, criteria and cleanup,
  and is cleared when the execution ends. The Python instance has the same
  lifetime.
- It replaces template fields and `Own`, the shared resources returned from
  `initialize`.
- Operations return their value directly (`Result<T>`); steps, criteria,
  initialize and cleanup already do.

## What Stays

- The effects journal, now durable plugin state. It restores external changes
  after a host or plugin crash, which a cleanup operation cannot; the plugin
  applies its inverses in `initialize`.
- Cycle responsibilities in the host: lifecycle, queueing, pause and stop,
  continuation, reruns, sealing and reports. "Cycles only group executions"
  describes what a plugin sees.

## Catalog Sections

Dokime's contributions to a plugin's catalog become `dokime.v1` messages
instead of versioned JSON blobs.

| Today | Becomes |
| --- | --- |
| `dokime.suites` | `dokime.v1` suites: templates, parameters, files, documents |
| `dokime.records`, `dokime.settings`, `dokime.run-start` | Typed `dokime.v1` messages |
| `dokime.telemetry` | Folded into the plugin's stream declarations |
| `dokime.listeners` | Typed `dokime.v1` message. Plugin-wide record listeners are operations the host calls and stay; only delivery into a running template is removed |

## Documents

A document definition is data: one `dokime.v1` message in the plugin's
catalog, the same whatever language wrote it. Each SDK offers builders that
assemble that message, so they stay thin.

```python
temperature = dokime.Stream("chamber.temperature", Temperature, label="Chamber Temperature [°C]")
celsius = temperature.field("celsius")

trend = dokime.plot.time_series(celsius)
spread = dokime.plot.histogram(celsius, label="Temperature Spread [°C]")
section = dokime.Section("Temperature").plot(trend).plot(spread)

document = dokime.Document("Thermal Cycle").section(section)
template = template.document(document)
```

```rust
let celsius = temperature.field("celsius");

let trend = Plot::time_series(celsius.clone());
let spread = Plot::histogram(celsius).label("Temperature Spread [°C]");
let section = Section::new("Temperature").plot(trend).plot(spread);

let document = Document::new("Thermal Cycle").section(section);
let template = template.document(document);
```

- Sections, cards and plots are values, composed and passed by value.
- A data expression is a small tree: a stream field, a record field, an
  aggregate, a transform. It is built by chaining methods that have the same
  names in every language. The host never parses expression text.
- A field reference comes from the stream or record value the plugin already
  declared, so a misspelled field is reported when the plugin starts.
- The builders for message shapes are generated from the `.proto` files
  where the language makes that practical. Only the expression chain and a
  few conveniences are written by hand per language, so a new plot kind or
  transform is one change to the definitions plus regeneration.
- Later improvements land as additions to the same message: the `at`
  transform that reads a stream's value at each record (`telemetry-at-record`),
  and a command that renders a document from a recorded execution while it
  is being authored.

## Protocol Packages

Protocol files are executor-owned under `proto/dokime/{model,host,plugin}/v1/`.
Host callers are explicit and independent of visibility. The former single-package
and client/context face plans are superseded; the current mapping is maintained
in `docs/guides/execution-host.md`. Generated output remains tracked.

## SDKs

Python public APIs always offer a flat form: compose values and submit them to
one call, and explicitly close resources. No API requires a context manager.
Import builders and their one-call submission arrive in `imported-executions`
after the SDK ownership and operation-boundary storage cutovers.

| Half | Uses | Built On |
| --- | --- | --- |
| Plugin | `harmos.v1` transport, `dokime.v1` messages and host calls | The Harmos SDK for the language |
| Client | `dokime.v1` services | Generated stubs plus a thin hand-written layer |

The Python client flow and its bound handles are defined in
[Python Project Shape and Client Surface](#python-project-shape-and-client-surface-locked-by-the-owner-2026-11-18).

- Every operation Dokime serves to clients appears in every language's client
  through generation; a check keeps the stubs in step with the protos.
- Rust absorbs `dokime-client` into the `dokime` SDK's client half; migrate
  CLI and desktop consumers and delete the separate client crate.
- Python is a pure Python package over `grpcio`: one wheel, no native build,
  `Remote` only. It does not start Dokime.
- Authoring parity across languages and between sidecar and guest is a soft
  target.
- Wasm guests are not Dokime plugins in the first version.

### Pilot SDKs

| Language | Folder | Plugin Placement | Distribution |
| --- | --- | --- | --- |
| Rust | `sdk/rust` | Managed and remote | `dokime` crate with plugin and client halves |
| Python | `sdk/python` | Managed and remote | One pure wheel |
| C# | `sdk/csharp` | Managed and remote | NuGet package |
| C++ | `sdk/cpp` | Managed and remote | Source archive with CMake |

Each has both halves and builds on the Harmos SDK for the same language. Go
is outside the pilot.

A plugin project has a strict `dokime.toml` manifest with `[plugin]` metadata,
`dynamic` project metadata, `[compatibility]` and `[scripts]`. `[scripts]` contains
an optional `build` and exactly one native `executable` or `launch` command.
The whole `assets/` folder is bundled; there is no `assets` key.
Paths stay inside the project root. Native executable paths may omit the platform
suffix; Windows resolves the corresponding `.exe` when the unsuffixed file is absent.
`dokime plugin bundle` builds the project and freezes its catalog and manifest
metadata, resolving `dynamic` fields from the language's project file and applying
explicit manifest overrides. The plugin class name supplies its identifier.
It packages the executable and `assets/` or the launch project's inventory after
applying `.gitignore`. Python launch projects include `pyproject.toml` and
`src/<package>/plugin/__main__.py`; the host runs their configured module launch
command through `uv`. Python starters ship no lock file. All four languages support managed bundles.

```sh
dokime plugin bundle ./thermal --out thermal.dokime
```

## One Command-Line Tool and Starters

```sh
dokime new my-plugin --lang rust|python|csharp|cpp
dokime plugin bundle <folder> --out plugin.dokime
dokime plugin inspect --bundle plugin.dokime
dokime dev --plugin <bundle-or-manifest-folder>
dokime installer package
dokime proto export
```

- `cargo dokime` is removed and `crates/cargo-dokime` is deleted. Every
  language uses the one `dokime` tool, and none but Rust needs Cargo.
- `dokime` at `sdk/rust` holds Rust authoring and the client, with protocol-only
  product dependencies, so a plugin or client never compiles the host.
- Owner decision (2026-11-18): Python starters do not embed `uv.lock` or ignore it.
  The first `uv sync` resolves the dependency named by `pyproject.toml` and creates
  the user's lock file, which belongs in their version control. In-repository
  example locks are removed; repository tests install the working-tree SDK.
- Every buildable starter folder, including both `plugin` and `test` (`Plugin`
  and `Test` for C#), owns root `dokime.toml`, `mise.toml`, `.gitignore` and
  `.gitattributes` files. The manifest builds and launches that folder's plugin;
  tasks only name operations the folder supports.
- A starter is an ordinary project in its language, with a mise task that
  wraps its build. `dokime new` fails for a language without an SDK and lists
  the supported ones.
- Concepts have the same names in every SDK: suite, template, step,
  observer, criterion, prompt, record, stream.

| Language | Install |
| --- | --- |
| Rust | `cargo add dokime` |
| Python | `uv add dokime` or `pip install dokime` |
| C# | `dotnet add package Dokime` |
| C++ | CMake `FetchContent` of the versioned source archive, pinned by URL and hash; gRPC and protobuf through a vcpkg manifest in the starter |

## Validation

- Each SDK's sample plugin builds from the `.proto` files alone, so the
  languages build in parallel and none waits for the host build.
- SDK transport conformance stories run in Nightly only. The shared execution-host system target runs on pull requests and in Nightly on Linux and Windows. One story runs against every language: the same suite, written once per
  SDK, executed by the same host, with the same expected evidence. A
  difference between languages is a failure.
- Tests at integration level and above follow a real user journey, one story
  per file. A story reads like a test template: top to bottom, with few or no
  indentation levels, a short comment naming each stage, and every line
  either an action the user takes or a check of what they then see.
- Setup, fixtures and mechanics live in shared modules with one convention.
  A shared module never chooses a scenario or hides an assertion, and a test
  never rebuilds what a shared module provides.
- No test depends on real time. A test waits for a condition or runs on a
  paused clock. A plugin under test can be served in the test's own process
  over an in-memory channel, so a paused clock accelerates the host and the
  plugin together.

## What It Removes

Sizes are of existing paths, not deletion totals; several files are mixed.

| Area | Path | Lines |
| --- | --- | --- |
| Run and signal serving, plugin walker, evidence pump | `crates/dokime-runtime/src/execution/{serving,run,template,unwind}.rs`, `execution/workflow/`, `transport/`, `domains/execution/run/` | about 10,000 |
| Execution hand-off | `ExecutionFixture`, `ProviderExecutorRegistry`, `ExecutionProviderTarget`, provider outcome projections | part of `domains/execution/provider*` |
| Authoring macros for templates, steps, criteria, suites, catalogs and plugins | `crates/dokime-macros/src/{template,suite,catalog,plugin}.rs` | about 1,900 plus their tests |
| Embedded plugins | `crates/dokime-runtime/src/runtime/embedded.rs` and its registration | about 1,200 with tests |
| PyO3 SDK, Python backend, stubs, wheel tasks | `sdk/python/src/`, `sdk/python/python/`, `sdk/python/tests/`, `tools/python/src/dokime_tools/{build,test}/` | about 24,500 |
| Leases, sample and record delivery, replay | `domains/telemetry/leases.rs`, `domains/execution/run/replay.rs` | mixed |
| Tests tied to the old loop | `crates/dokime-runtime/tests/integration/execution/workflow/` | about 6,300 |

Kept and renamed as the execution manager: durable admission and queueing,
live-run transitions, attributed verdict folding, storage finalization, report
intents and cycle accounting (`transactions/provider*`, `provider_finish`). Migrate
their semantics to the evidence-container model rather than retain run phases on
Execution or recover all unsealed objects as interrupted runs.

Kept for managed plugins: install, store, bundles, feeds and installer
entries.

## Operation and Dependency Boundaries

The extracted dokime-storage crate owns the product backend and durable shapes.
operation-boundary owns all remaining direct storage access removal, native cuts,
compound publication, query transforms, typed facade and product wrappers.
plugin-boundary owns public author SDK/contracts and registry policy. Neither the
old post-wave-five extraction schedule nor direct library wire composition remains
a pending task here; storage-interface retains comparative backend proof.

## Crate Structure

This is the locked target, not a claim that the source has reached it. Arrows
below mean "depends on"; lower layers never depend on higher ones.

```text
proto/dokime/v1/ + proto/harmos/        authoritative contracts
        |
        v
dokime-protocol                        generated definitions + light helpers
        ^
        +--- dokime [sdk/rust]          plugin authoring + client half
        |
dokime-storage                        application persistence (after wave 5)
        ^
dokime-operations                     declaration + handler per operation
        ^
dokime-runtime                        host library, main.rs, listener module
        ^
        +--- dokime-cli                commands, hosts through runtime library
        +--- dokime-gui                desktop

harmos-protocol <--- harmos [sdk/rust] plugin authoring + client half
       ^
harmos-domain                         native semantics, not wire mirrors
       ^
harmos-storage <--- harmos-operations <--- harmos-runtime

dokime-storage ---> Harmos storage interface
dokime-runtime ---> harmos-runtime
```

The Dokime host direction is protocol → storage → operations → runtime → CLI
and desktop, from foundation to consumer; a layer may use earlier layers directly.
The SDK's only dependency within the product crate graph is protocol. Ordinary
transport libraries and authoring derives do not introduce host dependencies.

| Crate | Target Ownership |
| --- | --- |
| `dokime` at `sdk/rust` | The product SDK itself, not `dokime-sdk` or another facade. `use dokime::prelude::*;` exposes plugin authoring and the client. Absorb `dokime-client` as its client half, then delete that separate crate. No host, operations or storage dependencies. Harmos uses the same `harmos` identity at `sdk/rust`. |
| `dokime-protocol` | Generated definitions and descriptors from the proto contracts, plus light constructors, operators, `Display` and conversions on those types for product code. No second definition model. Semantics needed by authors in other languages are specified in words and implemented in each SDK. |
| `dokime-operations` | Every host operation's declaration, gRPC mapping, Rust callee policy and handler together, one file per operation. Requests and replies are protocol messages. Handlers use narrow resource interfaces; they never import runtime. Admission validation, parameter expansion and expression evaluation live with the operations applying them. |
| `dokime-runtime` | The host: execution manager, transactions, state, cycle transitions, managed/remote plugin entries, listener and `main.rs`. Transactions, state and cycle transitions remain together. Runtime composes resources and registers handlers directly. Host embedders depend on this library. |
| `dokime-storage` | The retired runtime storage folders have been replaced by the current crate. Persisted shapes belong here; lifecycle decisions remain in runtime. operation-boundary owns canonical operation access and cuts; storage-interface retains backend scale proof. |
| `dokime-cli`, `dokime-gui` | Product commands and desktop composition respectively. The CLI embeds the runtime library, not its binary or a server facade. |

`dokime-domain` dissolves: definitions become protocol messages, host rules move
to their operations, and persisted shapes move to storage. Before storage
extraction, those shapes stay beside the existing storage implementation, never
in a replacement domain crate. `harmos-domain` retains only native state identity,
state/recovery contracts, storage query/schema semantics and validation that are
not wire mirrors; wire definitions and their light helpers belong to protocol.

The separate operations-contract role dissolves. SDKs use protocol, so once no
consumer needs a contract-only surface, remove the operations `runtime` feature.
Harmos's built-in operations retain their declaration-and-handler arrangement;
Dokime migrates area by area, deleting `hosted!` and `hosting::{Host, Serve}`
forwarding for each migrated area. Narrow resources expose owned inputs and
capabilities, not a renamed `RuntimeHandle` or callback into the whole runtime.
Deadlines, backoff and retries remain Rust policy owned by the callee.

There is no `dokime-server` crate or per-service adapter layer in the target.
In wave 2, extend Harmos's client-facing gRPC declarations/codecs to accept
generated protobuf request/reply types with authoritative descriptors, preserving
decoding-memory admission, cancellation, backpressure and streaming envelopes.
This keystone removes `Portable` wire projections, not operation behavior.

In wave 3, migrate `crates/dokime-server/src/grpc/` behavior to ordinary operations:
logs and trace replay/filtering, support bundles, bounded transfers and upload
commit/abort/digests, snapshot paging/frontier closure, watches, plot deltas and
reduction, diagnostics and redaction. Preserve their meaningful tests. Remote
access uses listener configuration and the one client authorization hook;
pairing/key persistence, certificate rotation, settings changes and shutdown
remain implemented responsibilities. Move `host/` startup into runtime `main.rs`
and the listener module; the reusable runtime library owns host lifecycle.
Delete the server crate, its generation/build dependencies and dead conversions
in wave 5 after CLI and desktop consumers have moved.

## Implementation Order

The host/SDK/CLI/desktop replacement and legacy deletions are landed. Complete the
remaining canonical operation/storage, plugin ownership and lifecycle cutovers
under their owning changes; retain this change's missing application behavior,
stable registry replacement and final native proof in tasks.md. No old worktree
name or wave number grants push authority or restores deleted loop consumers.

## Open Points

The remaining application capabilities and proof are in tasks.md. Import policy
questions remain in imported-executions; transparent reconnect remains in
remote-access-simplification. Decoded-memory admission was an additive upstream
follow-up, not a gate on adopting existing wire/queue-bounded clients. The final
registry release pin and all required native proof remain open.

## Current Implementation Evidence

Protocol Origin/TestCase are in `proto/dokime/model/v1/execution.proto`; the host
manager, run controls and scoped samples are in runtime. Settings/progress/logs/
traceability/records/statistics/document parts are typed host operations. Documents
Get declares structure before EvidenceCut/results; rendering does not publish.
SDKs contain generated clients/descriptors and shared thermal host fixtures.
The CLI's embedded protocol export and verified evidence download are present.
Current generated bindings remain tracked; protocol-generation tasks own changes.

Historical source snapshots and answered compiler requests do not establish
current blockers. Recheck remaining behavior and native proof against this tree
and the pinned dependency, retaining the ownership decisions above.

## Genuinely Unfinished Behavior

Aggregates.Get and Media.Captures Create/Get/Watch/Stop currently return
Unimplemented. The product `Effects` operations are removed: effects are plugin
state (see Host Telemetry Plugin Restoration, effect journal), restored by the
plugin's `initialize` before its controls run.
Capture supports local sources but explicitly defers line matchers/markers and
FTP/FTPS; SFTP and source feature-gating need implementation. Host telemetry with
its collectors/controls is absent. The restored storage/desktop benchmarks do not
replace every retired finalization/report workload. Complete all retained behavior
or obtain an explicit scope decision; disabled UI is not feature completion.

Host backup/reset recovery and connected-host build facts/discovery still need
complete public contracts for the corresponding desktop controls. Source-policy
findings require current deterministic reproduction, preserving generator ownership
and meaningful checks rather than restoring deleted APIs or adding unused modules.
Client-disconnection, partial-capture interruption, final expression/plot/report
parity, peer-routing and native installer journeys remain acceptance obligations.

## Validation and Delivery

Use targeted project tasks until all source owners hand off. Final combined proof
requires Windows and Linux, installed four-language SDK consumers, host/CLI stories,
both WebView lanes, fixtures, format/lint/language checks and locked dependency
metadata. Hosted CI/Nightly provide full proof only after authorized delivery.
Nightly publication gates on packaging, not suite results. No source checkbox or
historical Windows result establishes passing final native proof or push authority.

## Authoring Surface (Locked by the Owner, 2026-11-18)

Applies to every SDK language; Python is the reference implementation, the others follow its shape.

1. **Only the Product Package.** Python examples, starters and consumer-facing snippets import SDK symbols only from `dokime_sdk`, `dokime_sdk.operations` and `dokime_sdk.templates`; the other languages use their product SDK namespaces. Nothing an author writes imports or names the underlying library. Each Python submodule owns its `Context`; the root exports none.
2. **Short Prompt.** Examples and starters ask the operator with `ctx.prompt(text, choices=[...])`, which returns the chosen label. Its implicit local identifier is `question`, qualified by the bound template (`capture.confirmation.question` or `accuracy.voltage.question`) in every language. The long host-operation form is not used in examples.
3. **Qualified Identifiers.** A template is `<suite>.<template>` (`accuracy.voltage`). Everything declared inside a template (records, streams, prompts, steps, criteria) is `<suite>.<template>.<name>` on the wire and in evidence. Authors write and pass only the local declaration; the SDK qualifies it.
4. **Generated Identifiers.** An identifier the author does not give is generated from the code name, never from the label: class name minus a trailing `Plugin`, `Suite` or `TestTemplate`, method or attribute name for members; all code-name word boundaries become `_`, and names are lowercased in every language. `ReadVoltage`, `readVoltage` and `read_voltage` all become `read_voltage`. For example, `VoltageAccuracy` becomes `voltage_accuracy`, `HTTPAccuracySuite` becomes `http_accuracy`, and `read_voltage` stays `read_voltage`. An explicit member `identifier=` wins. Class labels derive from class names; a `label` attribute overrides the label. The existing label-derived fallback for steps and criteria is replaced by this rule.
5. **Declarations Live on the Template.** Records are declared on the template class (as streams already are with the stream decorator) and on observers. The plugin-level `records=` argument is removed.
6. **Plain Values.** Records and simple operations accept `str`, `bool`, `int`, `float` (and the language's equivalents); the SDK wraps them in the well-known wrapper messages. Structured rows stay protobuf messages.
7. **No `sections`.** The raw declaration-section list is removed from the author-facing plugin value; the SDK builds every section from typed declarations.
8. **Visibility Is a Type.** `Visibility.PUBLIC` (an enum) replaces module constants.
9. **`requires` Leaves the Author Surface.** Starters and examples never pass it. (Deleting the host-side requirement check is a separate decision pending an inventory; do not touch the host check.)
10. **Python Project Layout.** One `pyproject.toml` project contains `src/<package>/__main__.py` for the client flow and `src/<package>/plugin/__main__.py` for `plugin.serve()`. The plugin and suite classes live in their packages’ `__init__.py`; operations have one module per family and templates one module each under their suite. No root entry scripts or argument parsing are shipped.
11. **Running and Bundling.**
    - `dokime dev --plugin <path>` starts the application with console logging and a temporary data root and attaches one plugin. `<path>` is a bundle file or a project folder containing `dokime.toml`, for every language. A project runs its configured build before starting its executable or launch command. An installed sibling desktop attaches when present and closes with the disposable host; when absent, the host runs alone with a notice. `--no-desktop` starts only the host and plugin without looking for the desktop, and every automated caller passes it. A failing desktop remains an error that names `--no-desktop` as the way to run without it.
    - `dokime plugin dev --executable` is deleted.
    - `dokime plugin bundle <folder> --out <bundle>` accepts a manifest project folder. The manifest's optional `[scripts].build` command runs before description and packaging. Exactly one `[scripts].executable` or `[scripts].launch` is required; the whole `assets/` folder is bundled without an `assets` key. Launch projects apply `.gitignore` to their file inventory. Python projects require `pyproject.toml` and launch their plugin module with `uv`; no lock file is shipped; Python and `uv` on the host are an accepted requirement.
    - Installing a plugin asks nothing; the user decides what to install.

Qualified identifiers change the identifiers stored in evidence; existing roots need a fresh root (development storage policy).

### Python Reference Shape

The locked project section below owns the class and function examples.

The template `Context.prompt` accepts text or a template prompt declaration and returns a choice
label. `prompt` also accepts a full product `Prompt` message for structured fields,
rules and actions. `Context.record(declaration, value)` accepts plain values or
protobuf rows; `Context.records(declaration, step=bound_step).last()` returns the
plain value or structured row and refuses an empty result. `Context.statistics`
takes the stream declaration or bound stream method and an optional bound step.
`Context.operation(Operation(...))(value)` wraps and unwraps simple operation
payloads. Python `int` uses `Int64Value`; bool, str and float use BoolValue,
StringValue and DoubleValue. Records and operations share the same conversion.

A `CaptureSuite` containing `ConfirmationTestTemplate` has template
`capture.confirmation`, step `capture.confirmation.confirm` and record
`capture.confirmation.confirmation`. An `AccuracySuite` containing
`VoltageTestTemplate` has template `accuracy.voltage`; a `read_voltage` method becomes
`accuracy.voltage.read_voltage`. Explicit `identifier="rms"` on that method gives
`accuracy.voltage.rms`. Qualification is stored on the plugin's bound template;
reusing a class in another suite never mutates its local declarations.
Steps and criteria added during materialization use the same single identifier
function and underscore rule to qualify distinct local instance identifiers there;
both unbound and bound methods resolve to their declared handler.

Rust, C#, C++ and command-line starters must follow the same imports/namespaces,
short prompts, code-name identifiers and suite/template qualification,
local record/prompt declarations, primitive wrappers, enum visibility, declaration
references and removal of raw records/sections and example requirement lists.
Python launchers start `<package>.plugin`; `<package>` drives the plugin through a client.
Command-line running, bundling and trust behavior remains point 11's separate owner.

Generated underscore identifiers satisfy the pinned carrier
`crates/harmos-protocol/src/schema/identity.rs` segment grammar without a library
change or a second identifier family. Explicit identifiers remain author supplied.

The Python SDK generator owns `sdk/python/src/dokime_sdk/__init__.py`, composing
client, `Plugin`, `Suite` and `ResultStatus` exports. Operations and templates have
separate public submodules and separate contexts; the root exports no `Context`.
Shared generation owns the remaining descriptor projections and excludes this
root barrel from publication and drift validation; product generation checks
validate it through its SDK owner.

Rust runtime-defined plain operation calls remain an upstream decision: at the
pinned carrier revision, `sdk/rust/src/message.rs:52` requires an
`OperationRequest` with a static `ID`, and `sdk/rust/src/context.rs:223` accepts
only that request type. Product `sdk/rust/src/context.rs:271` therefore cannot
call a runtime-named `Operation` with primitive payload conversion. The carrier
needs a typed call that accepts the operation declaration/identity as a value,
while preserving requirement/fingerprint checks, scope, policy and cancellation;
no product transport bypass or library change is introduced here.

Generated host-request bindings preserve each method's unary or Watch shape,
including the streaming metadata used in requirement fingerprints. A unary view
of a Watch is invalid; automatic requirements must not be narrowed to hide it.

### Authoring Adoption Owners

- Rust: `sdk/rust/src/{lib.rs,context.rs,suite.rs,template.rs,plugin/}` and
  `sdk/rust/examples/` adopt the reference surface and qualify their evidence IDs.
- C#: `sdk/csharp/src/Dokime/{Authoring.cs,Plugin.cs,Context.cs}` and
  `sdk/csharp/examples/` adopt it; `sdk/tests/fixtures/csharp/` follows the same IDs.
- C++: `sdk/cpp/include/dokime/plugin.hpp`, its implementation and
  `sdk/cpp/examples/` adopt it; `sdk/tests/fixtures/cpp/` follows the same IDs.
- Command line: `crates/dokime-cli/{build.rs,src/commands/new.rs}` embeds the new
  Python project files; `src/commands/plugin/` and the application development
  command own point 11 and must invoke the configured Python plugin module.
- Native Python fixture launch: `crates/dokime-runtime/tests/support/mod.rs:91`
  must launch the thermal project’s plugin module through its manifest.
- Thermal evidence selectors: `crates/dokime-runtime/tests/support/thermal.rs`
  and system stories `client_answers_operator_prompts.rs`,
  `client_browses_sealed_history.rs`, `client_runs_thermal_suite_over_grpc.rs`,
  `csharp_plugin_matches_rust_thermal_evidence.rs`,
  `cpp_plugin_matches_rust_thermal_evidence.rs`,
  `python_plugin_matches_rust_thermal_evidence.rs` and
  `thermal_suite_survives_host_restart.rs` must use qualified template members
  and answer the prompt's actual action identifier.

## Python Project Shape and Client Surface (Locked by the Owner, 2026-11-18)

The owner iterated on a scratch project and locked it: "i like the python shape
we have, for me we can lock it". This section is the contract. It supersedes
the earlier Authoring Surface points it contradicts (root `plugin.py` and
`main.py`, top-level `dokime.toml` keys, identity from the plugin's
description alone, imports from `dokime`) and the proposed structure under
Open Points. The other SDK languages take the same shape.

### Project

```text
my-project/
  pyproject.toml        # one project; uv; dev group: ruff, ty; rich for output
  mise.toml             # uv and the dokime tool; build, check, format, dev, run
  dokime.toml
  .env.example
  .gitignore            # full Python rules, *.pair, *.dokime, _input/, _output/
  .gitattributes        # cross-platform Python rules
  assets/               # always bundled; the icon lives here
  src/my_project/
    __init__.py  py.typed
    __main__.py                 # client flow: python -m my_project
    plugin/
      __init__.py               # the plugin class
      __main__.py               # sidecar entry: python -m my_project.plugin
      operations/<family>.py    # one module per operation family
      suites/<suite>/__init__.py      # the suite class
      suites/<suite>/<template>.py    # one module per template
```

- No lock file is shipped. No argument parsing; no root `main.py` or `plugin.py`.
- Every Python and TOML file carries the header banner and the section comments
  the main project uses. No `__all__`.
- Run artifacts are written to `_output/`; `_input/` is the conventional home for
  local inputs. Both are ignored by git.

### Manifest

```toml
[plugin]
label = "My Project"
tags = ["example"]
# Read from pyproject.toml (or Cargo.toml); state a key here to override it.
dynamic = ["version", "description", "authors", "license", "repository"]
# Optional
publisher = "your-organization"
icon = "assets/icon.svg"
readme = "README.md"

[compatibility]
dokime = ">=0.1.5"

[scripts]
build = ["uv", "sync"]
launch = ["uv", "run", "python", "-m", "my_project.plugin"]
```

- Metadata uses the project vocabulary: identifier, label, description, tags.
  The identifier comes from the plugin class name and is not repeated here.
- `[scripts]` holds only what the product runs: `build`, and exactly one of
  `launch` or `executable`. User tasks stay in `mise.toml`.
- The whole `assets/` folder is bundled; there is no `assets` key.
- `description`, `authors` and `license` are required for publication only.

### Authoring

```python
# plugin/__init__.py
from dokime_sdk import Plugin

class ExamplePlugin(Plugin):
    suites = [CaptureSuite]
    operations = [echo]
```

```python
# plugin/suites/capture/__init__.py
from dokime_sdk import Suite

class CaptureSuite(Suite):
    templates = [ConfirmationTestTemplate]
```

```python
# plugin/suites/capture/confirmation.py
from dokime_sdk.templates import Context, TestTemplate, criterion, record, step

class ConfirmationTestTemplate(TestTemplate):
    confirmation = record(bool, label="Confirmation")

    @step("Confirm")
    async def confirm(self, ctx: Context) -> None:
        answer = await ctx.prompt("Continue?", choices=["Yes", "No"])
        await ctx.record(self.confirmation, answer == "Yes")

    @criterion("Confirmed")
    async def confirmed(self, ctx: Context) -> bool:
        return await ctx.records(self.confirmation, step=self.confirm).last()
```

```python
# plugin/operations/echo.py
from dokime_sdk.operations import Context, Visibility, operation

@operation("example.echo", visibility=Visibility.PUBLIC)
async def echo(ctx: Context, request: str) -> str:
    return request
```

```python
# plugin/__main__.py
plugin = ExamplePlugin()
await plugin.serve()
```

- A plugin, a suite and a template are classes that list what they contain. A
  plugin has operations; it never inherits them. Operations are plain functions
  whose request and response types come from their annotations.
- Identifiers and labels derive from class names: strip `Plugin`, `Suite` and
  `TestTemplate`, then snake case. A `label` attribute overrides the label.
- Three import homes: `dokime_sdk` (client, `Plugin`, `Suite`, `ResultStatus`),
  `dokime_sdk.operations` and `dokime_sdk.templates`. Each submodule has its own
  `Context`; the root exports none. Examples import nothing else from the SDK.
- Plugin state is reached from handlers as `ctx.plugin`.

### Client Flow

```python
from dokime_sdk import ResultStatus, connect

async def main() -> None:
    # Initialize Client
    client = await connect()

    # Initialize & Register Plugin
    plugin = ExamplePlugin()
    await client.plugins.register(plugin)

    # Initialize Test Cycle
    started = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    cycle = await client.cycle(label=f"Capture {started}")

    # Initialize Test Cases
    cases = await client.cases.materialize(ConfirmationTestTemplate)

    for case in cases:
        # Initialize Test Execution
        execution = await cycle.execution(case)

        # Run Test Execution
        async for progress in execution.run():
            console.print(f"  {progress.phase}")

        # Seal Test Execution & Report Outcome
        outcome = await execution.seal()

    # Seal Test Cycle
    await cycle.seal()

    # Save Evidence
    evidence = await cycle.evidence()
    (OUTPUT / f"{cycle.identifier}.zip").write_bytes(evidence)

    # Close Client
    await client.close()
```

- Calls return the value itself; no response unwrapping and no wire types in
  user code. A noun on the parent creates the child: `client.cycle(...)`,
  `cycle.execution(case)`, `execution.note(...)`, `execution.artifact(...)`.
  Every handle is bound from creation.
- `client.plugins.register(plugin)` serves the plugin, adds it and returns when
  it is ready; `client.plugins.install(bundle)` installs a bundle. They replace
  `create`. `client.close()` stops what this client serves; entries stay Offline.
- `execution.run()` is `start()` followed by `watch()`; both stay as primitives
  for fire-and-forget and for re-attaching. It is not awaitable on its own.
- `note` and `artifact` exist with the same signature on execution and cycle
  handles and on the step context.
- The client flow reads as commented, self-contained steps with no helpers. The
  operator answers questions in the desktop; an unattended script may set one
  handler before starting.
- Seal semantics are owned by the change `seal-finality`.

### Configuration

- Well-known variables read by the SDK: `DOKIME_REMOTE` (an `http://host:port`
  origin or a `.pair` file); plugin listeners default to `127.0.0.1:0`.
- Precedence: keyword argument, process environment, `.env` in the working
  directory, default. The SDK loads `.env` itself. This replaces the earlier
  rule that no client parses a `.env` file.
- `.env` holds addresses only; secrets stay in Secret Storage.
- `connect(connect_timeout=30.0)` and the command line's `--connect-timeout`
  share one name and unit.

### How the Handles Map to Host Operations

| Call | Host operations today |
|---|---|
| `client.plugins.register` | serve + `Plugins/Create` + watch until ready (composition; readiness should move into the host) |
| `client.cycle` | `Cycles/Create` with automatic continuation off |
| `client.cases.materialize` | `Cases/Create` from the class-derived suite and template identifiers |
| `cycle.execution` | `Executions/Create` with one binding |
| `execution.run` | `Executions/Start` + `executions.Progress/Watch` |
| `execution.seal` | none yet; see `seal-finality` |
| `execution.note` | `evidence.Notes/Create` |
| `execution.artifact` | file upload + `content.Artifacts/Create` |
| `cycle.seal` | `Cycles/Seal` |
| `cycle.evidence` | `Evidence/Archive` + file read |
| `client.close` | stop owned sidecars (entries stay Offline) + close the channel |

Most handle methods compose several operations, so the Python handles are
written by hand first; generating them from operation definitions is deferred
until the other languages follow.

## SDK Surface Specification

This section specifies the implementation target, not landed functionality. The
preceding locked section and `seal-finality` retain precedence. NEW means an
addition to the checked-in SDK, including a new author-facing projection of an
existing transport facility. Source references below describe this checkout
before implementation. The reference is every project file in
`D:/scratch/dokime-python`, including its source tree, configuration, icon and
locally resolved lock; its `.venv/` is an installed environment, not starter
source. Its lock is read for context and is not shipped.

### Python Public Homes

The distribution remains `dokime-sdk`; its only import package is `dokime_sdk`.
These are the complete deliberately exported names at the three author entry
points. Implementation modules begin with `_`; no `__all__`, dynamic attribute
exports, import aliases, public wire modules or alternate import homes are added.
The two names `Context` denote different classes, each owned by its submodule.

```python
# dokime_sdk
from dokime_sdk import (
    Artifact, Case, Cases, Client, Cycle, Cycles, Execution, Executions,
    Failure, Note, Outcome, Plugin, Plugins, Progress, PromptAnswer,
    PromptQuestion, RegisteredPlugin, ResultStatus, Scalar, SdkError,
    Suite, connect,
)

# dokime_sdk.operations
from dokime_sdk.operations import (
    BoundOperation, Context, FailureError, Operation, OperationPolicy, Payload,
    RetryPolicy, SecretDeclaration, SettingDeclaration, SettingGroup,
    SettingValue, StreamSubscription, Visibility, operation,
)

# dokime_sdk.templates
from dokime_sdk.templates import (
    Aggregate, AxisScale, BarArrangement, CaptureGap, CaptureMode,
    CaptureResult, CapturedArtifact, Card, Context, ContourRender, DataExpression, Document,
    Environment, Expression, FileLocation, FileParameter, FileRule, FileRuleAction,
    FileSelection, FileSource, FtpLocation, GridAxisEntry, GridHighlight, LineField, LineMarker, LineMatcher,
    Materialization, MirrorCapture, Parameter,
    ParameterDefinition, Plot, PlotStyle, Prompt, PromptDeclaration,
    Record, RecordRow, RecordsQuery, Reduction, ReferenceBand, ReferenceLine, SampleRow, SamplesQuery,
    Section, Severity, Statistics, Stream, StreamCapture, Summary, Termination,
    TestTemplate, Window, XyInterval, bar, box_plot, contour, criterion, field,
    frequency_series, grid_chart, histogram, materialize, observer, pie,
    plot_card, prompt, record, records_card, step, stream, time_series,
    trajectory, window_observer, xy,
)
```

`Cases`, `Cycles`, `Executions` and `Plugins` are typed resource objects returned
by `Client`; consumers do not construct them or the bound handles themselves.
`Scalar` retains the lossless recursive value domain in today's `values.py:35`:
None, bool, int, float, str, bytes, Fraction, timezone-aware datetime, lists and
tuples of scalars. `Payload` is bool, str, int, float or a protobuf message.
Transport imports used privately by the SDK are not consumer requirements.
Generated operation/resource clients remain internal implementation inputs;
each client-callable operation must still have a generated typed binding. The
public handle compositions below do not create automatic streaming RPC twins.

| Current Name or Inventory | New Home or Removal |
|---|---|
| `dokime._plugin.Plugin`, root `Plugin` | Root `dokime_sdk.Plugin`; replace constructor composition with the class below. |
| `dokime.authoring.Suite`, root generated `Suite` | Root `dokime_sdk.Suite`; one author class, protocol suite internal. |
| `dokime.authoring.Template`, root `Template`, `TestCaseTemplate` | `dokime_sdk.templates.TestTemplate`; protocol messages internal. |
| `dokime.authoring.Case`, root `Case` | `templates.Materialization`; root `Case` becomes the bound client case. |
| `dokime.authoring.suite`, `_SUITES`, single-template suite shortcut | Removed; explicit suite class owns template classes. |
| Root `Context`, `_context.Context` | Separate `operations.Context` and `templates.Context`; remove root export and old unified context. |
| `dokime.operations.{Operation,Visibility,operation}` | Same names in `dokime_sdk.operations`; operation annotations replace request/response decorator arguments. |
| `Step`, `Criterion`, `Sampling` implementation values | Internal; decorators are the public declaration syntax. |
| `authoring.{record,prompt,step,criterion,materialize,stream,observer,window_observer,Record,Stream,PromptDeclaration}` | Same names in `dokime_sdk.templates`. |
| `dokime.parameters.Parameter` builder; root generated `Parameter` | `templates.Parameter` builder and `templates.ParameterDefinition` data, respectively; no name collision. |
| `dokime.documents` and `dokime.plot` public names | Same builder/function/enum names in `dokime_sdk.templates`; generated document/plot messages internal. |
| `dokime.connection.connect` | Root `connect`; connection implementation private. |
| `dokime.client.ClientRoot` | Root `Client`; generated ClientRoot private, no public channel constructor. |
| `dokime.evidence.{Cycle,Execution,Telemetry,TelemetryRow,cycle,execution,executions}` | Bound root `Cycle`/`Execution`, `templates.SamplesQuery`/`SampleRow`, and client resource lookups; delete free selectors and old handles. |
| `dokime.schemas.FrozenSchemaCache` | Internal verified descriptor cache used by all readback handles. |
| `dokime.transfers.{upload_bytes,upload_file,download_report}` | Internal transfers used by plugin installation, artifacts, evidence and generated report bindings. |
| `dokime.values.{encode,decode}` | Internal scalar conversion; `Scalar` has the root home above. |
| Root `FailureError`, `FailureValue` | `operations.FailureError`, root `Failure` data and `SdkError`; no carrier exception escapes. |
| Root `Window` | `templates.Window`, product sampling window. |
| Root generated `ResultStatus` and `RESULT_STATUS_*` | Root `ResultStatus` enum with prefix-free members; remove constants. |
| Root generated policy/settings/secret declarations | `operations` names above; raw protocol representations internal. |
| Root generated `FileSource`, `FileParameter`, `CaptureMode`, `Severity`, `Termination`, `Prompt`, `Card` | Same names in `templates`, product values over the existing contracts. |
| Root generated `DataExpression`, `GridAxisEntry`, `GridHighlight`, `ReferenceLine`, `ReferenceBand` and capture constituent values | Same names in `templates`; these model values supply public builder arguments without a wire package path. |
| All other exports in today's root, `client.py`, `plugin/__init__.py`, `model`, `harmos`, `_generated/_exports/**` | Internal generated inventory; none is an additional public barrel. |
| `Plugin.start`, `Endpoint`, `plugins.create`, raw `records=`/`sections=`/`requires=` author arguments | Removed; `serve`, registered-plugin lifetime, typed class declarations and inferred requirements replace them. |

The last inventory row covering generated exports is exhaustive by exclusion:
every current export not in the three closed lists becomes internal, including
all response wrappers and versioned message namespaces. Rename wire *Python
imports* to `dokime_sdk._wire`; protobuf package names, message full names and RPC
identities remain unchanged. Regenerate `.py`, `.pyi`, gRPC stubs, descriptors and
export projections through tasks; keep their current tracking policy. Update
`hatch_build.py:26` license/notice destinations, wheel package paths, sdist inputs,
type-check/coverage paths, resource lookups, stub inventories and the public
surface guard. A wheel must contain no installable `dokime` import package.

### Python Classes and Declarations

The following signatures are public contracts; comments supply one-line
semantics. `Sequence` inputs are copied to immutable declaration inventories at
binding. Class attributes have the stated empty defaults, never shared mutable
runtime state. SDK-provided constructors take no arguments. Authors may add
their own constructor arguments for devices or other plugin state.

```python
class Plugin:
    identifier: ClassVar[str | None] = None  # Override the class-derived identity.
    label: ClassVar[str | None] = None       # Override the class-derived display label.
    version: ClassVar[str | None] = None     # Override owning distribution metadata.
    suites: ClassVar[Sequence[type[Suite]]] = ()  # Explicit suite classes.
    operations: ClassVar[Sequence[Callable[..., object]]] = ()  # Decorated plain functions.
    settings: ClassVar[Sequence[SettingDeclaration | SettingGroup]] = ()  # Typed saved defaults.
    secrets: ClassVar[Sequence[SecretDeclaration]] = ()  # Declared Secret Storage slots.
    def __init__(self) -> None: ...  # Bind/validate declarations without I/O or author hooks.
    async def initialize(self, ctx: operations.Context) -> None: ...  # Host-invoked activation operation.
    async def finalize(self, ctx: operations.Context) -> None: ...  # Host-invoked settled shutdown operation.
    async def serve(self, address: str | None = None) -> None: ...  # Start, serve until stopped, settle and release owned resources.

class Suite:
    identifier: ClassVar[str | None] = None  # Local suite identity override.
    label: ClassVar[str | None] = None       # Display label override.
    templates: ClassVar[Sequence[type[TestTemplate]]] = ()  # Explicit template classes in order.
    def __init__(self) -> None: ...  # Inert declaration construction.

class TestTemplate:
    identifier: ClassVar[str | None] = None  # Local template identity override.
    label: ClassVar[str | None] = None       # Display label override.
    parameters: ClassVar[Sequence[ParameterDefinition]] = ()  # Pure materialization inputs.
    files: ClassVar[Sequence[FileParameter]] = ()  # Declared file inputs.
    documents: ClassVar[Sequence[Document]] = ()  # Frozen document definitions.
    def __init__(self) -> None: ...  # One instance per execution, shared by its handlers.
    async def initialize(self, ctx: templates.Context) -> None: ...  # Acquire execution state before steps.
    async def cleanup(self, ctx: templates.Context) -> None: ...  # Release partial or complete state after every attempted initialize.
```

The SDK encodes plugin `initialize` and `finalize` as protected operations with
well-known identifiers `<plugin>.initialize` and `<plugin>.finalize` and Empty
requests and responses. Hooks use the ordinary operation context; no SDK stop
callback or parallel activation dispatcher invokes them. `serve()` and
`plugins.register()` share the library-owned listener. The library owns activation,
admission draining, invocation order and settled shutdown, including finalization
after attempted initialization. No hook runs during description or bundling inspection. A register call binds the *same* plugin object
as `ctx.plugin`; execution handlers and observers additionally share the same
execution template instance as `self`. No Python `ctx.set`/`ctx.get` store exists.

Identity follows the preceding locked rule, using the longest matching suffix
first (`TestTemplate`, `Plugin`, `Suite`). Overrides must be nonempty valid local
identity segments; member overrides stay local and are qualified once. Split
acronym-to-word and lowercase/digit-to-uppercase boundaries, preserve underscores,
then lowercase: `HTTPAccuracySuite` → `http_accuracy`, `ExamplePlugin` → `example`,
`ConfirmationTestTemplate` → `confirmation`, `ReadVoltage` → `read_voltage`.
The display label is the suffix-stripped code name split at the same boundaries
and underscores, joined with spaces in style-guide Title Case: `HTTP Accuracy`,
`Example`, `Confirmation`, `Voltage Accuracy`. Explicit label text is retained;
changing it never changes identity. A `CaptureSuite` binding the confirmation
class yields `capture.confirmation` and `capture.confirmation.confirmation` for
its record. Duplicate bound identities fail before serving. Reusing a template
in another suite creates another binding without mutating the class.

Client materialization from a class resolves that class against registrations
owned by the connected client. If it has zero or multiple bound homes, refuse
with an action to register its plugin or supply `plugin=`, `suite=` and the local
template identifier. Never select an arbitrary registry entry. Class discovery
does not run imports recursively or maintain a second global suite registry.

```python
# Decorators Preserve the Annotated Handler Type.
def operation[F: Callable[..., object]](
    identifier: str | None = None, *, visibility: Visibility,
    policy: OperationPolicy | None = None, description: str = "", replay_safe: bool = False,
) -> Callable[[F], F]: ...  # Infer one request and result annotation; identity defaults to function code name.

def step[F: Callable[..., object]](
    label: str, *, identifier: str | None = None, retries: int = 0,
    repeatable: bool = False, policy: OperationPolicy | None = None,
) -> Callable[[F], F]: ...  # Declare an async (self, ctx) -> None execution step.
def criterion[F: Callable[..., object]](
    label: str, *, identifier: str | None = None, policy: OperationPolicy | None = None,
) -> Callable[[F], F]: ...  # Declare an async (self, ctx) -> bool criterion.
def materialize[F: Callable[..., object]](handler: F) -> F: ...  # Mark a synchronous pure materializer.
def record[T: Payload](row_type: type[T], *, label: str = "", identifier: str | None = None) -> Record[T]: ...  # Declare a local record.
def prompt(text: str | Prompt, *, choices: Sequence[str] = (), label: str = "", identifier: str | None = None) -> PromptDeclaration: ...  # Declare a local prompt.
def stream[F: Callable[..., object]](label: str, *, identifier: str | None = None, row: type[Payload] = float) -> Callable[[F], F]: ...  # Declare a row source on a template or plugin.
def observer[F: Callable[..., object]](*, interval_ms: float = 1000, interval_setting: str = "", window_setting: str = "") -> Callable[[F], F]: ...  # Answer host sampling ticks.
def window_observer[F: Callable[..., object]](*, columns: type[Message], interval_ms: float = 1000, interval_setting: str = "", window_setting: str = "") -> Callable[[F], F]: ...  # Answer one host window with typed columns.

class Materialization:
    parameters: Mapping[str, Scalar]  # Resolved immutable input values.
    def step(self, label: str, handler: Callable[..., object], *, identifier: str | None = None, **arguments: object) -> Materialization: ...  # Append a declared step instance.
    def criterion(self, label: str, handler: Callable[..., object], *, identifier: str | None = None, **arguments: object) -> Materialization: ...  # Append a declared criterion instance.
```

Materializers use `def materialize(self, case: Materialization,
parameters: Mapping[str, Scalar]) -> None`; the instance used for this pure call
does not become execution state. Bound and unbound handler references resolve
identically. Default materialization appends declared steps/criteria in definition
order; custom materialization replaces that default. Added instances must use
unique local identifiers while retaining the declared handler identity.
Operations are plain `async def name(ctx: operations.Context, request: Q) -> R`
functions. Resolve postponed annotations with their defining module; reject
missing, ambiguous or unsupported payload annotations before serving. bool,
str, int and float map to BoolValue, StringValue, Int64Value and DoubleValue;
bool is never accepted as int. Primitive records and operation payloads share
one conversion owner. Structured payloads retain their protobuf schema.

```python
class Visibility(IntEnum):
    PRIVATE = 1
    PROTECTED = 3
    PUBLIC = 2

@dataclass(frozen=True)
class RetryPolicy:
    total_attempts: int               # Positive; includes the first attempt.
    initial: timedelta               # Positive host-clock backoff.
    maximum: timedelta | None = None # If supplied, bounded exponential backoff.
    deterministic_jitter: bool = False

@dataclass(frozen=True)
class OperationPolicy:
    timeout: timedelta | None = None
    attempt_timeout: timedelta | None = None
    cancellation_timeout: timedelta | None = None
    retries: RetryPolicy | None = None

@dataclass(frozen=True)
class Operation[Q: Payload, R: Payload]:
    identifier: str
    request: type[Q]
    response: type[R]
    visibility: Visibility
    policy: OperationPolicy | None = None
    description: str = ""
    replay_safe: bool = False

@dataclass(frozen=True)
class SettingValue:
    flag: bool | None = None
    integer: int | None = None
    number: float | None = None
    text: str | None = None
    duration: timedelta | None = None # Exactly one non-None variant, including false/zero/empty text.

@dataclass(frozen=True)
class SettingDeclaration:
    name: str
    title: str
    description: str
    default: SettingValue

@dataclass(frozen=True)
class SettingGroup:
    identifier: str
    label: str
    settings: tuple[SettingDeclaration, ...] # Presentation grouping only; see S3.

@dataclass(frozen=True)
class SecretDeclaration:
    identifier: str
    label: str
    required: bool = True
```

The NEW product policy/settings values use the existing canonical fields and
translate once at the SDK boundary; they do not duplicate policy execution or
setting storage. Validate positive finite policy durations and retry backoff at
operation declaration, naming the operation and invalid field; zero backoff is
invalid. None means inheritance. S1's serial admission field is intentionally
absent until the owner defines its scope; no implementer should infer its shape.

```python
@dataclass(frozen=True)
class Record[T: Payload]:
    row: type[T]
    label: str = ""
    identifier: str | None = None

@dataclass(frozen=True)
class Stream[T: Payload]:
    identifier: str | None = None
    row: type[T] = float
    label: str = ""
    def field(self, name: str = "value") -> Expression: ...  # Select a typed field before binding its qualified identity.

@dataclass(frozen=True)
class PromptDeclaration:
    value: Prompt
    identifier: str | None = None
```

### Python Capability Inventory

| Capability | Current Python Evidence | Expression in the Class Shape / Smallest Addition |
|---|---|---|
| Plugin lifecycle | NEW; `_plugin.py:330` only clears SDK sessions in a private stop callback. | `Plugin.initialize(ctx)` / `Plugin.finalize(ctx)` encoded as well-known operations above; no new public endpoint owner. |
| Deadline and retries | Exists: `operations.py:61` policy, `:91` decorator; `_plugin.py:260` translates step retries. | `policy=` on operation/step/criterion; retain one policy owner, no hard-coded retry delay in the adapter. |
| Serial admission | NEW; `operations.py:61` and `proto/harmos/model/v1/policy.proto:24` have no serial field. | Policy needs shared admission support; no product semaphore copied beside library admission. Owner decision S1 below blocks this extension alone. |
| Operation requirements | Exists: `_plugin.py:78` accepts explicit requirements and `:88` infers all host operations. | Remove author `requires`; infer product host requirements and literal `ctx.operation(Operation(...))` declarations from decorated functions/template hooks. Runtime-selected declarations need S2 below. Preserve the host requirement check. |
| Effects/restoration | Removed: host `Effects/Create`, `Get`, `Set` and every SDK's `prepare_effect` / `confirm_effect`. | A plugin keeps its inverses in durable plugin state and restores them in `initialize` (host plugin `journal.rs`). |
| Execution observers and streams | Exists: `authoring.py:360`, `:371`, `:382`; `_plugin.py:270`. | Decorated methods on `TestTemplate`, per-execution instance, host-owned windows and stream backpressure. |
| Plugin-scoped observers | NEW at the product surface; `_plugin.py:124` discovers streams only in templates. | Permit the same `stream` + observer decorators on `Plugin` methods; context has `plugin`, no execution. Bind at activation, close at stop; no second scheduler. |
| Stream subscriptions | NEW at the product context; no subscribe/watch method in `_context.py:68`. | `operations.Context.subscribe` below delegates admitted library stream demand; no incoming samples injected into a running template. |
| Template setup/cleanup | Exists: `authoring.py:197`, `_plugin.py:194`, `:255`. | Override `initialize` / `cleanup`; cleanup follows failed/cancelled initialization as well as later failures, receives termination, and tolerates partially initialized fields. |
| Parameter kinds/defaults/bounds/units | Exists: `parameters.py:36`, `authoring.py:126`; canonical `model/v1/parameters.proto:25`. | `TestTemplate.parameters`, builder `Parameter`, data `ParameterDefinition`; extend only missing builder factories, retain canonical validation. |
| Settings | Exists: `_plugin.py:82`, `_context.py:166`. | `Plugin.settings` typed declarations; read saved values on demand, no injection or applied/saved duplicate state. |
| Settings groups | NEW; `proto/harmos/model/v1/setting.proto:29` is flat. | Presentation-only `SettingGroup(identifier, label, settings)`; flatten storage keys unchanged, group metadata needs S3 below. |
| Secrets | Exists: `_plugin.py:81`, `_context.py:161`. | `Plugin.secrets`; only declared identifiers are readable through `ctx.secret`, never `.env` values. |
| Capture options | Exists: `_context.py:217` / `:220`, generated `Files.Capture` binding. | Template context `capture`, `start_capture`, `stop_capture`; expose product `FileSource`/`CaptureMode` including stream matchers/markers and mirror interval. |
| Documents and plots | Exists: `documents.py:36`, `plot.py:120`, `authoring.py:127`, `_context.py:188`. | `TestTemplate.documents`, builders in `templates`, `ctx.document` appends parts; canonical models remain internal. |
| Records and streams on template | Exists: `authoring.py:104`, `:144`, `:146`; `_plugin.py:87`. | Preserve local class record values and decorated streams; bind qualified copies, remove raw plugin records. |

`OperationPolicy` retains total invocation timeout, attempt timeout, cancellation
timeout and `RetryPolicy(total_attempts, initial, maximum,
deterministic_jitter)` as typed duration values. No retries without explicit
replay safety. Retried steps declare `policy.retries` with an explicit positive
`initial` backoff; `step(retries=n)` cannot express that policy and is refused
with guidance naming the operation and field. Absent policy inherits the operation owner's configured policy, never a
caller-created timeout. Private execution remains local and still uses the same
policy and cancellation machinery. Visibility must be explicitly PRIVATE,
PROTECTED or PUBLIC; no default or integer/module-constant substitute.

`SettingDeclaration(name, title, description, default)` uses typed
`SettingValue` variants flag, integer, number, text or duration. Groups contain
declarations, not additional stores; duplicate names across groups fail.
`SecretDeclaration(identifier, label, required=True)` describes a slot only.
Parameters accept BOOLEAN, INTEGER, UNSIGNED_INTEGER, NUMBER, TEXT, BYTES,
DATETIME and LIST as the canonical ValueType permits them;
the wire enum is authoritative rather than an SDK-invented type hierarchy.
Defaults are expressions (literal, array, range, arange, linspace, reference,
dynamic default, unary, binary); choices and inclusive min/max must match the
kind, list element kind and min/max items must agree. Invalid ranges, zero range
steps, wrong default kinds and missing required values fail materialization
without acquiring devices. Units are retained strings, never inferred from labels.

### Python Contexts

Constructors are SDK-private. These are the complete convenience method lists;
no transport, channel, library context or untyped `host` escape hatch is public.
`operations.Context` is also used by plugin hooks and plugin-wide observers.
`templates.Context` extends it with execution-only methods. Each is importable
only at its stated public home, even though inheritance shares implementation.
`datetime` values below are timezone-aware UTC; duration floats are seconds.

```python
class Context:  # dokime_sdk.operations
    plugin: Plugin                     # The actual author plugin object.
    attempt: int                       # Admitted attempt number.
    def operation[Q: Payload, R: Payload](self, operation: Operation[Q, R]) -> BoundOperation[Q, R]: ...  # Bind typed local/host/peer dispatch.
    async def sleep(self, duration: float | timedelta) -> None: ...  # Wait on the authoritative host clock.
    async def now(self) -> datetime: ...  # Read the host clock.
    async def cancelled(self) -> str: ...  # Wait for cancellation and return its reason.
    async def secret(self, identifier: str) -> bytes: ...  # Read a declared secret slot.
    async def settings(self) -> dict[str, SettingValue]: ...  # Read the saved plugin settings.
    async def subscribe[T: Payload](self, stream: Stream[T]) -> StreamSubscription[T]: ...  # Open admitted typed stream demand in inherited scope.

class BoundOperation[Q: Payload, R: Payload]:
    async def __call__(self, request: Q, *, policy: OperationPolicy | None = None) -> R: ...  # Apply one-call override and return the decoded result.

class StreamSubscription[T: Payload]:
    def __aiter__(self) -> AsyncIterator[SampleRow[T]]: ...  # Read demand-controlled samples until close/disconnect.
    async def close(self) -> None: ...  # Release this subscription without stopping its provider.

class Context(operations.Context):  # dokime_sdk.templates
    parameters: Mapping[str, Scalar]   # Frozen case inputs.
    execution: str                     # Trusted execution identity; never author-settable.
    termination: Termination | None    # Present during cleanup, including failed initialization.
    async def prompt(self, text: str | PromptDeclaration | Prompt, *, choices: Sequence[str] = ()) -> str: ...  # Return the accepted action label.
    async def record[T: Payload](self, declaration: Record[T], payload: T, *, request_id: str | None = None) -> int: ...  # Append and return the acknowledged ordinal.
    def records[T: Payload](self, declaration: Record[T], *, step: Callable[..., object] | None = None) -> RecordsQuery[T]: ...  # Select records in this execution.
    async def statistics(self, stream: Stream[Payload] | Callable[..., object], field: str = "", *, step: Callable[..., object] | None = None) -> Statistics: ...  # Read typed statistics for the selected stream/step window.
    async def progress(self, completed: int, total: int, *, label: str = "", request_id: str | None = None) -> None: ...  # Revision-checked replacement of execution progress.
    async def log(self, text: str, *, severity: Severity = Severity.INFO, request_id: str | None = None) -> None: ...  # Append attributed execution log evidence.
    async def traceability(self, identifier: str, value: Scalar, *, label: str = "", source: str = "", unit: str = "", request_id: str | None = None) -> None: ...  # Replace one revisioned traceability entry.
    async def document(self, document: str, part: Section | Card, *, section: str = "", request_id: str | None = None) -> None: ...  # Append one document part.
    async def note(self, text: str, *, request_id: str | None = None) -> Note: ...  # Attach an execution note before sealing.
    async def artifact(self, name: str, data: bytes, *, mime: str = "application/octet-stream", request_id: str | None = None) -> Artifact: ...  # Upload and attach bytes before sealing.
    async def capture(self, source: FileSource, output: str) -> CaptureResult: ...  # Capture a host-allowlisted source once.
    async def start_capture(self, source: FileSource, output: str, mode: CaptureMode) -> int: ...  # Start execution-owned continuous capture.
    async def stop_capture(self, capture_id: int) -> CaptureResult: ...  # Settle capture and return artifacts and gaps.
    async def environment(self) -> Environment: ...  # Read host process identity and whether this call is in a step.
```

The template context has all seven base methods plus the execution methods
declared in its block; the operation context has none of the execution-only
methods or fields. In a scoped peer call the operation context inherits the
opaque scope privately; it cannot manufacture an execution context or choose
another execution. Unknown declarations fail locally. Host admission remains
authoritative; SDK declaration validation is not an authorization boundary.
`secret` refuses undeclared names before sending a request. Subscriptions close
with their owning call/activation and forward cancellation/backpressure through
the shared carrier; no local poll/sleep loop is introduced.

Short prompt identity is `<template>.question`. Declared prompts use their bound
identity; full `Prompt` input is copied and bound rather than mutated. Zero
choices implies `OK`; duplicate choices and unrecognized/declined action results
fail with an actionable error. Records, statistics and capture results unwrap
payloads; no response messages escape. `records.last()` refuses an empty result.
An effect inverse is durable plugin state, not an SDK finally block that could be
lost with the plugin process. Capture stop and template cleanup
still settle on cancellation under their operation policies.

### Python Client and Bound Handles

```python
async def connect(
    target: str | Path | None = None, *, connect_timeout: float = 30.0,
    certificate_pem: str | None = None, access_key: str | None = None,
    expected_backend_id: str | None = None,
) -> Client: ...  # Resolve configuration and connect without starting a host.

class Client:
    plugins: Plugins       # Plugin registration/installation owner.
    cases: Cases           # Materialization resource.
    cycles: Cycles         # Cycle lookup/list resource.
    executions: Executions # Execution lookup/list resource.
    async def cycle(self, *, label: str, request_id: str | None = None) -> Cycle: ...  # Create an empty manually advanced cycle.
    def operation[Q: Payload, R: Payload](self, operation: Operation[Q, R]) -> BoundOperation[Q, R]: ...  # Bind one public operation call through the host.
    def on_prompt(self, handler: Callable[[PromptQuestion], Awaitable[PromptAnswer]] | None) -> None: ...  # Replace the single optional unattended answer handler.
    async def close(self) -> None: ...  # Stop owned sidecars, keeping their entries, then close the channel.

class Plugins:
    async def register(self, plugin: Plugin, *, address: str | None = None, request_id: str | None = None) -> RegisteredPlugin: ...  # Serve/register the object and wait for host-owned Ready admission.
    async def install(self, bundle: str | Path | bytes, *, request_id: str | None = None) -> RegisteredPlugin: ...  # Install a managed bundle and return its bound entry.
    async def get(self, identifier: str) -> RegisteredPlugin: ...  # Read an existing entry without taking lifetime ownership.

class RegisteredPlugin:
    identifier: str        # Installed-entry identity, not a transport handle.
    label: str             # Last observed display label.
    version: str           # Frozen declared bundle/plugin version.
    state: str             # Host entry state: Connecting, Ready, Stopping, Stopped, Unavailable.
    async def refresh(self) -> RegisteredPlugin: ...  # Refresh this entry's snapshot.
    async def unregister(self) -> None: ...  # Delete this entry with its current revision and stop only an SDK-owned sidecar.

class Cases:
    async def materialize(
        self, template: type[TestTemplate] | str, *, plugin: str | None = None,
        suite: str | None = None, parameters: Mapping[str, Sequence[Scalar]] | None = None,
        files: Mapping[str, str | None] | None = None, request_id: str | None = None,
    ) -> list[Case]: ...  # Register all host-expanded cases for one explicit template binding.
    async def get(self, identifier: str, *, cycle: str = "") -> Case: ...  # Read a retained case by identity.

class Case:
    identifier: str        # Host-assigned case identity.
    label: str             # Frozen case display label.
    suite: str             # Bound suite identity.
    template: str          # Qualified template identity.
    async def parameters(self) -> dict[str, Scalar]: ...  # Read the case's resolved inputs.
    async def refresh(self) -> Case: ...  # Refresh this bound case snapshot.

class Cycles:
    async def get(self, identifier: str) -> Cycle: ...  # Look up an existing cycle without creating it.
    def list(self, *, text: str = "", page_size: int = 0) -> AsyncIterator[Cycle]: ...  # Iterate every page at one retained cut.

class Executions:
    async def get(self, identifier: str) -> Execution: ...  # Read an existing execution without starting it.
    def list(self, *, cycle: str | None = None, template: str | None = None, state: str | None = None, page_size: int = 0) -> AsyncIterator[Execution]: ...  # Iterate executions at one retained cut.

class Cycle:
    identifier: str        # Bound host identity.
    label: str             # Last observed display label.
    phase: str             # Active, Sealing, Sealed or Seal Failed.
    async def execution(self, case: Case, *, request_id: str | None = None) -> Execution: ...  # Bind exactly one existing case to this cycle.
    async def refresh(self) -> Cycle: ...  # Refresh this cycle's snapshot.
    async def seal(self, *, request_id: str | None = None) -> None: ...  # Seal content, report and archive; refuse running executions.
    async def evidence(self) -> bytes: ...  # Download and verify this sealed cycle's retained archive.
    async def note(self, text: str, *, request_id: str | None = None) -> Note: ...  # Attach a cycle note before sealing.
    async def artifact(self, name: str, data: bytes, *, mime: str = "application/octet-stream", request_id: str | None = None) -> Artifact: ...  # Upload and attach cycle bytes before sealing.

class Execution:
    identifier: str        # Bound host identity.
    cycle: Cycle           # Bound parent, including after a lookup.
    case: Case             # Bound frozen case, including after a lookup.
    async def refresh(self) -> Execution: ...  # Refresh this execution's snapshot.
    async def start(self, *, request_id: str | None = None) -> None: ...  # Admit one automatic run without waiting for its progress.
    def watch(self) -> AsyncIterator[Progress]: ...  # Reattach to current progress through seal without starting work.
    def run(self, *, request_id: str | None = None) -> AsyncIterator[Progress]: ...  # On iteration, start once and then watch.
    async def stop(self, *, request_id: str | None = None) -> None: ...  # Request a stop at the next scheduling boundary.
    async def cancel(self, *, request_id: str | None = None) -> None: ...  # Interrupt active work while preserving cleanup; refuse queued work.
    async def seal(self, *, request_id: str | None = None) -> Outcome: ...  # Return the stable outcome, refusing a running execution.
    async def evidence(self) -> bytes: ...  # Download and verify retained execution evidence.
    async def note(self, text: str, *, request_id: str | None = None) -> Note: ...  # Attach a note while this execution is unsealed.
    async def artifact(self, name: str, data: bytes, *, mime: str = "application/octet-stream", request_id: str | None = None) -> Artifact: ...  # Upload/attach bytes while unsealed.
    def records[T: Payload](self, declaration: Record[T] | str, *, step: str = "", row_type: type[T] | None = None) -> RecordsQuery[T]: ...  # Build a finite decoded records query.
    def samples[T: Payload](self, stream: Stream[T] | str, *, fields: Sequence[str] = (), between: tuple[datetime | None, datetime | None] | None = None, row_type: type[T] | None = None) -> SamplesQuery[T]: ...  # Build a finite decoded sample query.
```

Owner amendment, 2026-11-19: remote gRPC plugins use `register` and managed bundles
use `install`. The CLI exposes `dokime plugin register <address>` and
`dokime plugin install <bundle>`; `plugin add` has no alias. The hand-written
Python collection already implements `register(plugin)`. Its former `upload`
method only refused installation because the product managed-install contract is
missing; remove that method and leave `install` absent until the contract above
is implemented. Do not publish a refusing `install` stub.

Lookup handles retain their client and host identity, never create a second
channel or start a plugin. Never accept a case/handle from another client/host.
`run()` is an async iterator, not a coroutine returning an iterator: no RPC occurs
until iteration starts. Breaking or cancelling observation releases that Watch
only; it does not call Stop or Cancel. A second `run()` is another start request,
subject to host admission, not an implicit rerun. Reattach with `watch()`.
Execution Start always selects automatic mode; interactive command controls
remain generated host operations rather than another starter flow.

Register takes ownership only of the sidecar it starts. Failed registration and
`Client.close` stop that sidecar and never delete the entry: the host keeps a
registered entry and shows it Offline until the plugin registers again, and only
an explicit unregister removes it. Close is idempotent, attempts all owned
cleanup even if one step fails, closes the channel last and reports collected
structured failures.

Owner amendment, 2026-11-20 (registered plugins run by hand): a plugin's own
entry point checks, in order, the managed launch variables (delegate to the
library run entry), then `DOKIME_REMOTE` from the environment or working-directory
`.env` (an empty value is unset): when set, the plugin binds a free loopback port,
connects with the client connection and registers itself through
`dokime.host.v1.Plugins/Create`; otherwise it serves silently. The registration
name is the plugin identifier, so a restart reuses the entry, and
`PluginsCreateRequest.sdk_version` (field 6) carries the product SDK version the
plugin was built with; a host refusal ends the process non-zero with the host's
message. The SDK prints nothing; starters print one authored line. It does not stop a host,
cancel admitted executions or terminate an unowned process. A running execution
may prevent unregister; report the refusal rather than silently cancelling it.
The answer handler is installed before Start; it observes pending prompts,
answers each identity once and is removed on close. The desktop remains the
normal starter prompt UI.

The current pin exposes addressable listeners in all four SDKs. Product C#
registration uses the library StartAsync and WaitAsync APIs directly; no console
capture or second serving task owns that listener. Starter run entries stay
silent and delegate authenticated launch announcement to the library.

The shared description currently carries identifier, version, SDK/protocol
versions and declarations, but has no plugin label, description or icon fields.
Generic declaration contributions are available, but the host has no agreed
plugin-presentation contribution or projection for these values. The SDK and host
owners must select one representation rather than introduce a parallel metadata
family. This is an unresolved product contract, not a missing generic contribution
facility in the library.
Architecture invariant 7 assigns ownership: "Dokime owns plugin contracts, the
four author SDKs and starters". The C# `PluginDescription` in
`sdk/csharp/src/Dokime/AuthorModel.cs` and C++ `PluginDescription` in
`sdk/cpp/include/dokime/plugin.hpp` serve host-free author inspection, while
`proto/dokime/host/v1/plugins.proto` has a separate advertised description used by
host plugin reads. Report this parallel description family to the owner before
replacing it; prefer deriving inspection and host presentation from one advertised
product contribution rather than maintaining independent metadata assembly.
Manifest scripts, bundle inputs and compatibility ranges remain packaging data;
hand-run registration must not read a manifest to obtain presentation values.
The current manifests also carry tags, authors, license, repository, optional
publisher and readme references that the advertised description does not expose.
These catalog values remain part of the presentation-contract decision.

The Rust integration harness uses library Launches reservations to discover the
listener; standard output is ordinary output in every placement.

Execution seal does not start, cancel or wait for active work. Repetition returns
the same outcome. A normal run or cancellation already seals the execution;
notes/artifacts after it go to the cycle until cycle seal. Cycle seal seals
never-started executions as not run and excludes them from pass/fail counts.
Seal Failed retains its cause and permits another seal attempt. Archive and
report bytes are retained at seal, including the report inside the cycle archive;
downloads never regenerate them. All of these are required `seal-finality` host
work, not client-side simulations of finality.

### Host Composition and Mutation Identity

RPC identities here use `package.Service/Method`. Every `dokime.host.v1` name
exists under `proto/dokime/host/v1` unless marked NEW. The current generated
plugin/invocation/clock calls use library host services; they are explicitly
identified as current dependencies, not fictitious product protos. Product
wrappers belong to the `plugin-boundary` / `operation-boundary` owners and must
land before final client proof. There is no SDK bypass to the carrier.

| Public Method | Ordered Operations and Projection |
|---|---|
| `connect` | Open channel; existing `harmos.host.v1.Status/Get` verifies paired host identity; await readiness within the connection budget. Product Status wrapper remains an operation-boundary dependency. |
| `plugins.register` | Start owned listener; current `harmos.host.v1.Plugins/Create` with remote placement; current `harmos.host.v1.Plugins/Watch` until Ready. Target NEW `dokime.host.v1.Plugins/Create` returns only after host-owned Ready; no SDK polling loop. Preserve entry identity/revision and class bindings. |
| `plugins.install` | Upload bytes using `dokime.host.v1.files.Uploads/Create`, `Append` in offset order, `Publish`; NEW `dokime.host.v1.Plugins/Create` with managed bundle reference, host-owned inspection/install/Ready; return one entry. |
| `plugins.get`, registered `refresh` | Current `harmos.host.v1.Plugins/Get`; target NEW `dokime.host.v1.Plugins/Get`; unwrap entry. |
| Registered `unregister` | Get entry/revision, then `dokime.host.v1.Plugins/Delete`; stop SDK-owned sidecar after detach. Missing previously removed owned entry is already detached. `Client.close` only stops owned sidecars and closes the channel. |
| `client.cycle` | `dokime.host.v1.Cycles/Create`, no origins, `continue_automatically=false`, metadata label; unwrap `cycle`. |
| `cycles.get`, cycle `refresh` | `dokime.host.v1.Cycles/Get`; unwrap `cycle` and retain revision. |
| `cycles.list` | `dokime.host.v1.Cycles/List` first page, then remaining pages at the first `resolved_at`, retaining selection/filter/order. |
| `cases.materialize` | Resolve registered class binding (or explicit identifiers); `dokime.host.v1.Cases/Create`, one SelectionPlan selection, `validate_only=false`; bind each returned PlannedCase. |
| `cases.get`, case `refresh`, `parameters` | `dokime.host.v1.Cases/Get` (with parent when known); `dokime.host.v1.Cases/Parameters` at the case's retained cut for parameters. |
| `cycle.execution` | `dokime.host.v1.Executions/Create`, parent=cycle identifier, one ExecutionBinding with case identifier; unwrap the one returned execution. |
| `executions.get`, execution `refresh` | `dokime.host.v1.Executions/Get`; bind the returned cycle/case identities without extra creation calls. |
| `executions.list` | `dokime.host.v1.Executions/List`; follow tokens at the initial retained cut, parent/selection identify the optional cycle/template/state. |
| `execution.start` | `dokime.host.v1.Executions/Start`, automatic mode; retain admitted execution identity, not a local running flag. |
| `execution.watch` | `dokime.host.v1.executions.Progress/Watch`; opening update includes current state. Terminal update becomes `Progress(sealed=True)`; an early end without terminal state fails. |
| `execution.run` | Start once, then Watch as above; the opening snapshot prevents losing a run that seals before Watch subscribes. |
| `execution.stop`, `cancel` | `dokime.host.v1.Executions/Stop` or `dokime.host.v1.Executions/Cancel`; acknowledgement is admission, not a synthesized sealed outcome. |
| `execution.seal` | NEW `dokime.host.v1.Executions/Seal` required by `seal-finality`; returns stable ExecutionOutcome. Today's `Executions/Get` is readback only and cannot substitute for this host action. |
| `cycle.seal` | `dokime.host.v1.Cycles/Seal`; host performs finalization. If acknowledgement precedes settlement, `dokime.host.v1.Cycles/Watch` observes Sealed or Seal Failed before returning. |
| Execution/cycle `evidence` | `dokime.host.v1.Evidence/Archive` for bound subject; `dokime.host.v1.Files/Get` for transfer bounds; `dokime.host.v1.Files/Read` in offset order at returned file identity. Verify length and SHA-256 before returning bytes. |
| Execution/cycle/context `note` | `dokime.host.v1.evidence.Notes/Create` with bound AnnotationOwner; return note. NEW plugin caller admission resolves trusted execution for the context form; current proto allows CLIENTS only. |
| Execution/cycle/context `artifact` | `dokime.host.v1.files.Uploads/Create`, `Append` in offset order, `Publish`, then `dokime.host.v1.content.Artifacts/Create` with bound ArtifactScope. Context omits arbitrary selectors. Return artifact. |
| `execution.records`, query reads | `dokime.host.v1.Executions/Get` for frozen schema binding if uncached, `dokime.host.v1.executions.Schemas/Get` pages at that cut/digest; `dokime.host.v1.evidence.Records/List` pages at their first evidence cut. Unwrap and decode by frozen message name. |
| `execution.samples`, query reads | Same frozen-schema resolution, then `dokime.host.v1.evidence.Samples/List` with raw samples, no bucket; follow retained cuts and unwrap/decode rows, keeping exact timestamps. |
| `Client.operation(op)(value)` | Current `harmos.host.v1.Operations/Get` validates public contract/fingerprint; current `harmos.host.v1.Invocations/Create` routes the encoded unary request and returns decoded response. NEW product wrappers preserve this ordering. No automatic retry of arbitrary mutating operation calls. |
| `Client.on_prompt` | Local handler replacement; while active, `dokime.host.v1.Prompts/Watch`, then `dokime.host.v1.prompts.Answers/Create` for the selected pending prompt; retain answer identity. |

Context clock/settings/discovery/subscription conveniences use admitted carrier
facilities internally; execution methods use their existing product host bindings.
`record`: `dokime.host.v1.evidence.Records/Append`; `statistics`:
`dokime.host.v1.Evidence/Statistics`; `progress`: `executions.Progress/Get` then
`Set`; `traceability`: `evidence.Traceability/Get` then `Set`; `log`:
`evidence.Logs/Append`; `document`: `Documents/AppendParts`; `capture`:
`Files/Capture`; continuous capture: `files.Captures/Start` then `Stop`;
environment: `Backend/Environment`. Each abbreviated name in this paragraph has
the prefix `dokime.host.v1.`. `Documents/AppendParts`, capture, records,
prompt creation and environment are plugin projections, never client admin
escape hatches. Follow caller filtering in generation.

Generate a UUID request identity once per logical mutation when the caller does
not supply one. Keep it and the normalized input stable across retries and surface
it in failure details; a fresh retry with a new identity is a new mutation.
Create cycle/cases/execution, Start/Stop/Cancel/Seal, note, artifact attachment,
progress/traceability/document/record/log mutations and prompt answers use
their request_id fields. File composition derives stable child IDs
`<id>/create`, `<id>/append/<offset>`, `<id>/publish`, `<id>/attach` and
`<id>/abandon`; preserve acknowledged Upload handles/fences while retrying.
On upload failure abandon unpublished bytes through Uploads/Delete; do not delete
a published file that already became evidence. Read calls and Watch use retained
positions/tokens, not mutation identities. Reject mismatched payload replay;
never retry by silently changing a revision. Current plugin Set/Delete lack
request_id; the target product wrapper must carry it plus expected_revision.
Unregister retains its internally generated identity across Client.close retries.
Invocations/Create has no request_id today: replay safety of a direct operation
belongs to its own request contract and policy, not a fake SDK cache.

### Result, Progress, Query and Error Values

All returned values below are product-owned immutable data. Identity strings are
opaque; positions and continuations are SDK-private. Times retain UTC and exact
nanoseconds where Python datetime would round them. Generic `T` is the decoded
payload type. A string query selector without `row_type` decodes through frozen
descriptors and returns a protobuf message; it never guesses the currently
installed plugin's schema. A declaration selector uses its declared primitive or
message type and validates it against the frozen binding.

```python
class ResultStatus(IntEnum):
    UNSPECIFIED = 0
    PENDING = 1
    PASSED = 2
    FAILED = 3
    BLOCKED = 4
    CANCELLED = 5

@dataclass(frozen=True)
class Outcome:
    status: ResultStatus             # Evaluated/declared status, PENDING if unevaluated.
    attribution: str                 # evaluated, declared or not_evaluated.
    reason: str                      # Host-provided explanation.
    termination: Termination | None  # Present for an unevaluated run.
    importer: str | None             # Present only for declared imported outcomes.

@dataclass(frozen=True)
class Progress:
    phase: str                       # Queued, Initializing, Stepping, Evaluating, Cleaning Up, Sealed.
    completed: int                   # Host progress value.
    total: int                       # Zero means no denominator was supplied.
    label: str                       # Host progress label.
    sealed: bool                     # Terminal observation, never called finished/ended.

@dataclass(frozen=True)
class Note:
    identifier: str
    text: str
    author: str | None
    created_at: datetime
    cycle: str
    execution: str | None

@dataclass(frozen=True)
class Artifact:
    identifier: str
    name: str
    mime: str
    size_bytes: int
    sha256: bytes
    cycle: str | None
    execution: str | None

@dataclass(frozen=True)
class PromptQuestion:
    identifier: str                  # Pending prompt instance identity, not declaration identity.
    text: str
    choices: tuple[str, ...]
    execution: str

@dataclass(frozen=True)
class PromptAnswer:
    choice: str                      # One supplied choice label.

@dataclass(frozen=True)
class RecordRow[T: Payload]:
    identifier: str
    sequence: int
    timestamp_ns: int
    received_at_ns: int
    record_type: str
    schema_version: int
    source: str
    value: T

@dataclass(frozen=True)
class SampleRow[T: Payload]:
    identifier: str
    sequence: int
    timestamp_ns: int
    received_at_ns: int
    stream: str
    schema_version: int
    source: str
    value: T
    values: Mapping[str, Scalar]     # Selected typed scalar columns, preserving exact integers.

class RecordsQuery[T: Payload]:
    def pages(self, *, page_size: int = 0) -> AsyncIterator[list[RecordRow[T]]]: ...  # Iterate a finite selection at one evidence cut.
    async def read(self, *, limit: int = 0) -> list[RecordRow[T]]: ...  # Collect up to limit rows; zero uses the host page bound.
    async def count(self) -> int: ...  # Return the selected count, independent of page length.
    async def last(self) -> T: ...  # Return the selected last value, refusing an empty selection.

class SamplesQuery[T: Payload]:
    def pages(self, *, page_size: int = 0) -> AsyncIterator[list[SampleRow[T]]]: ...  # Iterate raw rows at one fixed evidence cut.
    async def collect(self, *, max_rows: int = 100_000) -> list[SampleRow[T]]: ...  # Refuse rather than silently truncate at max_rows.
    async def to_polars(self, *, max_rows: int = 100_000) -> polars.DataFrame: ...  # Optional extra; preserve typed columns and nanosecond time.
    async def to_pandas(self, *, max_rows: int = 100_000) -> pandas.DataFrame: ...  # Optional extra; preserve integer/exact values.

@dataclass(frozen=True)
class Statistics:
    count: int
    min: Scalar
    max: Scalar
    mean: Scalar
    stddev: float
    median: Scalar
    p05: Scalar
    p95: Scalar
    p99: Scalar
    first: Scalar
    last: Scalar
    sum: Scalar

@dataclass(frozen=True)
class CaptureResult:
    artifacts: tuple[CapturedArtifact, ...]
    gaps: tuple[CaptureGap, ...]

@dataclass(frozen=True)
class CapturedArtifact:
    artifact: str
    size_bytes: int

@dataclass(frozen=True)
class CaptureGap:
    path: str
    offset: int
    reason: str                      # Truncated, Replaced, Vanished or Resume Unsupported.

@dataclass(frozen=True)
class Environment:
    backend_pid: int
    in_step: bool

@dataclass(frozen=True)
class Failure:
    classification: str              # Permanent, Transient, Cancelled, Timed Out or Refused.
    code: str
    message: str
    action: str
    operation: str
    plugin: str
    cause: Failure | None
    details: Mapping[str, str]
    diagnostics: str

class SdkError(Exception):
    failure: Failure                 # Original classification, causal chain and next action.
    request_id: str | None           # Retain this identity when retrying an unchanged mutation.
    def __init__(self, failure: Failure, *, request_id: str | None = None) -> None: ...  # Represent an SDK/client failure without leaking transport types.

class FailureError(SdkError):
    pass                             # The product exception author handlers raise for structured failure.
```

Invalid local declarations/configuration raise ValueError or TypeError before
I/O; local file operations retain OSError subclasses. Host/transport/connection,
integrity, resynchronization and missing-schema failures become SdkError with an
action, preserving remote details and causes. asyncio task cancellation retains
CancelledError and still releases owned observers/resources. SDK errors never
turn an unevaluated or declared result into an evaluated pass. Not run is an
unevaluated attribution with host termination/reason, not a new pass/fail enum.
Progress uses only seal vocabulary. The `seal-finality` owner renames
ProgressWatchResponse.finished to sealed at field 5 and regenerates the bindings;
there is no dual-field adapter or public finished/ended spelling.

Builder names moved from `documents.py` and `plot.py` keep their existing full
method signatures and value semantics, including Document(label), Section(label),
Summary(), Expression(value), Plot(value), enum-only plot options and owned
`message` values for canonical document definitions. No second document AST or
plot evaluator is introduced. `ParameterDefinition` exposes exactly the fields
of the canonical Parameter message (metadata, type, unit, required, group,
expression, choices, min, max, expansion, element_type, min_items, max_items,
runtime_writable); the protocol representation stays internal.

```python
class Parameter:
    def __init__(self, identifier: str, label: str, kind: str) -> None: ...  # Start a typed definition with an explicit kind.
    @classmethod
    def number(cls, identifier: str, label: str) -> Parameter: ...  # Floating numeric input.
    @classmethod
    def count(cls, identifier: str, label: str) -> Parameter: ...  # Unsigned integer input.
    @classmethod
    def flag(cls, identifier: str, label: str) -> Parameter: ...  # Boolean input.
    @classmethod
    def sweep(cls, identifier: str, label: str, values: Iterable[float]) -> Parameter: ...  # Cartesian expansion of numeric inputs.
    @classmethod
    def numbers(cls, identifier: str, label: str, values: Iterable[float]) -> Parameter: ...  # One list-valued numeric input.
    def unit(self, unit: str) -> Parameter: ...  # Retain a unit string.
    def default(self, value: Scalar) -> Parameter: ...  # Set a typed literal expression.
    def range(self, minimum: float | None = None, maximum: float | None = None) -> Parameter: ...  # Set inclusive numeric bounds.
    def items(self, minimum: int | None = None, maximum: int | None = None) -> Parameter: ...  # Set list-length bounds.
    def build(self) -> ParameterDefinition: ...  # Return an independent validated definition.
```

Other canonical kinds are available through the explicit-kind constructor and
ParameterDefinition fields; no factory per wire variant is required. Record
values expose `row`, `label`, `identifier`; stream values expose the same fields
and `field(name: str = "value") -> Expression`. `Window` is the sampling grant
with start, period and count; read queries use `between`, avoiding two public
Window type families under one name. Documents keep local declaration references
until binding; final frozen stream/record selectors use qualified identifiers.

Capture input values retain the current product schema with keyword constructors:
`FileSource(file=None, tree=None, newest=None)` selects exactly one FileLocation
or FileSelection; `FileLocation(local=None, ftp=None)` selects exactly one local
path or FtpLocation; `FtpLocation(url, secret_id=None)`; `FileSelection(root,
rules=())`; `FileRule(glob, action)` uses FileRuleAction.INCLUDE/EXCLUDE.
`CaptureMode(stream=None, mirror=None)` selects exactly one `StreamCapture(
matchers=(), markers=())` or `MirrorCapture(interval_s=None)`.
`LineMatcher(pattern, stream, fields=())`, `LineField(name, type)` and
`LineMarker(pattern, record_type)` retain their declared typed extraction.
Here LineField.type is one of the same canonical parameter-kind strings, not a
second kind enum. These constructor parameters and fields are complete;
the SDK rejects multiple/absent oneof alternatives and invalid modes before I/O.
Source validation and execution-owned settlement remain host policy. Plugins may
capture files from any host folder; the capture-folder allow-list is removed.
No local SDK file-watcher or polling capture implementation is added.

### Configuration Loading

`connect` resolves DOKIME_REMOTE from an explicit argument, the process environment,
then `.env` directly in the working directory. A connection without a nonempty
target refuses with guidance to set DOKIME_REMOTE; no default host exists.
There is no parent-directory search, import side effect, process-environment
mutation or starter loader.

The blocking plugin entry delegates launch credentials to the library before
reading pairing configuration. Otherwise a nonempty DOKIME_REMOTE selects
self-registration, and an absent or empty remote selects standalone serving.
Programmatic registration takes an optional listener address and otherwise binds
`127.0.0.1:0`; listener binding reads no environment configuration. An explicitly
empty listener address refuses. The reference `.env.example` intentionally
contains an empty remote placeholder.

Accepted `.env` grammar is deliberately small and identical across languages:

```text
file        = UTF-8 [BOM] lines, LF or CRLF
line        = horizontal-space* [assignment | comment] horizontal-space*
comment     = '#' any-characters-to-end-of-line
assignment  = key horizontal-space* '=' horizontal-space* value [inline-comment]
key         = [A-Za-z_][A-Za-z0-9_]*
value       = unquoted | single-quoted | double-quoted
unquoted    = characters through line end, trimmed at both horizontal-space edges
inline-comment = horizontal-space+ '#' any-characters-to-end-of-line
single-quoted = "'" literal-characters-except-quote-or-newline "'"
double-quoted = '"' characters-or-escapes '"'
escape      = '\\' | '\"' | '\n' | '\r' | '\t'  # Double Quotes Only
```

In unquoted input, `#` starts a comment only at the beginning of the value or
after horizontal whitespace; a literal `#` in an address/file name otherwise
remains literal. Quoted values preserve internal whitespace and `#`; after their
closing quote allow only horizontal space and an optional comment. Empty quoted
and unquoted values are accepted by the parser, then validated by the selected
configuration field. Reject duplicate keys, unmatched quotes, unsupported escape
sequences, NUL, multiline values, shell `export` prefixes and lines without `=`.
No interpolation, shell execution, backticks or `$VAR` expansion occurs. Parse
unknown well-formed keys but ignore them; only the two well-known address keys
configure the SDK. A missing default file is fine; malformed UTF-8, unreadable
existing files and malformed lines fail with path, line number and correction.
There is no secret-setting API in this parser and no `.env` secret import.

`connect_timeout` is a positive finite float in seconds, default 30.0, owned by
connect, with no new environment variable. Preserve current ordered `.pair`
endpoint attempts and certificate/identity checks; this section does not invent
the read policy still open in `remote-access-simplification`. Direct remote
origins use http with an explicit host/port and no path, credentials, query or
fragment; paired endpoints may use pinned HTTPS. Explicit certificate/key/host
identity inputs are for direct endpoints; a `.pair` file supplies them together
as one trusted tuple (do not mix sources). Relative pair paths resolve from the
working directory. Managed listener coordinates delivered by the host's private
launch protocol override interactive address discovery internally; they are not
another user-visible environment variable.

### Exact Project Manifest

The top-level TOML keys are exactly `plugin`, `compatibility`, `scripts`.
Reject unknown tables/keys at every level, duplicate TOML keys and wrong types.
`plugin` and `scripts` are required tables; `compatibility` is optional. There
is no schema-version key, language key, identifier key or assets key.

| Key | Type / Presence | Validation and Meaning |
|---|---|---|
| `plugin.label` | Optional string | Nonblank if supplied; otherwise class-derived label. |
| `plugin.version` | Optional string, required after resolution | Valid semantic version; overrides described project version for the bundle. |
| `plugin.description` | Optional string | Nonblank if supplied; required for publication. |
| `plugin.authors` | Optional array of tables `{name: string, email?: string}` | At least one author if present, nonblank name, nonblank email if supplied; required for publication. |
| `plugin.license` | Optional string | Nonblank SPDX expression; required for publication. |
| `plugin.repository` | Optional string | Absolute HTTP(S) URL if supplied. |
| `plugin.dynamic` | Optional array of distinct strings, default `[]` | Only version, description, authors, license, repository; resolve each requested missing key from the native project file. |
| `plugin.tags` | Optional array of distinct nonblank strings, default `[]` | Preserve declared order. |
| `plugin.publisher` | Optional string | Nonblank publisher identity, independent of class identity. |
| `plugin.icon` | Optional relative file path string | Confined existing regular file, included in bundle; assets/icon.svg in starters. |
| `plugin.readme` | Optional relative file path string | Confined existing UTF-8 regular file, included in bundle. Omit until a README exists. |
| `compatibility.dokime` | Optional string | Valid nonempty version requirement; absent means no declared minimum. |
| `scripts.build` | Optional nonempty array of strings | Program plus argv, no shell interpolation; first element nonblank, no NUL. Run once before inspection/dev/bundle from project root. |
| `scripts.executable` | Optional relative file path string | Exactly one executable/launch; resolve platform suffix as today; require built regular file. |
| `scripts.launch` | Optional nonempty array of strings | Exactly one executable/launch; program plus argv, validated like build. |

Static fields take precedence even if also named in dynamic; listing such a key
is permitted so users can override one value without editing the list. An absent
requested project field fails with its native file/key and an action to set that
field or a static plugin value. Values not requested are not implicitly imported.
No code execution, MSBuild evaluation or package installation is used to read
metadata. Choose the native file associated with the configured plugin executable
or launch project; ambiguity is an error, never directory iteration order.

For Python choose root pyproject.toml for a uv Python launch; for Rust choose
root Cargo.toml and its selected bin's owning package. For C# match the launched
DLL filename or native executable stem to one project's literal AssemblyName
(defaulting to its csproj stem); read only that project. The prescribed C#
starter sets AssemblyName explicitly. This resolves the two executable projects
without an extra manifest selector key. Refuse zero/multiple matches and ask for
static metadata if the source cannot be identified. C++ starters resolve all
metadata statically and never acquire dynamic fields from SDK tooling projects.

| Dynamic Key | Python `pyproject.toml` | Rust `Cargo.toml` | C# Plugin `.csproj` |
|---|---|---|---|
| version | `project.version` | `package.version` | `Project/PropertyGroup/Version` |
| description | `project.description` | `package.description` | `Project/PropertyGroup/Description` |
| authors | `project.authors` tables | `package.authors` strings, split terminal `Name <email>` into the canonical author table | `Project/PropertyGroup/Authors`, semicolon-separated nonblank names |
| license | `project.license` SPDX string | `package.license` | `Project/PropertyGroup/PackageLicenseExpression` |
| repository | `project.urls.Repository` (case-sensitive) | `package.repository` | `Project/PropertyGroup/RepositoryUrl` |

Rust `{workspace=true}` reads the corresponding `workspace.package` key from the
owning Cargo workspace; do not mistake the SDK workspace's version for a standalone
starter's version. Python dynamic build-backend metadata and legacy license tables
are refused for these fields: supply a static manifest value. C# reads only one
unconditional literal value for each key in the configured plugin project; repeated
conflicting values, conditions, imports or `$(...)` expressions require static
manifest values. Native starter layouts below make that file unambiguous. C++ has
no dynamic source mapping: write these fields statically; reject dynamic requests
without one of the three supported native metadata sources. Do not parse CMake as
a second metadata language.

Bundle identity is the described class-derived plugin identifier (respecting its
explicit class override), never package/project name, manifest label or publisher.
Build, inspect without initialize, resolve manifest metadata, then freeze the
description/catalog and manifest into one bundle. Version resolution order is
static plugin.version, requested native project.version, described plugin version;
refuse if none exists or is invalid. An explicit manifest version legitimately
overrides the description for the frozen bundle; installed identity/version must
reflect that resolved value consistently, not two competing version fields.
Python's described version comes from the distribution owning the plugin class
using package metadata, or an explicit class version; source-only plugins without
either require a manifest version for bundling and an explicit class version for
direct registration. Rust supplies its package version, C# its literal package
version, and C++ its statically configured project version to the plugin value.

After build, revalidate path confinement and symlinks. Include the entire
`assets/` tree, including entries ignored by `.gitignore`; a missing assets
directory is fine. Include an icon/readme outside assets explicitly if referenced.
Native bundles contain the resolved executable plus assets and referenced metadata
files. Put runtime companions (for example C# assemblies/runtime files) inside
assets during build, with the executable launch arrangement referring to those
bundled paths; do not retain arbitrary asset-root lists. Launch bundles contain
the source inventory filtered by project `.gitignore`, plus mandatory manifest,
native project metadata and assets. A required launch input ignored by the project
is an actionable error, not silently omitted. Never include `.git/`, `.venv/`,
local `.env`, credentials, `_input/` or `_output/`. Preserve the user's source
uv.lock after first sync and include it when present in a launch bundle; the
*starter archive* ships none. Packaging never silently deletes the user's lock
or data. Files escaping the project via traversal/symlinks are refused.

### Starter Manifest Migration

Keep both existing embedded example roots per language; each is one independent
project containing plugin and client entries. This does not multiply SDK packages.
Every listed `dokime.toml` replaces its whole old top-level schema with these
three tables. Common metadata below is shown once and applied to each named file;
the table fixes every per-file script value. Assets are implicit in every row.

```toml
[plugin]
label = "Starter Project"
tags = ["example"]
dynamic = ["version", "description", "authors", "license", "repository"]
icon = "assets/icon.svg"

[compatibility]
dokime = ">=0.1.5"

[scripts]
# The Exact Per-File Values Are Listed Below.
```

| File | New `[scripts]` Content |
|---|---|
| `sdk/python/examples/starter/dokime.toml` | `build = ["uv", "sync"]`; `launch = ["uv", "run", "python", "-m", "starter_project.plugin"]` |
| `sdk/rust/examples/starter/dokime.toml` | `build = ["mise", "run", "build"]`; `executable = "target/debug/starter-project"` |
| `sdk/csharp/examples/Starter/dokime.toml` | `build = ["mise", "run", "build"]`; `launch = ["dotnet", "assets/plugin/Example.Plugin.dll"]` |
| `sdk/cpp/examples/starter/dokime.toml` | `build = ["mise", "run", "build"]`; `executable = "assets/plugin/dokime-plugin"` |

C++ replaces `dynamic` with explicit version="0.1.0", description, authors,
license="Apache-2.0" and repository fields; publisher/readme remain optional
everywhere. All project metadata inputs acquire the five actual fields used by
dynamic; placeholder identities use `Your Name` and `your-organization` only.
C# uses the explicit plugin csproj named below as its native metadata source,
stages its DLL/deps/runtimeconfig and dependencies together in assets/plugin and
keeps the client output separate. This removes the previous Release-path asset
list; implementation proof uses Debug builds. C++ companions likewise belong in
assets with an executable-relative runtime lookup. Python starter files retain
the reference banners, section markers, rich output, .env.example, ignore rules,
py.typed and uv_build source package. Move the reference class/function bodies
unchanged apart from project package names; add no argument parsing or loader.

### Rust Project and Surface

Existing `sdk/rust/src/plugin/mod.rs:95` is an inert value builder; `suite.rs:46`
and `template.rs:196` compose declarations by value. `operation.rs:59` already
supports typed primitive operations and `:80` policies; `context.rs:358` and
`:373` support typed records. `client.rs:53` and generated `client/resources.rs`
provide low-level typed calls, not the locked bound handles. NEW: class-name
marker construction, complete suffix derivation, separate context homes,
connect/configuration, registration lifetime and all bound handle compositions.
The runtime-named operation call still needs the upstream typed call capability
recorded above; do not hide it with a product transport implementation.

```text
Cargo.toml  dokime.toml  mise.toml  .env.example  .gitignore  .gitattributes
assets/
src/lib.rs                         # Exposes the project's plugin declaration.
src/main.rs                        # Plugin binary, no arguments.
src/bin/client.rs                  # Second binary in the same Cargo package.
src/plugin/mod.rs                  # ExamplePlugin marker and plugin() composition.
src/plugin/operations/mod.rs       # Explicit catalog composition.
src/plugin/operations/echo.rs      # One operation family.
src/plugin/suites/mod.rs           # Explicit suite catalog.
src/plugin/suites/capture/mod.rs   # CaptureSuite marker and suite value.
src/plugin/suites/capture/confirmation.rs  # Template and its records/handlers.
```

Keep the native product-owned Registrar/Registration boundary; module catalog
values register through it privately or explicitly. No auto-registration on
import. `Plugin::of::<T>(version)`, `Suite::of::<T>()` and `Template::of::<T>()`
use the marker type's code name for identity and display label. `.identifier(...)`
and `.label(...)` override explicitly. Add these constructors rather than a
Plugin/Suite/TestTemplate derive or an attribute macro. Existing lower-level
explicit identity overrides use `.identifier(...)` rather than a parallel
constructor family. A template marker is stateless; execution state stays in the one
context store via `set(value)` and `get::<T>()`. Plugin-lived state uses its owned
state value and a typed `ctx.plugin::<T>()` borrow. Data alone uses derives such
as `Message`, `Parameters`, `Settings`, `Record` and `Row`; ordinary value
builders are consumed by value. Runtime validation of duplicate declarations and
unsupported values is acceptable. Operation handlers implement the existing
typed Payload/handler bounds; no catalog proof trait is introduced.

```rust
// Project Declaration in plugin/mod.rs
use dokime::{Plugin, Result, Visibility};
use dokime::operations::{Context, Operation};
use crate::plugin::suites::capture::suite;

pub struct ExamplePlugin;

pub fn plugin() -> Result<Plugin> {
    let echo = Operation::new("example.echo", Visibility::Public, echo);
    let plugin = Plugin::of::<ExamplePlugin>(env!("CARGO_PKG_VERSION"))
        .suite(suite()?)
        .operation(echo);
    Ok(plugin)
}

async fn echo(_ctx: &Context, request: String) -> Result<String> {
    Ok(request)
}
```

```rust
// Capture Suite and Template (Bodies Live in Their Respective Modules).
use dokime::{Result, Suite};
use dokime::templates::{Context, Criterion, Record, Step, Template};

pub struct CaptureSuite;
pub struct ConfirmationTestTemplate;

pub fn suite() -> Result<Suite> {
    let confirmation = Record::<bool>::of(ConfirmationTestTemplate::confirmation, "Confirmation");
    let template = Template::of::<ConfirmationTestTemplate>()
        .record(confirmation)
        .step(Step::new("Confirm", ConfirmationTestTemplate::confirm))
        .criterion(Criterion::new("Confirmed", ConfirmationTestTemplate::confirmed));
    Ok(Suite::of::<CaptureSuite>().template(template))
}

impl ConfirmationTestTemplate {
    pub fn confirmation() -> Record<bool> {
        Record::of(Self::confirmation, "Confirmation")
    }

    async fn confirm(ctx: &Context) -> Result<()> {
        let answer = ctx.prompt("Continue?", &["Yes", "No"]).await?;
        ctx.record(&Self::confirmation(), answer == "Yes").await?;
        Ok(())
    }

    async fn confirmed(ctx: &Context) -> Result<bool> {
        ctx.records(&Self::confirmation()).step(Self::confirm).last().await
    }
}
```

```rust
// src/bin/client.rs
use std::fs;
use dokime::{Result, ResultStatus, connect};
use dokime_example_test::plugin::plugin;
use dokime_example_test::plugin::suites::capture::confirmation::ConfirmationTestTemplate;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize Client
    let client = connect().await?;

    // Initialize & Register Plugin
    client.plugins().register(plugin()?).await?;

    // Initialize Test Cycle
    let cycle = client.cycle("Capture").await?;

    // Initialize Test Cases
    let cases = client.cases().materialize::<ConfirmationTestTemplate>().await?;
    for case in cases {
        // Initialize Test Execution
        let execution = cycle.execution(&case).await?;

        // Run Test Execution
        let mut progress = execution.run();
        while let Some(update) = progress.next().await? {
            println!("{}", update.phase);
        }

        // Seal Test Execution & Report Outcome
        let outcome = execution.seal().await?;
        println!("Passed: {}", outcome.status == ResultStatus::Passed);
    }

    // Seal Test Cycle
    cycle.seal().await?;

    // Save Evidence
    let evidence = cycle.evidence().await?;
    fs::create_dir_all("_output")?;
    fs::write(format!("_output/{}.zip", cycle.identifier()), evidence)?;

    // Close Client
    client.close().await?;
    Ok(())
}
```

The plugin binary constructs `plugin()?` and calls `run()` with no
argument parsing; its existing synchronous process boundary may block until
stop. Native run spelling follows the owning plugin-lifecycle contract.
`connect()` has `connect_with(ConnectionSettings)` for explicit target, timeout
and authentication overrides; settings fields mirror Python keyword names with
Duration for connect_timeout. Programmatic registration accepts a listener address;
the blocking run entry selects its mode only from the environment.
No library mutates process environment. Methods are snake_case; async I/O returns
`Result<T>`, ProgressStream.next returns `Result<Option<Progress>>` and owns its
Watch only. Root Client, Cycle, Execution, Case, RegisteredPlugin, Outcome and
ResultStatus use the same fields/verbs as Python; borrowed accessors expose
identifier/label, and `plugins()`, `cases()`, `cycles()`, `executions()` return
bound resources. Nondefault mutation/query arguments use operation-specific
values passed by value to `_with` methods; defaults implement only the meaningful
empty override. No generic Options hierarchy or runtime builder type growth.

### C# Project and Surface

Existing `sdk/csharp/src/Dokime/Authoring.cs:78` has suite/step/criterion
attributes, `:258` reflection-based templates, `:497` suite values;
`Plugin.cs:11` still has a sealed constructor-composed plugin. `Context.cs:58`
has primitive records and `:81` short prompts. Generated `Generated/Client.g.cs`
owns low-level clients; it is not the target handle layer.
NEW: inheritable author bases, distinct contexts, annotated plain static operation
functions, bound handles, connect/configuration and registration lifetime.

```text
Example.slnx
src/Example/Example.csproj             # Shared author code library.
src/Example/Plugin/ExamplePlugin.cs
src/Example/Plugin/Operations/Echo.cs
src/Example/Plugin/Suites/Capture/CaptureSuite.cs
src/Example/Plugin/Suites/Capture/ConfirmationTestTemplate.cs
src/Example.Plugin/Example.Plugin.csproj  Program.cs  # Plugin executable.
src/Example.Client/Example.Client.csproj  Program.cs  # Second executable.
dokime.toml  mise.toml  .env.example  assets/  .gitignore  .gitattributes
```

For both SDK starter roots use `Example` as the shared namespace and project
prefix, with distinct AssemblyName values from the manifest table. The plugin
csproj is the only metadata source; its Version/Description/Authors/
PackageLicenseExpression/RepositoryUrl are literal properties. Both executable
projects reference the shared library; neither duplicates its source or uses
custom include paths. Build tasks stage the plugin output in assets/plugin and
client output in its ordinary build directory. No source generator is needed
for the class model; attributes and startup reflection validate it once.

```csharp
// Plugin/ExamplePlugin.cs
using Dokime;
using Example.Plugin.Operations;
using Example.Plugin.Suites.Capture;

public sealed class ExamplePlugin : Plugin
{
    public override IReadOnlyList<Type> Suites => [typeof(CaptureSuite)];
    public override IReadOnlyList<Delegate> Operations => [(Func<Dokime.Operations.Context, string, Task<string>>)Echo.Call];
}

// Plugin/Operations/Echo.cs
using Dokime.Operations;

public static class Echo
{
    [Operation("example.echo", Visibility = Visibility.Public)]
    public static Task<string> Call(Context context, string request)
    {
        return Task.FromResult(request);
    }
}

// Plugin/Suites/Capture/CaptureSuite.cs
using Dokime;

public sealed class CaptureSuite : Suite
{
    public override IReadOnlyList<Type> Templates => [typeof(ConfirmationTestTemplate)];
}

// Plugin/Suites/Capture/ConfirmationTestTemplate.cs
using Dokime.Templates;

public sealed class ConfirmationTestTemplate : TestTemplate
{
    public static readonly Record<bool> Confirmation = new("Confirmation");

    [Step("Confirm")]
    public async Task Confirm(Context ctx)
    {
        var answer = await ctx.PromptAsync("Continue?", choices: ["Yes", "No"]);
        await ctx.RecordAsync(Confirmation, answer == "Yes");
    }

    [Criterion("Confirmed")]
    public Task<bool> Confirmed(Context ctx)
    {
        return ctx.Records(Confirmation, step: Confirm).LastAsync();
    }
}
```

`Plugin`, `Suite`, `TestTemplate` are root/root/Templates author bases with
Identifier and Label virtual overrides and class-derived defaults. Plugin
Settings/Secrets and Suite.Templates are explicit collections. Plugin StartAsync/
StopAsync and template InitializeAsync/CleanupAsync have the Python lifetimes;
Ctx.Plugin is the actual ExamplePlugin. Operations are static methods declared
in the plugin collection, not inherited service methods. All asynchronous
methods accept a final optional CancellationToken; parent cancellation releases
observation without implicitly cancelling host executions. Product
Dokime.Operations.Context and Dokime.Templates.Context have the same method
split as Python. Attribute suffixes are implicit (`Step` means StepAttribute),
not extra alias types. Existing typed data attributes remain for payload schemas.

```csharp
// Example.Client/Program.cs
using Dokime;
using Example.Plugin;
using Example.Plugin.Suites.Capture;

// Initialize Client
var client = await Client.ConnectAsync();

// Initialize & Register Plugin
await client.Plugins.RegisterAsync(new ExamplePlugin());

// Initialize Test Cycle
var cycle = await client.CycleAsync(label: "Capture");

// Initialize Test Cases
var cases = await client.Cases.MaterializeAsync<ConfirmationTestTemplate>();
foreach (var testCase in cases)
{
    // Initialize Test Execution
    var execution = await cycle.ExecutionAsync(testCase);

    // Run Test Execution
    await foreach (var progress in execution.RunAsync())
    {
        Console.WriteLine(progress.Phase);
    }

    // Seal Test Execution & Report Outcome
    var outcome = await execution.SealAsync();
    Console.WriteLine($"Passed: {outcome.Status == ResultStatus.Passed}");
}

// Seal Test Cycle
await cycle.SealAsync();

// Save Evidence
var evidence = await cycle.EvidenceAsync();
Directory.CreateDirectory("_output");
await File.WriteAllBytesAsync($"_output/{cycle.Identifier}.zip", evidence);

// Close Client
await client.CloseAsync();
```

The plugin executable is only construction plus `await plugin.RunAsync()`.
Client methods use PascalCase with Async for Task-based I/O: RegisterAsync,
UploadAsync, CycleAsync, MaterializeAsync, ExecutionAsync, StartAsync, WatchAsync,
RunAsync, StopAsync, CancelAsync, SealAsync, EvidenceAsync, NoteAsync,
ArtifactAsync, GetAsync, ListAsync and CloseAsync. RunAsync/WatchAsync/ListAsync
return IAsyncEnumerable of product values directly; unary calls return Task<T>
or Task for no result, evidence is byte[]. Outcome/Progress/Note/Artifact fields
match Python with PascalCase properties. Explicit target and timeout live in
ConnectionSettings passed to ConnectAsync, timeout is TimeSpan; shared grammar
reads working-directory .env at connect/serve/register and never modifies the
environment. No `using` block is required for resource lifetime.

### C++ Project and Surface

Existing `sdk/cpp/include/dokime/plugin.hpp:124` has materialization, `:148`
templates, `:246` single-template suite construction and `:254` plugin values;
`context.hpp:132` has prompts and `:134` primitive records. Generated
`include/dokime/_generated/client.hpp` plus `src/client.cpp` provides synchronous
resource calls, file upload and frozen schemas, not bound handles. NEW: author
classes with distinct suite/template composition, separate context homes,
configuration, handle compositions and owned registration. Keep C++20 ordinary
classes and value composition; no new reflection or coroutine framework.

```text
CMakeLists.txt  CMakePresets.json  vcpkg.json  dokime.toml  mise.toml
.env.example  .gitignore  .gitattributes  assets/
src/plugin.hpp  src/plugin.cpp
src/plugin/main.cpp                  # Plugin executable, no arguments.
src/plugin/operations/echo.hpp  echo.cpp
src/plugin/suites/capture/capture.hpp  capture.cpp
src/plugin/suites/capture/confirmation.hpp  confirmation.cpp
src/client/main.cpp                  # Second executable in the same CMake project.
```

The project declares one author-code library, a dokime-plugin target and a
dokime-client target, both linking the author library and installed Dokime::Cpp.
Headers use natural namespace qualification and using declarations only in .cpp
files; no namespace/type aliases or private SDK/library includes are needed.
Class-name identity is supplied by `Plugin::from<T>`, `Suite::from<T>` and
`Template::from<T>` through the existing SDK code-name facility; explicit string
overrides still validate. The plugin base owns its author object, declarations
and state. Declaration functions return complete values, rather than exposing
SDK internals or two registration engines.

```cpp
// Bodies Are Shown Together; Place Them in the Layout's Owning Headers/Sources.
#include <dokime/operations.hpp>
#include <dokime/templates.hpp>
#include <dokime/plugin.hpp>
#include <string>

namespace example {
class ConfirmationTestTemplate : public dokime::templates::TestTemplate {
public:
  inline static const dokime::templates::Record<bool> confirmation{"confirmation", "Confirmation"};

  void confirm(dokime::templates::Context &ctx) {
    const auto answer = ctx.prompt("Continue?", {"Yes", "No"});
    ctx.record(confirmation, answer == "Yes");
  }

  bool confirmed(dokime::templates::Context &ctx) {
    return ctx.records<&ConfirmationTestTemplate::confirm>(confirmation).last();
  }

  static dokime::templates::Template definition() {
    auto value = dokime::templates::Template::from<ConfirmationTestTemplate>();
    value.record(confirmation);
    value.step(dokime::templates::Step::method<&ConfirmationTestTemplate::confirm>("Confirm"));
    value.criterion(dokime::templates::Criterion::method<&ConfirmationTestTemplate::confirmed>("Confirmed"));
    return value;
  }
};

class CaptureSuite : public dokime::Suite {
public:
  static dokime::Suite definition() {
    auto value = dokime::Suite::from<CaptureSuite>();
    value.template_value(ConfirmationTestTemplate::definition());
    return value;
  }
};

std::string echo(dokime::operations::Context &ctx, std::string request) {
  return request;
}

class ExamplePlugin : public dokime::Plugin {
public:
  ExamplePlugin() : dokime::Plugin(dokime::Plugin::from<ExamplePlugin>("0.1.0")) {
    suite(CaptureSuite::definition());
    operation(dokime::operations::operation<std::string, std::string>(
        "example.echo", dokime::operations::Visibility::Public, echo));
  }
};
}
```

```cpp
// src/client/main.cpp
#include <dokime/client.hpp>
#include <fstream>
#include <filesystem>
#include <iostream>
#include "../plugin.hpp"

int main() {
  // Initialize Client
  auto client = dokime::connect();

  // Initialize & Register Plugin
  client.plugins().register_plugin(example::ExamplePlugin{});

  // Initialize Test Cycle
  auto cycle = client.cycle("Capture");

  // Initialize Test Cases
  const auto cases = client.cases().materialize<example::ConfirmationTestTemplate>();
  for (const auto &test_case : cases) {
    // Initialize Test Execution
    auto execution = cycle.execution(test_case);

    // Run Test Execution
    auto progress = execution.run();
    while (const auto update = progress.next()) {
      std::cout << update->phase << '\n';
    }

    // Seal Test Execution & Report Outcome
    const auto outcome = execution.seal();
    std::cout << "Passed: " << (outcome.status == dokime::ResultStatus::Passed) << '\n';
  }

  // Seal Test Cycle
  cycle.seal();

  // Save Evidence
  const auto evidence = cycle.evidence();
  std::filesystem::create_directories("_output");
  std::ofstream output("_output/" + cycle.identifier() + ".zip", std::ios::binary);
  output.exceptions(std::ios::badbit | std::ios::failbit);
  output.write(reinterpret_cast<const char *>(evidence.data()), evidence.size());
  output.close();

  // Close Client
  client.close();
}
```

The plugin executable constructs ExamplePlugin and calls run with no arguments.
C++ retains its existing blocking I/O idiom: unary methods return product values,
evidence is vector of bytes, Watch/Run return an owning ProgressStream with
`std::optional<Progress> next()`. This is observation of asynchronous host work;
it does not introduce a second SDK executor. Explicit Start plus Watch supports
fire-and-forget and reattachment. The C++ spelling of register is register_plugin
because `register` is a reserved keyword; all other verbs retain snake_case:
upload, cycle, materialize, execution, run, seal, evidence, note, artifact, close.
Lookup/list/readback/control methods have the same argument defaults and product
values as Python, with optional/string/vector/chrono equivalents. Product Error
exceptions preserve Failure data; destructors release observations, and explicit
close reports cleanup errors. ConnectionSettings values override working-directory
.env/process configuration under the shared grammar; chrono duration represents
connect_timeout. No user code owns grpc contexts or library call types.

### Implementation Inventory and Ownership

The order is (a), then parallel Python (b)/(c) and manifest (d), then Python
consumers (e), then native SDKs (f)/(g)/(h) once the Python compositions are proven.
Native SDKs may proceed independently of one another. These are source/proof
items, not a declaration that implementation has happened. The rename is a
serial filesystem prerequisite: its destination paths are handed off once,
before any semantic writer starts. It is not concurrent permission for (a) to
edit files later owned by another item. Subsequent owners below are disjoint;
generated roots have one owner, never handwritten edits from another worker.
The owner coordinates coupled host work in its existing changes rather than
assigning the same protocol/runtime files to SDK workers.

Generation has a concrete upstream prerequisite for (a): the pinned shared
generator's `harmos_tools/build/resources/configuration.py:36` has only
`namespace`, and `sdk.py:73`, `:177` uses that same value for both Python package
paths and protocol selection. Add an independent Python import-package setting
in that generator and publish/pin it through the generator owner before rename
proof. Keep the protocol namespace `dokime`; do not set namespace to dokime_sdk,
copy the renderer into this repository or post-process generated output by hand.
This is an implementation capability request, not another SDK surface decision.

| Item | Exclusive Paths After Rename / Work and Deletions | Guard and Required Proof |
|---|---|---|
| (a) Python Package Rename Alone | Move `sdk/python/src/dokime/**` to `sdk/python/src/dokime_sdk/**` mechanically, update its self-imports only; own `sdk/python/{pyproject.toml,hatch_build.py}` and rename-specific `tools/python/src/dokime_tools/build/{protocol.py,protocol_callers.py}`, `tools/python/src/dokime_tools/build/wheel.py`, `tools/python/src/dokime_tools/test/wheel.py`. No class/handle semantics. Hand off the destination tree before (b)/(c)/(e). | Existing root public-surface guard must scan the new tree and reject the old import path; installed wheel contains LICENSE/NOTICE, py.typed and stubs, imports only the new package. Minimal rename consumer/stub/generation checks pass before semantic cutover. No compatibility package or alternate import path. |
| (b) Python Authoring Classes and Contexts | Own `sdk/python/src/dokime_sdk/{_plugin.py,_context.py,_payloads.py,_authoring.py,operations.py,templates/}`; move old authoring/documents/plot/parameters implementations into those owned private/template files. Own authoring-focused `sdk/python/tests/unit/test_authoring.py`, `tests/integration/{test_plain_operations.py,test_materialize_parameterized_steps.py,test_documents_survive_wire_roundtrip.py}`. Delete old `authoring.py`, old public plugin barrel, unified root context, `_SUITES`, suite decorator/shortcut, endpoint API, constructor-only composition and explicit decorator request/response arguments. | Typed class/function declarations, separate contexts and ctx.plugin; deterministic identifier/reuse/duplicate/primitive tests; initialize failure/cancellation still runs cleanup. Focused SDK type/build and owned tests, plus public surface and strict installed-wheel stub checks after (e) publishes roots. No duplicate policy or stream scheduler. S1–S3 remain explicit blockers only for their capabilities. |
| (c) Python Client Handles and Configuration | Own `sdk/python/src/dokime_sdk/{_client.py,_connection.py,_configuration.py,_evidence.py,_transfers.py,_schemas.py,_values.py,_errors.py}` and any private subdirectories below those boundaries; move old connection/evidence/transfers/schemas/values code there. Own `sdk/python/tests/unit/{test_frozen_schemas.py,test_exact_values.py}` and client-focused `tests/integration/{test_connection.py,test_client_controls_remote_plugins.py,test_reports_read_fixed_storage_file.py}`, `tests/system/test_evidence_queries.py`, new client/configuration tests. Delete free selector functions, separate old Cycle/Execution wrappers and public connection module. | Deterministic fake-service composition traces, mutation replay/lost-reply cases, cancellation without execution cancellation, close ownership, fixed-cut paging, frozen decoding and digest verification. Type/stub checks; focused client tests; real host proof waits for named host dependencies, never patched SDK finality. |
| (d) Command-Line Manifest, Bundles and Embedding | Own `crates/dokime-packaging/src/project.rs`, affected packaging manifest/bundle sources/tests, `crates/dokime-cli/{build.rs,src/commands/new.rs,src/commands/plugin/,src/commands/dev.rs}`, relevant CLI units/system stories, and the four starter `dokime.toml` paths above. Own shared CLI fixture/tooling manifest consumers outside SDK trees. Delete old top-level manifest keys, `assets` lists, root-script embedding assumptions, argument-based plugin launches and old-schema fixtures. SDK workers do not edit these manifests. | `check:repository`, focused `dokime-packaging`/`dokime-cli` build and units, exact embedded file inventories, all four created starter bundle/inspect/dev command-line stories with `--no-desktop`; static/dynamic precedence and native metadata errors, confinement, asset completeness and stable bundle identity/version. Existing build task remains the command runner. |
| (e) Python Starter, Examples, Generation and Consumer Tests | Own all `sdk/python/examples/**` except (d)'s manifests; `sdk/python/{README.md,mise.toml,tools/**}`, `sdk/python/tests/generation/**`, new starter consumer stories, `sdk/tests/fixtures/python/**`, Python shared-SDK story callers, and Python fixture launch/selectors in `crates/dokime-runtime/tests/support/` plus the Python-specific system stories named in Authoring Adoption Owners. Own product-generated `sdk/python/src/dokime_sdk/{__init__.py,_generated/**,_wire/**}` and generation of those paths, never generated hand edits. (a) hands off generator files before this starts. Delete old root main.py/plugin.py/suite.py/template.py layouts, old imports, shipped example locks and superseded export assertions. | Public-surface check at the three homes, strict stub check, generated-bindings check, typed installed-wheel consumers, `mise -C sdk/python run example-test`, reshaped SDK example tests and command-line created-starter story. Class reference flow runs against a real host and saves valid retained evidence. Regenerate all tracked artifacts and verify no old package is installed. |
| (f) Rust SDK and Starters | Own `sdk/rust/**` except (d)'s starter manifest; Rust shared SDK fixtures `sdk/tests/fixtures/rust/**` and Rust-specific shared story adapters. Add separate public operation/template contexts and bound client modules while retaining the protocol-only dependency boundary. Update the two-binary Cargo starter project, entries, catalogs, README and tests; delete their argument parsing, old single-template shortcut consumers, library-facing APIs and superseded client selectors. Runtime/CLI production consumers outside this list stay with the integrating owner. | `mise -C sdk/rust run format`, focused root build task for `dokime`/derive and consumer binaries, dependency-tree check and focused SDK integration tests. Generated bindings check and public surface, created Rust starter client/plugin and bundle stories on the native host. No full lane, bench profile or worker clippy. Upstream typed runtime-operation dependency remains visible. |
| (g) C# SDK and Starters | Own `sdk/csharp/**` except (d)'s manifests; `sdk/tests/fixtures/csharp/**` and C# shared story adapters. Replace sealed constructor authoring with classes/attributes, add contexts/handles/configuration, split shared author library/plugin/client projects and update locks, README, tests and generation inputs/output. Delete old argument parsing, single-template shortcut consumers, public library namespaces and replaced API assertions. | C# format/check, focused Debug build via the SDK task with `--project`, focused authoring/client tests, generation/client checks and installed NuGet consumer. Native created starter flow, shared capture contract, bundle/dev stories. Do not use the task's existing Release default for worker proof. |
| (h) C++ SDK and Starters | Own `sdk/cpp/**` except (d)'s manifests; `sdk/tests/fixtures/cpp/**` and C++ shared story adapters. Add classes/contexts/handles/configuration and second CMake binary, migrate examples/README/tests/installed package headers. Regenerate tracked binding output through generation tasks. Delete argument parsing, old single-template shortcut consumers, public library aliases/escape hatches and replaced API assertions. | C++ format/check, focused Debug SDK and consumer build/tests via SDK tasks, generated header/binding check, installed CMake consumer. Native created starter plugin/client, shared capture and command-line bundle/dev stories; Windows and Linux proof are independent. |

Paths in braces name files, not new generic modules required merely for symmetry.
Use the existing cohesive file when it already owns that concern. If a named
production file differs in the implementation checkout, establish the exact
equivalent before assigning it; do not create a parallel implementation. Shared
SDK driver files (for example the common `sdk/tests` runner) stay with the
integrating owner; each language owns only its fixture subtree/adapter, avoiding
simultaneous edits to the runner. The same rule applies to root mise tasks and
the shared generator: (a)'s mechanical configuration update finishes first;
later SDK owners change only their own generator inputs/output, with a follow-up
from the generator owner if its capability must change.

All eight items remove superseded code and consumers in their own delivery;
none leaves aliases or an installable old namespace. Keep tests for preserved
behavior; delete tests only when the covered behavior is explicitly superseded,
with that decision named. SdkError projection must preserve all failure causes,
classification and actions. No tests assert authored prose, timing, sampled
resources or one configuration constant. The public-surface guard becomes an
AST/import inventory of the three intentional homes rather than disappearing
when `__all__` is removed. It explicitly permits the two distinct Context names,
refuses a root Context and checks internal generation never becomes a fourth
author barrel. Strict stubtest covers authored public classes/functions as well
as generated resource bindings. Generation checks retain descriptor shapes,
fingerprints, caller filters and Watch metadata.

Every implementation owner runs formatting, an owned-language check without
clippy, focused compilation and changed-behavior tests through existing mise
tasks. `check:rust` currently includes lint, so workers use the focused build task
and formatting/dependency checks instead; the orchestrator owns lint. Final
installed starter/real-host and cross-language proofs happen after all required
source dependencies land. Hosted CI/Nightly owns full validation after authorized
delivery. A specification-only edit runs OpenSpec strict validation and the
explicitly requested `check:repository`, with no builds or implementation tests.

### Decisions Still Requiring the Owner

These are not permission to implement competing designs. Everything independent
of them can proceed from the signatures and ownership above.

| ID | Unsettled Contract / Recommended Smallest Resolution |
|---|---|
| S1 | Serial admission has no current policy field or agreed scope (per operation, plugin or device resource). Extend the shared operation policy with per-operation, per-activation serial admission, preserving its timeout/cancellation accounting; do not add a second product lock engine. The owner must choose the scope before implementation. |
| S3 | SettingGroup has no metadata contract in current flat settings. Recommend product-only group metadata over unchanged saved keys, exposed by the product settings schema. The owner must decide its stored/catalog representation; do not invent nested setting names or a parallel settings registry. |

The settled contracts are:

- S2 is settled by the earlier owner decision on `requires` (Option A): authors
  never declare requirements and the SDK declares every operation the plugin can
  reference. No new class attribute is added.
- S4: the plugin class uses `initialize` and `finalize`; the SDK encodes these as
  the well-known lifecycle operations rather than stop callbacks. Templates use
  `initialize` and `cleanup`. This owner decision supersedes the earlier
  start/stop and setup/cleanup spellings without aliases. The library owns lifecycle
  invocation and connection scopes; missing carrier capabilities remain dependencies,
  not permission for an SDK-local lifecycle dispatcher.
- S1 and S3 are not needed by the starter and stay open. Nothing in items
  (a)-(h) may implement serial admission or settings groups until the owner
  chooses.

Host-owned Ready registration, product plugin administration/invocation wrappers,
plugin Notes/Create admission, Executions/Seal and retained cycle report/archive
materialization are implementation dependencies, not unanswered SDK naming
questions. Their owners must publish the exact request/response contracts before
bindings are generated; SDK workers must not create lookalike protos or mark a
mock-only client proof as real-host proof. Ordered pairing attempt read policy
remains with `remote-access-simplification`; this section adds no retry budget.

### Overlaps and Documentation Impact

| Both Locations | Current Callers and Responsibility | Recommendation for the Owner |
|---|---|---|
| `sdk/csharp/src/Dokime/Client.cs:ArtifactAsync` and `FileTransfers.cs:UploadAsync` | Bound cycle/execution artifacts use the client loop; template evidence and file inputs use the transfer helper. Both create, append, publish, verify and abandon uploads. | Keep the transfer owner and delete the client loop after owner approval; retain child mutation identities and abandon only unpublished uploads. |
| `sdk/csharp/src/Dokime/Registration.cs:PluginEndpoint` and `sdk/csharp/tests/Stories/Support/PluginServer.cs` | Registration and carrier stories both use the library addressable listener. The product wrapper retains idempotent close, while the test helper owns its channel. | Console-capture ownership was removed with the library listener adoption. |
| `crates/dokime-client-internal/src/bin/content_fixture.rs:Wire` and `sdk/rust/src/message.rs:Wire` | The unpublished protocol fixture and SDK independently adapt reflected prost messages to the carrier Message trait. The orphan rule prevents direct implementations for protocol types. | Prefer a library-owned prost adapter, then delete the fixture adapter and SDK adapter. Never restore a fixture dependency on the SDK to share this code. |
| `sdk/{rust,csharp,cpp}/tools/test_created.py:Host/HostFactory` and `sdk/python/tests/system/test_starter.py:Host/HostFactory` | Each created-project story declares the same structural host-support contract for runpy loading. | Give a shared test-support module the contract, then delete the per-story protocol copies; keep test support outside the public SDK. |
| `sdk/python/tests/support/host.py:fixture` and `sdk/tests/test_authoring.py:commands` | Prepared host/CLI/Rust consumers resolve profile-aware paths through the shared lookup; shared capture independently hard-codes the C# development and C++ test fixture paths. | Extend the shared prepared-fixture inventory, then delete the C#/C++ path construction from the capture story after the owner selects its profile policy. |
| `crates/dokime-gui/src/report/`, `crates/dokime-operations/src/host/reports/create.rs:193` and `crates/dokime-runtime/src/services/state.rs:151` | Current Reports/Create collects a protobuf capture, inserts its Base64 bytes into the generated React template and retains that HTML. Runtime supplies the template; `src/documents.rs` resolves documents and stored report metadata, with no separate HTML renderer. The previous overlap claim and `docs/guides/gui.md:5` are stale. | Keep this single pipeline; the owner must reconcile the stale documentation and review any proposed renderer change against the current retained-report contract before deleting code. |
| `crates/dokime-gui/benches/system/fixture.rs:Fixture::start` and `crates/dokime-gui/src-tauri/src/connection/host.rs:Session::install_plugin` | The benchmark admits a remote fixture and desktop installation admits a managed plugin; both independently issue Create, Watch and Get until Ready. Their callers are benchmark fixture startup and desktop installation respectively. | Keep readiness admission in Session and delete the benchmark loop once that owner supports remote placements; the owner must choose before either mechanism is removed. |
| `crates/dokime-runtime/src/plugin.rs:PluginProcess` and the library `crates/harmos-runtime/src/plugin/grpc/mod.rs` | Development CLI, host fixtures and product stories use the product launcher for project scripts and externally owned children; registry-managed placements use the library launcher. Both implement native child startup, Stop and bounded exit handling, with distinct process owners. | Consider a library-owned reusable spawned-process lifecycle and delete the product mechanics after it supports project launch inputs; retain the external launcher's ownership and never transfer it through launch credentials. No consolidation is authorized here. |
| `crates/dokime-runtime/src/host.rs:remove_plugin` and `detach_plugin` | Host facade consumers can invoke either method; detach forwards directly to remove with identical durable deletion behavior. | Prefer remove and delete the redundant detach entry point after the owner confirms its public API contract; this adoption does not change it. |
| `proto/dokime/host/v1/plugins.proto:PluginState` and `proto/harmos/host/v1/plugins.proto:PluginState` | Product RPC projections (`crates/dokime-operations/src/plugin_policy.rs:333`) add Offline while native `Host::entry_state` exposes the library family without Offline. The launch proof uses the native return type; desktop/client projections retain the product type. | Prefer library-owned Offline classification and then delete the product enum after owner approval; preserve current Offline behavior and add no conversion or alias for the launch test. |
| `sdk/rust/src/stream.rs:Stream` and library `sdk/rust/src/stream.rs:Stream` | Product templates, observers and push producers use the context-adapting product wrapper; native host collections use the library stream directly. Both own identifiers, descriptions and row types. | Prefer one library stream after author-context composition is available, then delete the product wrapper; collection adoption does not authorize removing template and producer support. |
| `sdk/cpp/src/plugin.cpp:Plugin::serve`, `sdk/tests/fixtures/cpp/thermal/main.cpp:main` and the library `sdk/cpp/src/server.cpp:Listener::close` | Product serving and the thermal fixture explicitly shut down the process default EventEngine; the library listener settles its dedicated serving engine. These are parallel teardown owners for different engine instances, with a repeated process-global release in the fixture. | Ask the library owner to own the full transport teardown, then delete the product and fixture process-global shutdown calls; do not remove either resource release speculatively. |
| `crates/dokime-runtime/tests/system/client_edits_settings.rs:SettingsPlugin` and `sdk/rust/src/plugin/mod.rs:into_harmos` | The settings story builds its catalog with the SDK but implements the Instance protocol itself, including lifecycle acknowledgements; authored plugins use the SDK's library-owned listener. | Use an unpublished library-owned settings protocol fixture, then delete the story's manual carrier and SDK catalog dependency after the owner agrees. |
| `mise.toml:lint:rust` and `tools/python/src/dokime_tools/build/rust/tests.py:build` | Rust lint selects all default-feature targets; test archive production additionally selects the GUI testing feature. The latter therefore found a fixture compilation failure after the lint task passed. | Share the required test-feature inventory and delete the independent hard-coded selection after the task owner agrees; retain both lint and archive proof. |
| `tools/python/src/dokime_tools/build/cli_fixtures.py:build` and `build/desktop_fixtures.py:build` | Every language's CLI fixture producer rebuilds the development CLI; desktop fixture production independently builds the selected-profile CLI. Concurrent consumers can hold the same executable open while another producer replaces it. | Build each required CLI profile once before fixture consumers, then delete their unconditional rebuild calls; preserve development versus selected-profile fixture policy. |
| `tools/python/src/dokime_tools/build/cli_fixtures.py:build` and `sdk/rust/tools/test_created.py:CreatedProjectStory` | Fixture production and the created-project story separately create and compile the same Rust starter into the checkout target. Both own generated-project dependency resolution and compilation setup. | Give generated-project compilation one owner, then delete the duplicated orchestration after the owner selects the required independent fresh-build proof; preserve user-owned locks and each story's build assertions. |
| `sdk/cpp/tools/sdk.py` and the library `tools/python/src/harmos_tools/build/cpp.py` | Product and library SDK tasks independently bootstrap the pinned vcpkg checkout and orchestrate native compiler, CMake, format, test and install operations. | Give the library a configurable native build owner, then delete the duplicated product bootstrap/build orchestration after the owner agrees; keep product task and artifact policy product-owned. |
| `sdk/cpp/cmake/Protocol.cmake:dokime_protocol`, `sdk/cpp/tests/CMakeLists.txt` and the library `sdk/cpp/cmake/Protocol.cmake:harmos_protocol` | Product services, the thermal fixture and library services independently configure native protobuf generation and its compiler dependencies. | Share one configurable generation recipe, then delete the product helper and separate fixture recipe after the owner agrees; preserve their distinct protocol inventories and install namespaces. |
| `tools/python/src/dokime_tools/test/archive.py:install_wheels` and `sdk/python/tools/installed.py:main` | Archived test lanes and bindings install the selected-profile wheel through the former; SDK preparation installs a hard-coded development wheel and its example distributions through the latter. Both select and install the built SDK wheel. | Give wheel installation one profile-aware owner, then delete the hard-coded development-wheel selection and installation block after the owner agrees; retain explicit example installation and the caller's selected artifact. |

The pinned library's `sdk/cpp/CMakeLists.txt` also adds its sample executables to
consumer builds. Make those examples opt-in or top-level-only upstream; product
starters should not grow target-name exclusions for library samples.

The C++ starter's normal build stages Release runtime inputs for bundling. Debug
executables stay in the build tree: their dependency binaries can exceed the
existing per-file bundle bound. This preserves the bundle admission limit and
the created-project build deadline; it does not change SDK Debug test builds.
The C++ CLI fixture producer reads Debug executables and their companions from
the consumer build tree rather than the starter's bundle assets. Rust fixture
production selects the Debug compiler environment independently of its fixture
output profile, matching the generated project's normal build task.

### Prompt Deadline and Stream Lag Proof

`crates/dokime-runtime/tests/system/python_client_runs_thermal_suite_over_grpc.rs`
advances the shipped starter's execution clock by two hours while its prompt is
pending. The prompt remains pending across operation deadlines, but the same
clock drives the continuous observer in
`sdk/python/examples/starter/src/starter_project/plugin/suites/capture/confirmation.py`.
That jump correctly violates the stream window lag policy and interrupts the
execution before its final Passed assertion. This is REWORK: the owner must
choose clock advancement that waits for every due stream window, or separate
prompt-only fixture coverage. Neither permits relaxing operation deadlines,
stream lag limits or the outcome assertion.

The owner writes `docs/`; no documentation file changes belong to SDK workers.
When implementation lands, these current statements need reconciliation:

| File / Line | What Becomes True |
|---|---|
| `docs/project/architecture.md:15`, `:37`, `:1133`, `:1217` | Product-only contexts/handles and class composition replace exposed carrier types and old entry/lifecycle descriptions; retained evidence follows seal-finality. |
| `docs/index.md:144` | SDK/CLI workflows use bound cycle/case/execution handles, product plugin operations and explicit seal finality. |
| `docs/guides/plugins.md:21` | Python execution state belongs to the TestTemplate instance, with plugin state separately reachable through ctx.plugin. |
| `docs/sdks/rust.md:7`, `:37`, `:52`, `:96` | Separate operation/template contexts, one starter with two binaries, operation-encoded lifecycle hooks and bound client flow replace the old snippets. |
| `docs/sdks/csharp.md:65`, `:81`, `:115`, `:139` | Author bases/attributes, one shared-library/plugin/client starter and Task/IAsyncEnumerable bound handles use product-only namespaces and the shared configuration grammar. |
| `docs/sdks/cpp.md:5`, `:13`, `:26`, `:85`, `:90` | The library pin is the common maintenance revision; author classes, protected lifecycle operations and blocking bound handles use one argument-free starter whose build stages Release bundle inputs. |
| `docs/guides/execution-host.md:308` | The exact three-table manifest, dynamic metadata and implicit assets replace top-level scripts/assets. |
| `docs/project/architecture.md:449` | External inverses belong to durable plugin state; the product Effects service and per-call confirmation tickets are removed, with durable state access and bounded recovery now supplied by the library. |
| `docs/guides/execution-host.md:920` | Effects/Create, Effects/Get and Effects/Set no longer exist in the protocol or SDKs. |
| `docs/sdks/cpp.md:152` | The product effects surface is removed; the host plugin journal now uses revisioned library-backed plugin persistence. |
| `docs/sdks/rust.md:100` | `operations::Context::environment()` returns the same backend process ID and in-step flag as the template context. |
| `docs/guides/gui.md:5` | Reports/Create embeds a protobuf capture into the generated React report template; there is no separate native HTML renderer in `dokime-runtime/src/documents.rs`. |

The root README and SDK package READMEs describe the product imports, bound
connection/client flow, protected lifecycle operations and single starter roots.
The documentation changes above remain with the owner; workers do not edit
`docs/` or unrelated prose.

## Host Telemetry Plugin Restoration (Decided by the Owner, 2026-11-20)

`plugins/sidecars/host` (package `plugin-host`, `publish = false`) restores the host
telemetry plugin last present at `367aae91d^` on the current SDK, shipped as a default
plugin beside OBC. The decisions:

- **Full parity.** System, network, CPU, Wi-Fi and Bluetooth telemetry, radio and
  adapter controls, adapter inventory, power plan operations and CPU load control.
  Fields, units and calculations of the old rows are preserved in
  `src/domain/telemetry.rs`; old boundary types (`TelemetryContract`, `TelemetryRow`,
  `StreamSpec`, `Listenable`, section macros, injected configuration) are replaced by
  SDK rows, streams, observers, operations and flat setting declarations.
- **Demand-driven telemetry (supersedes plugin-wide retention).** Host streams are
  demand-driven by design: nothing is collected, cached or published without a
  listener, and no native resource is retained outside collection demand. The calibrated CPU
  scalar may remain cached. The earlier
  decision to retain plugin-wide telemetry from plugin connect to disconnect is
  withdrawn.
- **Stream shape (owner-locked).** A stream's row holds fields that always exist
  together; when they do not exist, the stream does not publish. No stream row has an
  optional, repeated or map field, and no column is nullable. Fields are grouped by
  availability (connected, battery present, calibrated, sensor present). A
  collection's outcome is its own stream, `<collection>_probe` with `outcome` and
  `error`, published every tick (`error` is empty on success); the measured value is
  published only when it was read. A list is one row per element, every row of one
  collection sharing its timestamp.
  An echo without a reply publishes `timeout` in `wifi_latency_probe` and 100%
  loss in `wifi_loss`; it publishes no `wifi_latency` value. A probe that cannot
  run publishes `error` and its diagnostic instead of measured values.
- **Collections.** One collection per native measurement (system, network, Wi-Fi
  link, Wi-Fi scan, latency echo, Bluetooth inventory) feeds several declared streams
  from one measurement with one timestamp, may publish zero or more rows to each,
  ticks at an interval setting, and runs only while one of its streams has a
  listener; its start and stop hooks acquire and release native handles, so no handle
  or cached reading outlives the last listener. The Rust SDK shape, proposed for the
  owner's review before the other SDKs copy it:

  ```rust
  let stream = harmos::plugin::Stream::<WifiAdapter>::new(WifiAdapter::STREAM);
  let collection = Collection::operation(async |ctx, rows: CollectionOutput| {
      let reading = read(ctx).await?;
      rows.publish(&stream, reading.adapter());
      Ok(())
  })
  .stream(harmos::plugin::Stream::<WifiAdapter>::new(WifiAdapter::STREAM))
  .interval_setting("wifi_link_interval")
  .on_start(acquire)
  .on_stop(release);
  let plugin = plugin.collection(collection);
  ```

  The library collection now owns demand and granted windows. The product-side
  `Collection`, feed/listener state and sleep-loop producer are removed. Rust
  collections use the library `Collection` and `CollectionOutput` directly;
  `Plugin::collection` registers that value without scheduling it. A collection
  callback binds its invocation and captured author state through
  `operations::Context::from_collection`; it never retains a prior invocation.
  The in-process template harness refuses collection sampling and directs callers
  to a product host, since it cannot drive library-owned collection windows.
  System COM and network watching acquire through `on_start`; Wi-Fi link counters
  acquire there too. Stop hooks discard resources and cached measurements.

- **Effect journal.** Plugin state, persisted by the host through the library's state
  management; no product effects store and no product `Effects` operations.
  `plugins/sidecars/host/src/journal.rs` implements `controls::Journal` over one opaque,
  revisioned plugin state value (`journal::Store`: load, save with expected revision):
  `record(target, inverse)` before a write (an outstanding target keeps its earlier
  inverse and returns `false`), `clear(target)` after a restore, `outstanding()` in
  recording order. The plugin's `initialize` hook runs `controls::State::restore`,
  newest first, before any control runs; a failed inverse stays outstanding and fails
  `initialize`, so the host withholds the plugin with that failure on its entry. A host
  that cannot persist the value fails its state operation before any control writes. Inverses
  are absolute control states, so a restore repeated after a crash between record,
  write, restore and clear is idempotent. The `Context` store uses the library
  StateHandle and propagates persistence failures before a control writes.
- **Start snapshot.** The old automatic adapter capture at run start becomes an
  explicit typed snapshot operation that a consuming template calls from its
  `initialize` and records itself; the host adds no run-start contribution.
- **Delivery.** Four packages with disjoint ownership (table below). Package 1 lands
  the contracts, host prerequisites, settings, domain model, module tree and Build
  integration; packages 2-4 add only inside their own files.

### Host Prerequisite Status

| Prerequisite | Status |
|---|---|
| Duration settings | `dokime::operations::Context::duration_setting` and `setting` (`sdk/rust/src/operations.rs`), backed by `crate::context::Context::duration_setting` (`sdk/rust/src/context.rs`); the name is the one `Observer::interval_setting` takes. |
| Collection host calls | Library `sdk/rust/src/context.rs:230` refuses the old unscoped producer because it has neither a child transport nor an output carrier. Native collection invocations carry host calls; interval settings are resolved by the host. The product no longer calls Settings/List from a producer. |
| Plugin-wide retention | Withdrawn: host streams are demand-driven; historical facts follow. Blocked. Storage keys every sample, stream and telemetry file on an execution (`streams`, telemetry catalog), `crates/dokime-runtime/src/streams.rs` drops unscoped publications, the runtime observes no plugin connect/disconnect, and `dokime.model.v1.EvidenceScope` has no plugin selector, so plot requests (`PlotRequest.scope`) cannot address a plugin-scoped stream (Samples List/Watch could through `sources`). The contract needs `EvidenceScope` to name the plugin entry; that regenerates every SDK binding and needs the owner's go. |
| Effect journal | `plugins/sidecars/host/src/journal.rs` persists the Effects value through the library StateHandle. The product composes plugin state with its durable storage/recovery owner. Revision conflicts re-read within the current invocation; no local files or product state manager are added. |

### Effect Journal Library Boundary

The library owns one opaque revisioned state value per plugin entry, accessible
from initialization and admitted collection/operation invocations. The product
composes StateGet/StatePut with its storage and recovery backend; plugin inverses
use this facility rather than application transactions, product Effects calls or
plugin-local files. Recovery compaction retains the latest published snapshot
and the uncovered suffix in `crates/dokime-storage/src/backend/recovery.rs`.

A durable record acknowledged before mutation makes a crash before that mutation
restore the already-current absolute state. A crash after mutation, or after
restoration but before clear, replays the same absolute inverse idempotently.
Clear is durable after restoration. Failed restoration retains its entry and
fails initialization. Fake-store tests prove ordering and conflict preservation;
actual machine mutations and disk-process crash restoration are not this proof.

The platform implementations set absolute radio, adapter and power-plan states,
including when restoring an already-current state. There is no separate
outstanding-effect operator read operation: restoration failure surfaces through
the plugin entry. An operator read capability remains a separate owner decision.

The owner's plugin-state decision supersedes the product-owned confirmation
tickets in `operation-grammar/specs/operation-contracts/spec.md:590`. That separate
change and `docs/project/architecture.md:449` require owner reconciliation; this delivery
does not restore the old ticket contract.

### Host Plugin Ownership

Package 1 owns every file not listed below, including `Cargo.toml` of the plugin
(dependency lines are added by the area that first uses them: `sysinfo` and
`starship-battery` by package 2, `zbus` by the first of packages 3 and 4; the
workspace already declares them), `src/lib.rs`, `src/main.rs`, `src/settings.rs`,
`src/domain/**`, every `mod.rs`, `src/platform/windows/apartment.rs` and
`tests/integration/{main,settings}.rs`. Packages never edit another package's files.

| Package | Owns | Consumes |
|---|---|---|
| 2 System, Network, CPU | `src/streams/{system,network,cpu}.rs`, `src/collectors/{system,network,cpu}.rs` (each may become a directory), `src/platform/windows/{connectivity,power}.rs`, `src/platform/unix/{connectivity,power}.rs`, `tests/integration/{system,network,cpu}.rs`, `benches/**` and the bench target | `domain::telemetry::{System, Network}` and `Column`, `settings::{duration_value, integer, number, text, selected, CpuLoad}`, `Host::{system, network, cpu}`, `platform::windows::apartment::retain`; exposes the power-read interface package 4 uses |
| 3 Wi-Fi, Bluetooth | `src/streams/{wifi,bluetooth}.rs`, `src/collectors/{wifi,bluetooth}.rs`, `src/platform/windows/{wifi,echo,bluetooth,devices}.rs`, `src/platform/unix/{wifi,echo,bluetooth}.rs`, `tests/integration/{wifi,bluetooth}.rs` | `domain::telemetry::{WifiLink, WifiScan, WifiLatency, Bluetooth}`, settings readers, `Host::{wifi, bluetooth}`, `apartment::retain` |
| 4 Controls, Adapters, Power | `src/controls/**`, `src/adapters/**`, `src/power/**`, `src/platform/{windows,unix}/{controls,adapters}.rs`, `tests/integration/{controls,adapters,power}.rs` | `domain::controls`, settings readers, `Host::{controls, adapters, power}`, package 2's power reads, and the effect journal once it lands |

Each area registers through its `catalog(plugin: Plugin) -> Result<Plugin>`, which
`src/lib.rs` already calls, and keeps plugin-wide state in its field of `Host`
(constructed by `Default`; an area may replace the derive with its own `Default`
implementation and release resources in `Drop`). Documents for host streams belong
to the area that declares the stream and need plugin-wide retention first.

### Host Integration Gate

The pinned library supplies durable plugin state. `journal::Store for Context`
reads one typed Effects value through `ctx.state().get`, retains that handle's
observed revision, and performs revisioned Put before the native write. Store
futures follow the library's invocation-local future contract. A
`state_revision_conflict` re-reads the value and reapplies record or clear inside
the same operation's policy; other failures propagate. The unavailable-state
fallback and refusal gate are removed. Fake stores prove concurrency and inverse
preservation; native controls are not executed by these tests.

System collection start acquires its COM usage lease; stop drops the native
inventory before releasing the lease. A stopped collection cannot lazily recreate
its inventory. Standalone radio and connectivity probes own scoped leases. The
CPU `OnceCell<f64>` retains only the calibrated machine value: calibration's
scoped threads are all joined before it returns, and no worker or handle is
stored in the cell. This scalar retention is permitted outside listener demand.

The shipping build already includes `plugin-host` beside OBC. The release plugin
inventory in `tools/project.toml` declares both native targets; installer manifests
retain those kit defaults unless explicitly excluded. Distribution export includes
their bundle bytes in the checked file inventory, so portable archives retain the
same default plugins as local installer staging. No installer execution or machine
installation is needed to prove this staging behavior.

## Registered (Unmanaged) Plugin Lifecycle (Locked by the Owner, 2026-11-20)

### Managed Launch Handshake

The library pin is `324ad01b0892b1ca6a3bce4bb9525b22974f606b`.
Managed startup uses the library's `Launches` reservation and its
`HARMOS_HOST_ENDPOINT` and `HARMOS_LAUNCH_TOKEN` environment. The plugin serves
its listener and calls `harmos.plugin.v1.Launches/Announce` once. Standard output
is ordinary output and never a readiness protocol. A partial credential pair
is an SDK startup error. Detached development launches use the same handshake.

The source inventory for the product cutover is the workspace library pin and
generated consumers, `crates/dokime-runtime/src/plugin.rs`, its Python fixture
support, the CLI first-hour fixture and the desktop e2e fixture. The native
`dokime-host --launch-plugin` fixture owns the e2e process through
`PluginProcess`; JavaScript receives that fixture owner's endpoint instead of
parsing plugin output. SDK entries select announcement, self-registration or standalone serving from
the environment; retired address configuration and output parsers are removed. The library process owner is private; replacing the development
process owner with it remains an owner decision.

### Registry Process Ownership

Launch credentials authenticate readiness and do not transfer process ownership.
The library registry must send Instance.Stop only when its activation owns a
spawned process. The pinned `crates/harmos-runtime/src/plugin/grpc/mod.rs` gates
Stop on that ownership. Removing a remote entry settles its host-side activity
and closes the registry connection without stopping its externally owned endpoint.
Managed process teardown keeps
its Stop, exit, kill and reap policies unchanged. This removes the need for a
second product lifecycle marker or for mode selection based on credentials.

The product cannot enforce this private registry boundary. It retains ordinary
launch reservations and self-registration, and does not bypass registry teardown.
The library checkout is read-only for this delivery.

### Registration and Presentation

An unmanaged plugin is started by its author with `DOKIME_REMOTE` configured and
registers itself through `dokime.host.v1.Plugins/Create` with a remote placement.
The host reads no bundle, manifest or project file for it: identity, label,
operations and catalogs come from the plugin's own description.

- Wire: `PluginsCreateRequest.sdk_version` (optional) carries the product SDK
  version the plugin was built against. The product Create policy refuses a
  remote registration whose version's caret range does not contain the host
  version with `plugin_incompatible`, naming both versions. Managed bundles keep
  their manifest check.
- Wire: `PluginState.PLUGIN_STATE_OFFLINE` presents a remote entry the library
  reports unreachable (`Unavailable` with `plugin_unavailable`) without a
  failure. Entries are durable and only an explicit Delete removes them; the
  SDK no longer deletes its entry when the plugin exits.
- Re-registration reuses the entry matched by name (the plugin identifier). The
  library first answers replays and refuses conflicting request identifiers;
  an `entry_exists` refusal with the same address enables or restarts the
  entry, and a new address deletes and recreates it under the same name.
- Wire: `Suite.unavailable` marks the last reported suites of an Offline plugin.
  They stay listed; planning and materialization refuse them with an action.
- Desktop and `dokime plugin list` show each entry's placement (Managed or
  Unmanaged) and status (Ready, Starting, Offline, Failed, Disabled, Stopping,
  Stopped); the desktop list filters by both.

Library gaps at the current pin: a remote placement cannot change its address,
so a new address drops host-saved plugin settings through Delete and Create; a
remote entry whose first attach fails (for example after a host restart while
the plugin is down) is not retried until the plugin registers again; reconnect
failures that are not unreachability (changed identity, settings that no longer
fit) also carry `plugin_unavailable` and present as Offline; operations of a
disconnected plugin leave the library catalog, so only suites and templates are
retained as unavailable.

## Desktop Tables and Native Settings

Every desktop table uses `src/components/data-table/DataTable.tsx` under
`crates/dokime-gui/`. It owns the frame, title and one-line description,
right-aligned filter/action band, column controls, fixed-height empty/loading
row, full-width failure slot and always-visible pagination footer. Controls
follow filter, primary action, secondary actions and overflow order. Only
declared column filters appear; numbers use ungrouped decimal text, numeric
columns align right, units use square brackets and Actions is last on the right.
Keyboard order follows the markup: filter/actions, headers, rows, footer.
Callers keep their existing failure renderer and supply it to the shared slot.

The inventory contains 13 table presentations (paths below are relative to
`crates/dokime-gui/src/`). Apart from the grid plot, all already used DataTable;
differences came from caller wrappers and header/state placement. Their shared
density, sort controls, empty state and footer remain component-owned.

| Table | Source | Presentation Decision |
| --- | --- | --- |
| Cycles | `features/cycles-home/CyclesHome.tsx:179` | Keep each group title inside its frame; put the existing failure in the first group's body. |
| History | `features/explore/HistoryLibrary.tsx:166` | Add the missing title/description; retain the server-owned row filter and selection. |
| History Executions | `features/explore/HistoryDetailSurface.tsx:139` | Add title/description, move loading/failure into the body and delete the superseded external empty paragraph. |
| History Comparison | `features/explore/HistoryComparisonView.tsx:81` | Keep the explanation in the description and add the missing title. |
| Storage Sources | `features/settings/StorageSourcesCard.tsx:156` | Remove the SettingsCard wrapper and external explanations; right-align numeric priority. |
| Shared Secrets | `features/settings/sections/SecretsSection.tsx:175` | Move the explanation into the description and keep Add, Refresh and transfer overflow after the filter. |
| Plugins | `features/settings/plugins/PluginTable.tsx:133` | Add title/description and shared loading/failure slots; the disabled update action explains the missing host target through its tooltip and accessible description. |
| Saved Backends | `features/settings/BackendsTable.tsx:135` | Remove the separate card and external loading/failure band; keep pairing import in the action band. |
| Suite Templates | `features/library/LibrarySurface.tsx:156` | Name each table in its frame; the empty/loading/failure case also renders through DataTable. |
| Document Values | `components/document/DocumentSurfaceView.tsx:255` | Add title/description; infer numeric alignment from recorded values. |
| Metric Statistics | `components/StatsSummaryTable.tsx:153` | Add title/description; preserve exact right-aligned values and each metric's unit-bearing label. |
| Trace Statistics | `components/StatsSummaryTable.tsx:168` | Add title/description; retain row activation for trace visibility and exact right-aligned values. |
| Grid Plot | `plots/catalog/GridChartPlot.tsx:105` | Replace raw Table markup with DataTable while preserving validity, cell colors, highlights and independent playback; put units in numeric column headers. |

Native desktop stream settings, remembered parameter values and update selection
use one `settings.json` and `src-tauri/src/settings.rs` reader/writer. Missing keys
use defaults; unknown keys are ignored. Retired files and browser parameter values
are not migrated or deleted. Engine / Streams exposes only `interval_window_ms`
and `interval_lag_windows`. The capture-folder allow-list and its refusal stories
are removed; canonical-path validation, capture sizes and deadlines remain.
Host-owned preferences, pairing credentials and installed distribution metadata
retain their existing owners; they are not copies of native desktop settings.

### Open Presentation Ownership

These overlaps require an owner decision before either implementation is removed.
Paths below are relative to `crates/dokime-gui/`.

| Both Locations | Current Callers and Responsibility | Recommended Deletion |
| --- | --- | --- |
| `src/stores/ui-store.ts:218` and `src-tauri/src/settings.rs:156` | Shell controls persist sidebar/console geometry and active-cycle state in WebView localStorage; native Engine, parameter and updater callers persist their settings through Store. Desktop-local persistence therefore has two owners, although their saved keys differ. | Fold the desktop-only UI state into Store, then delete the Tauri localStorage writer; retain the UI state API and host-owned appearance preferences. |
| `src/components/StatsSummaryTable.tsx:136` and `:199` | StatsSummaryTable routes multiple metrics to DataTable but one metric to StatsSummaryGrid; plot/document callers therefore get two statistics presentations. | Delete the single-metric grid branch and component if the owner chooses one statistics presentation; preserve exact values and metric units. |
| `src/features/explore/ExploreSurface.tsx:124` and `src/components/data-table/DataTable.tsx:309` | History's Load More control appends bounded server pages; the shared footer pages the rows already loaded. History operators encounter two paging controls. | Let the shared footer own bounded server paging, then delete Load More; first settle continuation, row-count and page-indicator behavior rather than fetching all history or inventing a second paging contract. |
| `src/features/library/LibrarySurface.tsx:144` and `src/components/data-table/DataTable.tsx:204` | Library's catalog filter selects suites and templates; every suite table also has the shared component's local row filter. Library callers can therefore narrow the same templates through two inputs. | Keep the catalog filtering owner and delete the per-suite local filtering path after the owner chooses a persistent placement inside the shared presentation; preserve suite grouping and keyboard focus when a group stops matching. |

The compiler task also has overlapping feature inference in the repository-root
`tools/python/src/dokime_tools/test/focused.py:167` (`selections`) and `:55`
(`run`). The changed-path workflow calls both: selections builds required target
features, then run reads the same target metadata and appends those features
again. The workspace no-run command therefore repeats `dokime-gui/testing`.
Keep required-feature policy in run and delete the selection-side copy after the
tooling owner settles the shared handling of explicit test targets and `--tests`;
this worker does not change tooling or its compiler selectors.

The inherited updater in `crates/dokime-gui/src-tauri/src/app/updates/mod.rs:63`
also gates fields and methods directly. The `stkr-rust` module-only gating rule
requires a structural split of optional updater code/state while keeping saved
policy under its current single owner. Whether that refactor joins this delivery
is an open owner scope decision; the settings persistence change does not claim
that convention cleanup is complete.

### Launch Entry and Lifecycle Ownership

Every starter calls its SDK's single blocking entry. Launch credentials take
precedence over pairing and dotenv parsing, and the SDK delegates announcement
to the library. Otherwise a configured remote selects self-registration and an
absent remote selects standalone serving. Programmatic registration owns its listener; no listener environment variable
or output parser participates in startup.

A remote registry entry does not own the process serving it. The pinned library
gates Instance.Stop on its existing spawned-process ownership, including entry
removal, disable, initialization/reconnect failure and host shutdown. Detaching
an external listener ends its attached RPCs while the listener continues serving.
Registration across a host restart remains required, and its existing guard and
Ready assertions remain unchanged. Launch credentials authenticate announcement;
they never transfer process ownership.

The Python library's `sdk/python/src/harmos/_authoring.py:426` answers Stop without
closing its endpoint, including when that endpoint was started through managed
run. The library must keep addressable listeners independent while allowing its
managed run entry to finish on Stop. The product delegates announcement and does
not duplicate either transport policy.
Product launch reservations remain token-and-endpoint authentication only; no
second product lifecycle marker is introduced.

The launch source inventory comprises the four SDK entry implementations and
starters, SDK listener configuration and readiness fixtures, the product launcher
and registration stories, generated bindings after merging dev, and desktop
plugin-table integration. Required proof stays open until performed.

The host duration-setting path also makes a redundant read:
`plugins/sidecars/host/src/settings.rs:539` validates through `checked`, then
`sdk/rust/src/context.rs:390` fetches the same setting again for conversion.
Host collector and control timing readers call this path. A shared conversion
of the already-validated value would remove the second request and the change
between validation and use; its API owner remains an open streamlining decision.

### Host Collection Integration Contract

The host plugin shares one `Arc<Host>` between operation authoring and native
collection callbacks. Interval resolution and tick timestamps belong to the
library host. Sparse rows use typed member streams and the library output,
which retains publication validation until handler completion. Durable control
inverses use the library state operation family composed by the product host;
no plugin-local state or journal files exist. The launcher proof observes Ready,
two network collection rows, and Ready again before joining its owned child.

This requested native collection adoption supersedes the earlier product-only
collection author surface: callbacks, member streams and outputs use library
values. The product operation-context adapter binds the shared Host value for
existing collectors. Other authoring surfaces and other language SDKs retain
their current owners; this delivery adds no second collection implementation.

Scoped execution ingestion distinguishes native collection members from ordinary
interval observers: collection rows retain their per-tick source timestamp and
are persisted individually, while ordinary observer column batches keep their
existing expansion path in `crates/dokime-runtime/src/streams.rs`.

Collection lifecycle hooks in the pinned library currently clear their child output
(`sdk/rust/src/server.rs:604`) and cannot call host operations. The host plugin's
hooks therefore acquire/release native resources only. Network link baselines
are read during the configured collection invocation and seed only unknown
links, preserving queued native events. Supporting host calls in library hooks
would require retaining a Samples child output in `transition` and pumping child
requests/replies while the hook runs, instead of treating every input as closure.
The plugin requires no such call, settings cache, fixed interval or alternate
transport. Other settings remain read on their current collection/operation call.

Network resource acquisition completes synchronously inside the lifecycle hook,
like the system COM lease and Wi-Fi link counter acquisition. No detached
blocking job can install native resources after cancelled demand has stopped.
