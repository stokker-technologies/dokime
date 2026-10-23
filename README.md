# Dokime

**Run repeatable engineering tests. Keep the evidence.**

Dokime turns hardware and software test procedures into operator workflows,
live measurements, and self-contained reports. Use it for bench testing,
device qualification, and manufacturing checks where a result needs more
context than pass or fail.

Define the procedure in Rust. Run it across operating conditions. Review what
happened, with the inputs, measurements, and decisions that explain the result.

[Quick Start](#quick-start) · [The Test Domain](#the-test-domain) ·
[Examples](#examples) · [Documentation](#documentation)

![Dokime running the simulated charger test, with execution progress and live voltage and current measurements](screenshots/cycle.png)

## Features

- **Test Procedures**

  Define reusable suites and templates with typed parameters, ordered steps,
  acceptance criteria, and explicit cleanup. Keep the procedure readable
  alongside the equipment operations it calls.

- **Parameter Sweeps**

  Expand one template across combinations of operating conditions. Three input
  voltages and three loads produce nine cases without duplicating the procedure.

- **Live Measurements**

  Capture typed measurements while a test runs. Compose time-series plots,
  distributions, and other views with units and reference limits tied to the
  case's parameters.

- **Acceptance Criteria**

  Evaluate captured samples and structured records. Keep the actions that
  perform a test separate from the criteria that judge its evidence.

- **Operator Interaction**

  Combine automation with confirmations, forms, and manual checkpoints.
  Retain operator responses with the execution they belong to, along with
  screenshots, camera photos, and audio recordings captured on the testbench.

- **Traceability**

  Capture specimen identity, resolved inputs, and execution context. Keep logs,
  structured records, and attached files alongside the measurements they explain,
  including files watched in a local or FTPS folder while the execution runs.

- **Self-Contained Reports**

  Present results, plots, and captured evidence in self-contained HTML reports.
  Review them without installing Dokime or connecting to a running server.

- **Plugins**

  Integrate equipment and contribute suites, operations, observers, and settings.
  Dokime builds on [Harmos](https://github.com/stokker-technologies/harmos)
  for typed contracts and managed plugin lifecycles.

- **Automation**

  Run through the desktop application or control execution through the CLI and
  programmatic interfaces. A desktop session can use its local backend or attach
  to an explicitly configured remote server.

## The Test Domain

Definitions describe what can be tested. Execution captures what happened when
concrete cases ran. A **suite** organizes related **templates**; each template
materializes **cases** with resolved parameter values. A **cycle** groups the
executions selected for a test campaign.

```mermaid
flowchart LR
    subgraph Definition["Definition"]
        Suite["Test Suite"]
        Template["Test Case Template"]
        Case["Test Case<br/>Resolved Parameters"]
        Procedure["Steps and Criteria"]
        Suite -->|"1 → 0..N templates"| Template
        Template -->|"1 → 0..N cases · expansion"| Case
        Template -->|"Declares"| Procedure
    end
    subgraph Execution["Execution"]
        Cycle["Test Cycle"]
        Run["Test Execution"]
        Snapshot["Definition and Input Snapshot"]
        Evidence["Measurements, Records,<br/>Step Attempts and Results"]
        Cycle -->|"1 → 0..N executions"| Run
        Run -->|"1 → 1 snapshot"| Snapshot
        Run -->|"Captures"| Evidence
    end
    Case -->|"1 → 0..N executions over time"| Run
```

Each execution identifies exactly one case and one cycle. Repeating a case
creates another execution; retrying a step is recorded as a step attempt within
that execution. A cycle can bring together cases from different templates and
suites—it is not a copy of one suite. A cycle is created explicitly, starts
empty and grows only as cases are appended to it; running an empty cycle is
refused.

For a charger campaign, one template might measure steady-state charging while
another checks overcurrent protection. Cartesian expansion turns the operating
points into concrete cases:

```text
Input Voltage:  207 V, 230 V, 253 V
Load Impedance: 80 Ω, 100 Ω, 120 Ω

3 Voltages × 3 Loads → 9 Cases Of The Same Procedure
```

A list can also be one parameter value. An ordered load sequence such as
`[100, 50, 10]` Ω can describe three steps within one case; it does not have to
mean three separate cases. Expansion is declared explicitly.

## Quick Start

Dokime is pre-1.0 and currently supports Windows and Linux. To launch from
source, install [mise](https://mise.jdx.dev) and use the pinned toolchain:

```sh
git clone https://github.com/stokker-technologies/dokime.git
cd dokime
mise trust
mise install --locked
mise run prepare
mise run dev
```

This starts the desktop application with its local backend. The
[OBC plugin](plugins/obc-plugin/) supplies a simulated charger; no physical charger
is required. The task builds the OBC bundle into `target/debug/plugins/` and
points `DOKIME_PLUGINS_DIR` at that directory, so the backend loads it for the
session without installing it. Create a cycle
with **New Cycle** (its title defaults to the current date and time), then add
OBC's charging templates to it from the library, choose operating points, and
inspect live measurements and results.

To install OBC into a regular application instead, build its bundle and add
it through the application's plugin settings, then restart:

```sh
cargo run -p cargo-dokime -- build --package obc-plugin --output target/debug/plugins/obc-plugin.dokime
```

An installed Dokime places `dokime`, `dokime-server` and `dokime-gui` side by
side, and the CLI launches the other two: it looks beside itself, then in
`DOKIME_HOME`, then in the platform's installation directories. Arguments after
`--` reach the launched program unchanged:

```sh
dokime serve -- --application-root ~/dokime-lab
dokime gui --endpoint http://127.0.0.1:50051
dokime plugin install target/debug/plugins/obc-plugin.dokime --backend lab
```

`dokime plugin install` stages the bundle on the backend, shows its identity,
digest and the environment variables its secrets read (names only), and records
consent for the next restart; `--yes` consents without the prompt. Plugin authors
use `cargo dokime new`, `build`, `inspect` and `dev`; `dev` rebuilds on every
change and runs the bundle through `dokime serve` and `dokime gui`.

## Browser Clients

The desktop never needs gRPC-Web: its WebView reaches the backend only through
the shell. A browser client is the one caller that does, so a backend serves it
only when asked, together with each exact page origin allowed to call:

```sh
dokime-server --grpc-web --allowed-origin https://frontend.example
```

Repeat `--allowed-origin` for additional origins. An origin contains only its
scheme and authority; paths, credentials and wildcards are refused, and an
origin without `--grpc-web` is an error. Remote listeners still require
explicit remote configuration, TLS and bearer credentials; browser-origin
permission does not grant operation access. A GUI built with
`DOKIME_TRANSPORT=grpc-web` is such a browser client.

## Plugin Operation Permissions

Remote plugin calls require an explicit permission on the existing server
`--auth-file` bearer grant. Product operations keep their declared role policy:

```json
{
  "grants": [{
    "subject": "operator",
    "role": "control",
    "token": "REPLACE_WITH_A_PRIVATE_TOKEN_OF_AT_LEAST_32_BYTES",
    "operations": [
      { "owner": "chamber", "id": "chamber.read", "grants": [] },
      {
        "owner": "chamber",
        "id": "chamber.stream",
        "grants": [{ "operation": "chamber.stream", "scope": "chamber.temperature" }]
      }
    ]
  }]
}
```

The owner and operation ID are exact; wildcards and duplicate permissions are
refused. Unlisted plugin operations remain denied, including trusted-local
connections. Listing an operation grants it; `grants` adds only the scopes a
request may resolve to, such as the channel, stream or source it names. Each grant
names the listed operation and one exact scope; an empty list admits only requests
that resolve to no scope. The same permission
filters live discovery and actual invocation. Rediscover after a provider changes;
metadata does not promise admission or bypass request-dependent grants.

## Backend Profiles

The CLI connects to an existing server. Save and select a backend with:

```sh
dokime backend add lab --endpoint https://dokime.example:8765 --expected-backend BACKEND_ID --ca /path/to/ca.pem --keyring
dokime backend use lab
dokime auth login --backend lab --token-file /path/to/token
dokime status --backend lab --json
dokime operation list --backend lab
dokime operation show dokime.runtime.health --backend lab --json
dokime auth logout --backend lab
```

`operation list` and `operation show` read live provider contracts, including
owner, generation, grants, gRPC availability and conditional access. Discovery
is bounded and does not promise admission of a later invocation. `show` retains
all visible providers of the selected operation.

`operation list` displays operation ID, provider owner and description. Select a
provider explicitly for metadata or a call:

```sh
dokime operation show OPERATION_ID --owner OWNER --backend lab --json
dokime operation call OPERATION_ID --owner OWNER --backend lab --input-file request.json --json
dokime operation call OPERATION_ID --owner OWNER --backend lab --input-file request.json --input-stream-file items.jsonl --json
```

Inputs use the discovered contract's protobuf JSON shape. Streaming input is one
item per line; output emits item and completion events. Ctrl+C closes the local
call. From Rust, `dokime_client::Client::call_operation` uses the same live
contract discovery and shared Harmos codec:

```rust
let client = Client::connect(Target::new(endpoint, None)?, Some(token), None).await?;
let stream = client
    .call_operation("obc-plugin.get_status", Some("obc-plugin"), b"{}", Vec::new())
    .await?;
while let Some(event) = stream.message().await? {
    println!("{event:?}");
}
stream.close().await;
```

Browse the registered suite catalog, stop work explicitly and re-run a case:

```sh
dokime catalog list --backend lab --json
dokime catalog show REGISTERED_SUITE_ID --backend lab --json
dokime execution stop EXECUTION_UUID --backend lab --json
dokime cycle stop CYCLE_UUID --backend lab --json
dokime cycle rerun CYCLE_UUID CASE_UUID --key rerun-1 --backend lab --json
```

Replace the identifiers with those returned by the backend. `catalog list
--json` emits one complete suite definition per line; a later read failure
returns a nonzero exit status after any earlier lines. `execution stop` stops
one execution while the rest of its cycle keeps running; `cycle stop` stops
every execution of the cycle. Both return the observed cursor and do not wait
for a sealed report. `cycle rerun` appends the case's next attempt as a new
execution, leaving earlier attempts and their evidence untouched, and admits it
once under the given key. Cancelling a local request wait leaves backend work running;
query the durable request again to observe settlement.

Selection uses explicit `--backend` or `--endpoint`, then `DOKIME_BACKEND`, then
that configuration's saved default. An explicit endpoint uses only credentials
supplied for that invocation. Profile edits affect future connections.
`backend list`, `show`, `use`, and `remove` operate offline; removal clears a
matching default and retains OS credentials. `auth login` negotiates backend
identity and the operations available to the caller with the supplied bearer, then saves it for
that endpoint and marks the selected profile for keyring lookup. It refuses a
profile whose target changed during login. `auth logout` removes the local
credential; it sends no revocation request. Use `--token-file` on individual
commands for headless connections when an OS store is unavailable.

`status` and `auth status` perform a bounded live check. JSON distinguishes
`ready`, `unreachable`, `authentication`, `permission_denied`, `incompatible`,
`not_ready`, `timed_out` and `credentials_unavailable`; failed checks return exit
code 1. `--timeout-seconds` bounds connection and credential-store waits. A
store operation already in progress can finish after its caller times out.
Profile settings and OS credentials are separate writes: a file publication
failure can leave the credential saved, and explicit logout removes it.

Configuration lives at `$XDG_CONFIG_HOME/dokime/backends.json` (falling back to
`$HOME/.config/dokime/backends.json`) on Linux, `$APPDATA/dokime/backends.json` on
Windows, or `$HOME/Library/Application Support/dokime/backends.json` on macOS.
`--profiles-file` overrides `DOKIME_PROFILES_FILE` and the standard location.

The version 1 JSON contract keeps secrets in a separate OS credential store:

```json
{
  "version": 1,
  "default": "lab",
  "profiles": [{
    "name": "lab",
    "endpoint": "https://dokime.example:8765",
    "expected_backend_id": "BACKEND_ID",
    "ca_certificate_pem": null,
    "credential": "keyring"
  }]
}
```

Names contain 1–64 ASCII letters, digits, dots, underscores or hyphens. Unknown
fields, duplicate names, unsupported versions, invalid endpoints and dangling
defaults are refused. Files are limited to 1 MiB and 128 profiles. CLI edits lock
and atomically replace the file; contention refuses the edit for retry. Tokens
stay outside JSON. Python reads explicitly named profiles with
`DokimeClient.from_profile("lab", token=token)`; it uses the same configuration
location and accepts an explicit `path` override.

Add the Python client with `uv add dokime-client` and import `DokimeClient` from
`dokime_client.client`. See the [Python Client](packages/dokime-client/README.md)
for local distribution installation and complete examples.

## Planning And Creating Cycles

Name a template by its readable ID, `<suite id>.<segment>`, as `dokime catalog
show` lists it:

```sh
dokime plan preview --template obc.charger.steady.measurement --backend lab --json
dokime cycle create --label "Campaign 1" --backend lab --json
dokime cycle append CYCLE_ID --template obc.charger.steady.measurement --backend lab --json
```

A cycle is always created explicitly and empty; `cycle append` adds the cases a
plan generates to an existing cycle and never creates one. An omitted
`--identifier` is derived by the backend.

Preview leaves registration unchanged. `plan register` materializes cases
without a cycle; append them by ID with `cycle append CYCLE_ID --case-id CASE_ID`. Registration returns case IDs; retain its
key and repeat the same request to recover registration after an uncertain
response. Repeated selections and `--case-id` arguments preserve order and
create distinct attempts. Omitted parameters and files use registered backend
defaults. `--selection-json` accepts a canonical `RegisteredSelection` and can
be repeated for parameter sweeps and file selections.

Each `--input-file` takes the operation's canonical JSON request and refuses
selection or metadata flag overrides. For example, `plan register` accepts:

```json
{
  "plan": {
    "selections": [{
      "suite_id": "SUITE_ID",
      "template_id": "TEMPLATE_ID",
      "parameters": [],
      "files": []
    }],
    "max_cases": 0,
    "case_ids": [],
    "cycle_id": ""
  },
  "idempotency_key": "campaign-1"
}
```

`plan preview` uses the same `plan` without `idempotency_key`, and `cycle
append` takes it with the target `cycle_id`. The backend
validates expansion and materializes cases; zero `max_cases` selects its limit
of 1000. A plan selects templates or registered case IDs. Each materialized
case is labelled `<Template Label> #<n>`, numbered per template from `#1`; a
cycle being appended to continues its own numbering, and
the case's parameter values stay on the case rather than in its label.
Until a cycle or execution seals, operators can rename it with the pencil
beside its title in the desktop app or with `Execution/RenameCycle` and
`Execution/RenameExecution`; a sealed, archived or History subject refuses with
`FailedPrecondition`. Renaming an execution renames its case, every attempt
included, and each rename is recorded as an `operator.label_edited` record
carrying the old and new label. Numbering continues from each case's stored
ordinal, so a renamed `#2` is never reused. For `cycle create`, save this
canonical request as `cycle.json`:

```json
{
  "metadata": {
    "identifier": "campaign.1",
    "label": "Campaign 1",
    "description": null,
    "tags": [],
    "attributes": [],
    "created_at": null,
    "updated_at": null
  },
  "continue_on_failure": false,
  "context": null
}
```

```sh
dokime cycle create --input-file cycle.json --case-id CASE_ID --request-key run-1 --wait-seconds 60 --backend lab --json
```

For scripts, `--case-id` appends registered cases right after creation and
`--request-key` then runs the cycle. Creation prints and flushes the cycle ID
before appending or submitting work. Retain the
cycle ID and run key to recover an uncertain admission. Creation, admission and
an optional terminal receipt are separate JSON lines; a later failure leaves
already emitted recovery identities available. Creating a cycle does not
itself run it. Waiting locally does not seal a report or cancel backend work.

## Examples

### Qualify An On-Board Charger

Harmos supplies the runtime and plugin boundary. Dokime adds the testing domain:
procedures, parameter expansion, operator interaction, acceptance criteria, and
self-contained reports.

The [OBC charger simulation](plugins/obc-plugin/src/suites/) demonstrates startup,
steady-state measurement, overcurrent protection, and recovery. It models a
charger testbench for learning the workflow; it is not a validated electrical
model or a ready-made vehicle qualification standard.

### Define The Operating Points

A shortened version of OBC's
[steady-state parameters](plugins/obc-plugin/src/suites/steady/measurement.rs) declare the
nine-case voltage/load sweep:

```rust
use dokime_runtime::definition::ParameterDefault;

#[dokime_runtime::parameters]
pub struct Parameters {
    #[parameter(
        label = "AC Voltage (RMS)", unit = "V",
        expansion = "cartesian",
        default = ParameterDefault::space([207.0, 230.0, 253.0]),
    )]
    pub ac_voltage_v: f64,

    #[parameter(
        label = "Load Impedance", unit = "ohm",
        expansion = "cartesian",
        default = ParameterDefault::linspace(80.0, 120.0, 3),
    )]
    pub load_ohm: f64,
}
```

The template names its parameter type once. Every expanded case resolves those
inputs before execution, so the procedure and its document see the same values.

### Run The Procedure And Capture Evidence

OBC's [steady-state template](plugins/obc-plugin/src/suites/steady/measurement.rs)
initializes shared device resources, energizes the charger, observes its
waveform, asks the operator to inspect the trace, and attaches a measurement
summary. Cleanup stops the simulated device.

This excerpt is the template's measurement observer. `InputMeasurement`,
`Samples`, `ObserverStreams`, and `reading` belong to the example:

```rust
#[dokime_runtime::observer(interval_ms = 5, streams = ObserverStreams)]
async fn samples(ctx: &mut ObserverContext<'_>) -> Result<(), Error> {
    let row = reading(ctx.resources())?;
    ctx.stream::<InputMeasurement>()
        .into::<Samples>()
        .push(row)
        .await;
    Ok(())
}
```

The same template uses `ctx.ask(...)` for operator sign-off and writes its
summary through `ctx.artifacts()`. Its criterion checks that enough samples
were captured and that the completion record accepted the run:

```rust
#[dokime_runtime::criterion(id = STEADY_ACCEPTANCE_CRITERION, label = "Trace Accepted")]
fn accepted(ctx: &CriterionContext<'_>) -> Result<bool, Error> {
    Ok(
        ctx.samples::<Samples>().len() as u64 >= ctx.parameters().sample_count()?
            && super::super::support::decided(
                ctx.records::<Completed>(),
                |run| run.accepted,
            )?,
    )
}
```

These are excerpts from the complete template, not standalone programs.
The [protection template](plugins/obc-plugin/src/suites/protection/overcurrent.rs)
adds an automated load ladder and verifies that an overcurrent fault disconnects
the supply and prevents restart. A real charger integration can extend these
procedures with regulation, efficiency, and thermal criteria for its hardware.

A step also reaches two built-in operations through `ctx.host()`, with nothing
to declare. `RunProcess` runs a vendor tool in the execution's artifact folder,
sends its output to the execution log or into an artifact, and records its exit.
`WatchFile` captures a device log written during the step into an artifact and
turns matching lines into rows and records. The
[startup template](plugins/obc-plugin/src/suites/startup/sequence.rs) tails the
simulated charger's log this way:

```rust
let watch = ctx
    .host()
    .submit_as::<WatchFile>(WatchFileRequest {
        execution_id: None,
        source: Some(FileWatchSource {
            payload: Some(FileWatchSourcePayload::Local(LocalWatchSource {
                root: self.log.to_string_lossy().into_owned(),
            })),
        }),
        output: "charger".into(),
        artifact: Some("charger/device.log".into()),
        policy: FileWatchPolicy::Stream,
        poll_interval_ms: Some(100),
        maximum_seconds: Some(1),
        maximum_bytes: None,
        matchers: vec![LineMatcher::publishing::<ChargerLog>(
            r"t=(?P<elapsed_s>[\d.]+) state=Running input_power_w=(?P<input_power_w>[\d.]+)",
        )],
        markers: Vec::new(),
    })
    .await?;
// The watch takes its baseline first; the charger then logs its readings.
let captured = watch.join().await?.into_owned();
```

Anything a step leaves running is cancelled when the step ends, with a warning
record.

### Present The Result

Compose plots from the same typed measurement fields used by the procedure.
This shortened example follows OBC's
[measurement document](plugins/obc-plugin/src/suites/common/measurement.rs); `stream` selects
the suite-owned stream carrying `InputMeasurement` rows:

```rust
let mut voltage = Plot::xy_line("Battery Voltage");
voltage.unit("V");
voltage.x(InputMeasurement::column_elapsed_s().on(stream));
voltage.y(InputMeasurement::column_dc_voltage_v().on(stream));

let mut charging = Section::new("Battery Charging");
charging.plot(voltage);

let mut document = Document::new("default", "Charger Qualification");
document.section(charging);
```

Column handles keep plot expressions tied to the declared data shape, and they
compare straight into filters: `voltage.only(column.gt(0.0))` narrows a plot,
and `section.each(column, "Point {value}")` repeats a section once per distinct
value, in ascending order. Templates can also add parameter-based reference
bands; the
[soak starter](crates/cargo-dokime/templates/src/suites/soak/hold.rs) demonstrates
a target-temperature band and event markers.

Time series and scatter plots list their traces in a statistics table beneath
the plot instead of a legend: count, min, max, mean, median, std, and rms of
each trace over the window shown, computed by the backend over the exact rows
rather than the reduced points the plot draws. Clicking a row hides or shows its
trace, a live plot's table also reads the value under the cursor, and the report
freezes the table with its plot.

Open or download a cycle's or an execution's **self-contained report** to share
the plots, verdicts, and context without requiring access to the testbench. A
running cycle's report is materialized at a fixed persisted-data boundary. Final
reports retain their original HTML and frozen execution context. A cycle report
queries each member's retained evidence at its original persisted boundary. Missing or damaged report artifacts
produce an explicit error. Stopping a test preserves the last persisted evidence
boundary for report finalization. If publication fails, the recorded test
verdicts stand and no partial report is kept; stopping the cycle again publishes
from the same persisted boundary, including after a restart. Stopping a cycle no
run will finish, even an empty one, seals it with its report.

## Cargo Features

These flags configure the `dokime-runtime` plugin SDK dependency. Desktop
application users do not need to select them. The default SDK includes native
plugin support and authoring macros; operation, model and record definitions
live in `dokime-domain`, which every feature set shares.

| Feature | Default | Enables |
| --- | --- | --- |
| `native` | Yes | Native plugin SDK, sidecar support, and plugin prelude. |
| `derive` | Yes | Suite, template, step, observer, and other authoring macros. |
| `storage` | No | Storage APIs and the SQLite backend. |
| `host` | No | The application host the server embeds: storage, gRPC and host collectors. |
| `fft` | No | Spectrum analyses on top of `host`. |
| `simulation` | No | Simulated time and devices on top of `host`. |
| `media-native` | No | Native camera, microphone and screen capture on top of `host`. |

The application stores state checkpoints and their journal through Harmos's file
state backend; SQLite holds the data catalog and evidence. A root written by an
earlier storage layout is refused as incompatible with this build and needs a
fresh storage root. Existing data is never silently reset.

For remote automation, depend on the reusable client crate:

```toml
[dependencies]
dokime-client = { git = "https://github.com/stokker-technologies/dokime", rev = "<tested-commit>" }
```

Replace `<tested-commit>` with the revision used by your integration.
`dokime-client` owns remote connection, authentication, discovery, listening,
recovery and profile access. The `dokime-runtime` crate owns plugin authoring; its
`storage` feature does not enable `native` or `derive` when defaults are disabled.
Disabling defaults retains the SDK's underlying Harmos dependency. See the
[SDK manifest](crates/dokime-runtime/Cargo.toml) and
[client manifest](crates/dokime-client/Cargo.toml) for details.

## Documentation

- [OBC Plugin](plugins/obc-plugin/) — the charger simulation and complete test templates.
- [Plugin Starter](crates/cargo-dokime/templates/) — a complete temperature-soak procedure.
- [Runtime And Plugin SDK](crates/dokime-runtime/src/lib.rs) — public authoring APIs and module documentation.
- [Client API](crates/dokime-client/src/) — programmatic access to cycles and executions.
- [Contributing](CONTRIBUTING.md) — repository setup and development workflow.

## Project Status & Roadmap

Dokime is in active development. Pin the revision you integrate: public APIs,
plugin contracts, and development storage formats may change before 1.0.
Existing development data has no compatibility guarantee; incompatible storage
must use a fresh root rather than silently resetting an existing one. When the
desktop's local backend refuses its data root, the startup failure screen offers
**Back Up And Reset Local Storage**: it archives the root into
`<root name>-backup-<UTC stamp>.zip` beside it, or wherever you pick, then
empties the root and starts the backend again.

The public introduction will grow with a complete on-board charger walkthrough
and a downloadable self-contained example report. GitHub hosts the authoritative source and native validation workflows.

Follow [issues](https://github.com/stokker-technologies/dokime/issues)
for development discussions and [the changelog](CHANGELOG.md) for recorded changes.

## Development Setup

Use the source setup in [Quick Start](#quick-start). For a headless server:

```sh
mise run build
cargo run -p dokime-server-cli --
```

The repository flows are mise tasks:

```sh
mise run format --check
mise run lint
mise run check
mise run build
mise run build:rust:tests
mise run check:python:generated
mise run check:typescript:generated
mise run test
mise run package
mise run release:publish:dry
mise run ci
```

`check` combines language formatting/linting, Rust feature checks and the
WebAssembly domain build. `build` exports shipping binaries, the OBC bundle,
frontend and generated clients under `target/build/<profile>/<target>/`.
The `dokime-cli` and `dokime-server-cli` crates own the `dokime` and
`dokime-server` executables and never publish to crates.io. The `dokime` and
`dokime-server` library packages retain their public names and APIs.
Raw release downloads use their Cargo package identities, for example
`dokime-cli-<version>-<Rust-triple>[.exe]`; installation keeps the `dokime`
command name. Desktop installer product names stay unchanged.
`build:rust:tests` separately builds the test archive and fixtures and verifies
compile contracts in their build tree. Archived Rust tests resolve resources from
nextest’s runtime package directory after workspace remapping; consumers use
their checkout and transported Build outputs. Generated-client static checks run as
`check:python:generated` and `check:typescript:generated` after Product Build.
`test` runs existing artifacts through language layers:

| Task | Layer |
| --- | --- |
| `mise run test:rust:unit` | Curated invariants inside the crates. |
| `mise run test:rust:integration`, `test:python:integration`, `test:typescript:integration` | In-process crate and client stories. |
| `mise run test:rust:acceptance`, `test:python:acceptance` | Stories over one process and a client: server, CLI and Python client. |
| `mise run test:rust:system`, `test:typescript:system` | Server/OBC walkthrough and the Linux native desktop walkthrough. |

The desktop walkthrough drives the built shell through WebKitWebDriver on Linux
and needs a display. `mise run ci` runs the applicable local checks, producers,
tests, packaging and dry publication; native provider CI proves every configured
platform. `mise run ci:nightly` adds coverage and a separate benchmark producer.
CI installs runtime libraries for prebuilt test jobs; compilation and packaging
jobs retain their declared development dependencies.

The runtime Cargo package carries its embedded report and brand payload.
Contributors refresh that tracked payload with `mise run prepare:report` when
changing the GUI report or root branding; Rust Check verifies its freshness.
See [Contributing](CONTRIBUTING.md) for prerequisites and the full workflow.

### Generated Clients

The Rust build generates the protobuf descriptors of every public operation;
`mise run build` regenerates the Python package's contract files from them with
Harmos's `cargo harmos bindings python` exporter and refuses a stale tracked
copy. The desktop's TypeScript client is generated from the same descriptors
by the Buf generator after the Cargo build. Ordinary wheel and source-package
installation uses the tracked files and needs no Rust compiler.

The desktop WebView never talks to the network. Every call from the generated
client crosses one opaque IPC bridge into the shell, which forwards the
protobuf bytes undecoded to the session's Rust client, owned backend or
attached: one `rpc` command for unary calls, and one Tauri channel per server,
client or bidirectional stream. The bridge knows paths and bytes only, fences
every request and reply with the connection generation, cancels a stream when
its subscription or session ends, and bounds messages by the limits the backend
advertises. Endpoints and credentials stay in the shell. Artifact uploads and
recordings keep a native file-backed adapter, because their bytes are hashed and
patched in the shell rather than framed by the WebView.

## Contributing

Bug reports, focused fixes, clearer examples, and equipment integrations are
welcome. [Open an issue](https://github.com/stokker-technologies/dokime/issues)
to discuss a proposal, and read the [contribution guide](CONTRIBUTING.md) before
submitting changes.

## License

Dokime is licensed under [Apache-2.0](LICENSE-APACHE).
