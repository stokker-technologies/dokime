# Dokime Plugins

Python processes can contribute embedded operations, observers, settings and
execution templates through `dokime.Plugin` and `Backend(plugins=[...])`. They use
the same managed generation and publication boundaries as native providers. See
[Python Authoring](../sdk/python/README.md#author-an-embedded-plugin) for declarations,
callback ownership and batch publication. These providers live with their Python
backend; distributable Python plugin bundles remain outside the current SDK.

## Security

Read [Plugin Security](plugin-security.md) before installing or distributing a
plugin. It defines bundle verification, native process authority, secret handling,
the threat model, private-plugin approval and current API compatibility limits.
Publisher signing, organization allowlists and WebAssembly isolation remain proposals.

## Python Projects

Create a project with `dokime new bench --lang python`, install the matching SDK
with [uv](../sdk/python/README.md#install), and run `uv run python main.py`.
The [runnable source](../sdk/python/examples/starter/) owns the generated files:
`main.py` composes the backend; `plugin/__init__.py` assembles its catalog;
`plugin/operations/read.py` owns the instrument request, policy and handler;
`plugin/suites/capture.py` owns parameters, prompt, stream, steps and criterion.
Python operation handlers are the driver boundary, just as Rust operations are.

```python
# Read Voltage
operation = Read(index)
voltage_v = await ctx.operation(operation)

# Publish the Reading
row = Row(index, voltage_v)
await writer.push(row)
```

Async callbacks stay on the starting event loop; synchronous operations run on
workers. Resources are plugin-owned shared objects, while serialized requests
and replies are independent values. Native deadlines and cancellation still
apply. A retained callback context refuses calls after its owner ends. Completed
evidence and sealed reports can be reopened without registering the Python
plugin again; the GUI shows its unavailable template without offering a new run.
Installed plugins always load and reserve their IDs, including when disabled.
Changing the Python plugin means restarting its backend; no Python sidecar bundle
or hot-reload lifecycle is provided.

## Native Plugin Runtime

Startup failures appear in a desktop notification and in Settings › Plugins,
with the bundle's display name and the underlying cause. The verified ordinary
settings remain editable even when the plugin could not start. Secret declarations
show their shared Secret Storage identifiers and availability read-only. Use
**Add in Secret Storage** to open Add Secret prefilled with a missing identifier,
then use **Reload** on the plugin page, which starts that plugin again while Dokime
keeps running. There are no secret bindings or environment-variable sources.
Notification history retains the settings link.

A failed boot has one typed reason with the plugin identity, message and refusal
code. Initialization failures also retain the encoded Harmos failure, including its kind,
application refusal code, named context and source diagnostics. Plugin authors
should return errors through `dokime::plugin::refusal` to preserve this context;
missing secret access is classified by `MISSING_SECRET_CODE` and the
`dokime.secret` field, without parsing diagnostic messages.

`plugins/sidecars/host/` supplies the six host telemetry streams through the supervised
plugin ingest boundary: `host.system`, `host.network` (with OS-reported
`internet`), `host.wifi.link` (the current connection), `host.wifi.scan` (one row
per nearby network), `host.wifi.latency` (round trip and loss to the latency host)
and `host.bluetooth`. Separate operation, stream and observer catalogs share
the same declarations; each stream has its own demand-driven observer,
which runs only while a lease holds that stream. Operation definitions own the
policies and `settings.rs` owns the configurable collection intervals and native probe settings; Windows
and Unix implementations are selected at module declarations. Windows radio and
network probes use native APIs; the system collector reads Git identity through
a bounded Git subprocess. Its Collection settings retain the
Wi-Fi/Bluetooth adapter selectors and latency host, plus the minimum interval
between active Wi-Fi scans; a scan due sooner reports the cached network list.
The [Host Settings](product-guide.md#host-settings) section lists native probe
timings, collection intervals, refresh and sampling settings, plus static operation policies and replay windows.
Empty adapter selectors use the native default. Native repeated fields
carry lists; acquisition timestamps travel in framework publication metadata.
`host.network` carries aggregate counters, `internet`, and the default-route
adapter id. Timestamped `link_up`, `link_down`, `address_added`, `address_removed`
and `default_route_changed` rows come from Windows IP Helper callbacks or Linux
route netlink notifications, held only for the observer's lease. A receive
overrun fails observation visibly; inventory is never polled to infer events.
The Public `plugin_host::host::adapters::Adapters` query (`host.network.adapters`)
returns adapter identity, kind, MAC, status, MTU, speed, IPv4/IPv6 prefixes and
scope, gateways, DNS servers, DHCP state and default-route ownership. Linux
uses `getifaddrs`, kernel routes and NetworkManager's per-device DNS/DHCP data;
DHCP is unknown when NetworkManager does not manage an interface.
The installed host contributes a synchronous `RunStart` capture: every execution
records its adapter inventory before the first step.

The Public `host.network.radio.set` request uses the shared
`plugin_host::domain::controls::{Radio, RadioPower}` enums:

```rust
use plugin_host::{domain::controls::{Radio, RadioPower}, host::controls::RadioSet};

let request = RadioSet {
    radio: Radio::Wifi, // Wifi, Bluetooth Or Cellular
    state: RadioPower::Off, // On Or Off
    allow_disconnect: false,
};
```

`RadioState.kind` uses the same `Radio` enum. Adapter inventory uses
`host::adapters::{AdapterKind, AdapterStatus}` for kind and status; identities,
addresses and native power-plan IDs remain strings. Requests and inverse
snapshots encode enum variants, with no legacy string aliases. Unknown native
radio-control values return `plugin_host::Error::UnknownValue`; telemetry
renders known Wi-Fi and Bluetooth states as text. Unknown Windows Wi-Fi link
values render as `unknown_<n>` while other fields remain available; see
[Windows host telemetry](product-guide.md#host-telemetry) for partial
samples and location permissions.
It uses Windows.Devices.Radios or NetworkManager/BlueZ native switches.
`host.network.adapter.set` takes `{ adapter_id, enabled, allow_disconnect: false }`;
Windows uses quiet SetupAPI and refuses without administrator rights, without UAC.
Linux requires root or CAP_NET_ADMIN for interface writes.
Before disabling, the host maps the trusted backend PID's established native TCP
sockets to adapters. A change affecting a live connection refuses unless
`allow_disconnect: true` comes from an execution step; a standalone request flag
cannot bypass the guard. Bluetooth device presence uses native enumeration.

Radio, adapter and `host.power.plan.set { plan_id }` changes preserve the exact
prior state as invertible transactions. During an execution the backend durably
journals the inverse before writing and records native apply/restore timestamps.
It keeps one inverse per `(RecordScope, resource)`: the earliest inverse preserves
the original state, and later prepares never replace it or move its restoration
position. Cleanup restores each resource once in reverse first-prepare order
after completion, failure or cancellation. Each ticket retains its calling
provider for confirmation ownership and evidence; cross-execution resource
exclusion still applies.

Every prepare gets a unique ticket, including coalesced calls. Confirming an
applied call records that ticket's native timestamp. If a later write fails, the
host rolls back to that call's prior state and confirms `restored: true`; this
does not retire the earliest inverse. An unconfirmed prepare also leaves it
pending. A rollback of the first call retires the inverse only when no later
prepare has joined it. Later tickets and their metadata persist with the kept
inverse until resource restoration succeeds.

`test.execution.effect.prepared` and `.applied` describe individual calls using
their own `ticket`, `prepared_at`, description and native `applied_at`.
`restoration_ticket` identifies the earliest prepare; `coalesced` is true for
later tickets. `.restored` with `resource_restored: false` records a call rollback
that leaves the inverse pending. Cleanup or recovery emits one `.restored` with
the earliest ticket and `resource_restored: true`. A first-call rollback with no
later prepares also has `resource_restored: true`. Prepared/applied records have
`resource_restored: false`.

Cancellation keeps native work owned until it settles. On restart, pending
inverses run before startup returns; a failed or unavailable inverse blocks
startup and stays pending. Keep the restoring plugin installed. Standalone
changes persist without run-end inversion. Development storage stays v1; use a
fresh storage root for incompatible schemas and preserve the previous root.
The Public `host.power.plan.get` query reads the active Windows power scheme
or Linux ACPI platform profile; unavailable native reads return a refusal.
The Public `host.cpu.load` query and `host.system` rows expose calibrated
capacity and estimated load in GFLOP/s. At plugin start, a bounded kernel runs
eight independent f64 fused multiply-add accumulators on each available logical
CPU. Each FMA counts two operations; completed work divided by total wall time
gives kernel-equivalent capacity. The calibration window and deadline-check batch
size come from host settings; the default window is half the CPU operation's attempt policy. Estimated load is capacity multiplied by OS utilization / 100;
it is a workload-equivalent estimate, rather than a measured instruction count
or the processor's theoretical peak. Calibration reflects the compiled kernel,
current machine and power state at startup. The method is beside `CpuLoad`.


### Closed-Loop CPU Load

The host plugin provides the Public request
`plugin_host::host::cpu::load::LoadHold` (`host.cpu.load.hold`):

```rust
LoadHold { target: LoadTarget::Percent(60.0), threads: None }
LoadHold { target: LoadTarget::Flops(1_000_000_000.0), threads: Some(2) }
```

Finite percent targets lie in `(0,100]`; finite FLOP/s targets are positive.
`threads: Option<u32>` accepts `1..=logical_cores`; `None` selects all available logical cores. Percent
control includes other processes and only adds missing host load. FLOP/s control
counts this generator's work; the OS exposes no system-wide floating-point counter.
Unreachable targets saturate at zero or full duty rather than claiming success.
Existing `host.cpu.load.capacity_gflops` remains the sole startup capacity estimate;
`LoadTarget::approximate_flops` and `approximate_percent` convert targets using it.
The calibration and generator execute the same counted eight-accumulator FMA kernel.

CPU sampling has one fixed-cadence owner. `host.system.collect`, `host.cpu.load`
and control read its latest completed window, so consumer cadence cannot shorten
CPU delta windows. Memory and other system probes remain collected on demand.

`CpuLoadSettings` in `plugins/sidecars/host/src/settings.rs` owns all control
timing and tuning beside the other host tunables. Plugin settings expose the CPU
Load Control section:

| Setting | Default | Meaning |
| --- | --- | --- |
| `period_ms` | 250 | Controller period |
| `sample_ms` | 250 | Shared CPU sampling period; at least the native sampler minimum |
| `slice_ms` | 10 | Busy/sleep slice per worker |
| `proportional_gain` | 0.5 | Proportional gain on normalized error |
| `integral_gain` | 0.5 | Integral gain per second |
| `integral_min`, `integral_max` | -1, 1 | Clamped integrator bounds |
| `settlement_ms` | 2000 | Worker settlement budget |
| `batch_iterations` | 4096 | Worker kernel iterations between stop checks |

Sampling and slices must fit a controller period; settlement must cover a slice.
The operation policy has no practical deadline and never retries. Its 60 s cancellation
grace is defined beside the settings defaults; the settlement budget must fit that grace. Actual elapsed
wall time determines counted FLOP/s and the duration-weighted mean and RMS error.
The PI controller clamps duty to `[0,1]` and back-calculates integration on saturation.

The distinct `host.cpu.load.control` stream carries the selected mode, nullable
percent or FLOP/s target/measurement columns, duty and generator FLOP/s. Columns
have structured units and bracketed labels such as `Target [%]` and
`Generator [FLOP/s]`; existing estimated load uses `Estimated CPU Load [GFLOP/s]`.

Use the runtime invocation builder’s `.start()`, hold, then `.cancel().await?` to stop and join, returning
`LoadSummary { target, mean, error_rms, duration_s, samples }`. Dokime's step/run handle
sweep cancels forgotten handles; native drop cleanup also stops and joins workers.
No effect-journal inverse is needed: stopping restores CPU use and process death
cannot leave generation running or persistent native state changed.

`mean` and `error_rms` use the selected target's units; `samples` counts measured
intervals, including the final partial interval. Cancellation during publication
also settles the workers and returns the final summary. The OBC Checks template
`obc.checks.cpu_load` demonstrates the complete consumer sequence using the
step’s `ctx.operation(operation).start().await?` builder to start its handle.

The runtime decodes these rows through the same codec as other plugins.
`host.system` also exposes `memory_used_gib` (Memory Used [GiB]) and
`memory_total_gib` (Memory Total [GiB]), with structured unit metadata `GiB`.
Each value divides the existing byte reading by 1024³. On Linux and Windows,
both readings share the snapshot used for `memory_pressure_percent`, which is
100 × used / total; pressure is absent when total memory is zero. The original
byte columns remain available. Host document defaults plot used and total GiB;
OBC system sections plot used GiB beside memory pressure.
Direct `RuntimeHandle::publish_telemetry_batch` calls also retain original
canonical payload bytes. If their value map omits `timestamp`, the runtime
infers it from the sample instant using the contract's column type: `Datetime`
preserves the full instant, while `Number` carries seconds since the Unix epoch.
Explicitly supplied timestamp values remain subject to contract validation.
`plugin-host` is one crate: with no features it is the light definitions crate
(operation requests and responses, ids, policies and stream rows); the `runtime`
feature adds the operation implementations, platform dependencies and the
sidecar executable, which the product build enables. The concrete plugin and
its native declaration live in the binary's `src/app.rs`; the library owns
contracts, native resources and observers parameterized by the serving plugin.
Even when workspace feature unification enables `runtime`, linking the library
cannot introduce a host sidecar declaration into another plugin's executable.
It is never published to crates.io; the host ships as the `plugin-host.dokime` plugin
artifact. A plugin outside this repository calls host operations through the
definitions from a Dokime release tag, matching the tag used for its `dokime`
SDK dependency:

```toml
plugin-host = { git = "https://github.com/stokker-technologies/dokime", tag = "v0.2.0", default-features = false }
```

Host stream rows and contracts live in `plugin_host::domain::telemetry` (the rows are also exported at the crate root). Host document defaults and `wifi_channel_occupancy` live in `plugin_host::domain::document`. Dokime itself owns no host stream definitions; its `dokime::host` module contains only application lifecycle records. Install the host bundle before admitting templates that subscribe to `host.*`: absent publishers receive the ordinary per-stream refusal, and valid leases remain independent. Application-registered catalogs retain `PluginOwner::Host` and appear as Built-in.

Its bundle and release-feed name are `plugin-host`; the stored sidecar owner is
`plugin_host`, using Harmos's portable owner spelling; `host` names application-owned operations
in Harmos. The bundle is `plugin-host.dokime`, with `host.*` stream and operation IDs.
The host sidecar declares no automatic restart: a process fault deactivates it
until an explicit reload or the next startup.

Host observers run only while their streams are leased. Execution subscriptions
and live plot viewers hold those leases; releasing the last lease stops the
observer while the plugin process can remain idle.

`plugins/sidecars/obc/` is the test-only [onboard charger plugin](plugin-obc.md). Its `runtime/` crate owns
the simulated testbench. Its bundle is `plugin-obc.dokime` and stored owner is
`plugin_obc`. It is built explicitly for tests and is never included
in the product's default bundles or installer.

A plugin declares its identity, requirements and contributions on
`#[dokime::plugin] impl dokime::Plugin`; `const CATALOGS` lists its executable
`#[dokime::harmos::catalog]` operation catalogs. Contribution catalogs use
`#[dokime::catalog]`. Each request struct declares its id, policy and
`resources = [...]`, and implements `Operation::run`.

A separate `impl dokime::harmos::Lifecycle` owns configuration, errors and the
`prepare` → `initialize` → serve → `stop` → `cleanup` lifecycle. Preparation
registers every shared resource with `setup.resource(...)` and prepares suite
resources with `dokime::execution::prepare::<Self, Suites>(setup)`.
Initialization activates resources that need a running runtime. Operations,
listeners and observers read prepared resources from their execution contexts;
mutable resources register as `Arc<tokio::sync::Mutex<T>>` or
`Arc<tokio::sync::RwLock<T>>`. Observer attachments are prepared from their catalog
and the plugin's declared streams.

On restart, Harmos prepares and starts plugin generations sequentially in
dependency order, matching declared required contracts to provided contracts
exactly, including embedded providers and host operations. Providers become
ready before consumers; independent plugins have a deterministic order based
on plugin names. Bundle inspection and extraction happen before activation
and do not run plugin lifecycle hooks.
Installed packages with unavailable or circular contracts are refused while
unrelated plugins start normally. Settings → Plugins retains the boot reason,
and the `plugin_contract_unavailable` or `plugin_contract_cycle` admission code.
Launch configuration seeds unsaved fields; explicitly saved values take precedence.
Saving settings preserves the running generation and its settings snapshot in execution evidence. The backend compares saved
values with that generation's startup values and reports differing setting IDs.
The GUI marks these **Applied on Reload**. **Reload** reads saved settings and
restarts the program, its dependents and all their observers, including
`activation = "plugin"` observers, which resolve their new collection intervals.
A reload refused before the program stops keeps the running generation and its
pending setting marks; a generation that fails to start is recorded like a failed boot.
An unrelated plugin's retained cleanup failure does not join that reload. The failed
owner and plugins that depend on it still require successful cleanup before replacement.
Unavailable-contract reasons identify the owner and required contract; cycle
reasons identify every involved plugin.

### Observer Refusals and Diagnostics

Harmos refusal `-32607` means the operation or route deadline elapsed. It does
not identify a restart or a retired generation: operation execution, lane admission,
device contention and an unavailable remote application can all spend the same
budget. The diagnostic retains the called operation and underlying cause. Calls
keep their authored policies, and uncertain writes are never replayed automatically.

Reload fences ordinary callers and joins the old observers before shutdown and
generation retirement. Their owned calls can settle against that generation.
A confirmed carrier exit during shutdown is successful physical termination;
an authored cleanup error still fails cleanup. A disconnected observer host ends
its activation. Repeated identical callback failures log once until success or a
different failure; recovery has its own record.

An observer awaits each callback serially and skips missed ticks. A remote call
that uses its whole 30 s deadline therefore delays that observer's next sample;
changing cadence alone cannot shorten it. For SDK evolution, an optional per-call
budget capped by the operation policy, or a separately owned connection sampler
with a cached availability snapshot, could support faster failure. Neither is
implemented here; plugin code and operation deadlines are unchanged.

Sidecar stderr groups continuation lines into bounded records and preserves
severity, including `error:` diagnostics. Operator cancellation logs at INFO;
step and cleanup failures remain ERROR. Routine file producer completion is DEBUG.

The unresolved-symbol backtrace hint means the matching debug information is
absent. Release builds can retain symbols separately: matching PDBs on Windows
and split debug information on Linux can be optional diagnostic downloads, keeping
normal installers small. In-process Windows symbol resolution needs the matching
PDB beside the executable; offline symbolization needs the exact binary and its
symbols. Current bundles do not promise these symbol artifacts. See the
[Cargo profile reference](https://doc.rust-lang.org/cargo/reference/profiles.html#debug)
for debug information and splitting controls; adding symbol publication is a
separate packaging change.


Cargo supplies package identity, version and description. Suite declarations
produce their execution factories; authors do not maintain a second runtime
inventory.

Template runners stop cooperatively through `Signal<T>` with
`ExecutionSignal.cancel = true`, retaining the run's final response and evidence.
Cancelling the owning Harmos invocation aborts its child calls, including evidence
delivery, so run policies retain immediate abort for that control path.

`cargo dokime sidecar build` compiles the selected native executable, runs its bounded
catalog export, and creates a `.dokime` bundle containing the executable,
frozen registrations and `assets/`. Ordinary `cargo build` only compiles.
Catalog export must be deterministic: declaration identities use stable seeds,
while runtime execution identities remain fresh.

`mise run build:rust` produces the shipping `plugins/plugin-host.dokime` bundle.
`mise run build:rust:tests` prepares the explicit OBC test fixture under
`fixtures/plugins/plugin-obc.dokime`.
Operators select or install bundles; the product does not seed these fixtures. A card's probe
and dropdown sources come off the authored settings aggregate —
`PluginSettings::status_operation()`, `PluginSettings::option_providers()` — read
from the admitted catalog; the host has no compiled plugin adapters.

Release staging groups shipping sidecars into one native archive per target:
`dokime-sidecars-<version>-<target>.zip` on Windows and `.tar.gz` on Unix.
Members keep their declared unversioned `.dokime` filenames. `plugins.json`
format 2 lists plugin names, kinds, versions, Dokime requirements and target
archive/member/hash mappings. The desktop lists every compatible plugin from the
selected update source, verifies the archive against SHA256SUMS and the requested
member against the feed, then installs only that member. There is no v1 reader
or individual-plugin release-asset fallback. See the
[release feed example](product-guide.md#organization).

Custom installers use top-level `[[plugin]]` entries with exactly one of `name`
(a release feed name) or `path` (a local bundle relative to the organization
pack). The pack's `plugins/` folder is storage only; unlisted files never ship
and plugin files are not overlaid. Repeat `--plugin <path>` for equivalent
external bundle or sidecars archive inputs relative to the invocation directory.
Names come from feed and product declarations; they do not depend on crate naming.
See [Distribution From A Release](product-guide.md#distribution-from-a-release).

Plugin upload and installation use `Install::POLICY.timeout` as the maximum
silence between frames and while waiting for the final installation response.
Each pulled upload frame rearms the deadline, so active uploads can exceed that
budget in total duration. The server and desktop derive their waits from the
same operation policy; desktop and Rust client uploads carry no total
`grpc-timeout`. The existing
installation policy permits one hour of silence, including staging and
activation after upload. Bundle and frame size caps are unchanged.

Operators install bundles through **Settings → Plugins**. Installation validates
the complete bundle and frozen registrations without running author code, then
atomically binds an installation UUID to its SHA-256 in `settings.json`; immutable bytes live at
`<application root>/data/plugins/<UUID>.dokime`. The boot sweep deletes
unreferenced files only inside this store-owned directory. Installer bundles
remain read-only inputs under `<install directory>/plugins`, separate from the
store even when the install directory equals the application root. Files copied
into the store manually are not admitted. Earlier development layouts require
a fresh application root; existing files are never moved or reset automatically.
`DOKIME_PLUGINS_DIR` (or
`--plugins-root`) names a development directory whose bundles load for one
process; it is never a store. A corrupt `settings.json` fails explicitly; missing
or changed bundle bytes remain visible by UUID and error, without displaying
unverified author metadata.

Installation checks configured and installed owner collisions before publication. Replacing an installed
package marks its previous version for removal after successful activation; package names identify
installations, while executable IDs identify dependencies. Boot selects native
files for the current platform, verifies frozen and live registrations before
initialization, and admits catalogs only after every surviving package initializes.
A failed package loses all its executable siblings and dependent packages.
A bundle that declares a Harmos guest is refused at admission.

Catalog ownership is explicit. Host registrations survive plugin removal.
Provider reconciliation rebuilds the catalogs owned by configured, embedded and
installed plugins from their admitted registrations.

## Operation Invocation

Use `ctx.operation(request).await?` in handlers, templates, criteria and observers.
Await returns Harmos `Response<T>`, which dereferences to the value; use
`into_owned()` when taking owned fields. `.start().await?` returns a scoped instance.
The operation macro owns default timeouts, delays and retries. `.retry(settings.retry)`
overrides one call, using settings, parameters or configuration. Enabled overrides
require declared replay safety, preserve attempt deadlines and record every attempt.
Template built-ins such as StartCapture and RunProcess use the same method without a
`requires` entry; listing an unconditional built-in there is a compile error.
Import `OperationHost` for carrier hosts. Rebuild sidecars and guests together for child-call envelope v2.

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
  attach it. `Document::new("Hold at Temperature")` derives `hold.at.temperature`;
  set `document.id("hold");` when label edits must keep the key. Validation
  refuses invalid or empty derived keys and collisions in one registry, naming
  the labels and directing authors to `.id(...)`. The runtime owns the internal
  `default` view and merges template content into it.
- Suffix field names with their unit (`duration_s`, `distance_m`); separate
  attributed fields with a blank line.
- Use the same type names in every template file: `Template`, `Parameters`,
  `Row`, `Required`, `Streams` and `Prompts`. Catalog variants match their
  operation names.
- Cleanup attempts everything and logs a warning for each failure.

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

## Plugin Startup and Stop Budgets

Two host settings bound a sidecar program's lifecycle outside its own code.
They live in `PluginPreferences` in
`crates/dokime-domain/src/model/settings/application.rs`, are edited under
**Settings › Plugins** and are read again at every start, restart and stop, so a
saved change applies to the next one, including live installation changes and
explicit plugin reloads.

| Setting | Default | Range | Bounds |
| --- | --- | --- | --- |
| Plugin Startup Timeout [s] | 60 | 1–3,600 | Launch, handshake and endpoint staging of each start attempt |
| Plugin Stop Timeout [s] | 10 | 1–3,600 | Settling of the executions a stopping or replaced plugin owns |

The startup budget covers what happens before the plugin's lifecycle code runs:
the process launch, including the operating system loading the libraries it links,
its handshake and the staging of its native endpoint. A program that exceeds it
fails to start with `Plugin <id> could not initialize` and a cause naming the
budget and this setting. `prepare` and `initialize` themselves run afterwards,
bounded by the plugin's declared lifecycle `INITIALIZATION_POLICY` (and
`SHUTDOWN_POLICY` for stop), a Harmos operation policy the plugin authors with
its other declarations. A plugin that opens a slow vendor client during
`initialize` declares a longer initialization policy; the host setting is not a
per-plugin declaration.

## Live Installation Changes

Installing, reinstalling or updating a bundle, and enabling, disabling or removing
one, take effect in the running backend without restarting Dokime. The plugin
store records the change first, and the backend's restarter
(`crates/dokime-runtime/src/runtime/plugins/installs.rs`) then brings the running
programs in line with it through the same reload as an explicit Reload: a new
bundle starts, a changed bundle replaces the running generation, and a disabled or
removed bundle stops. Each change is admitted like a startup. The desired catalog
is validated as a whole before anything stops: conflicting catalogs, unavailable
or circular contracts, and the packages that need a removed or refused provider.
A package that loses a provider it requires stops with it and records why, so the
running set matches what the next start would admit.

Every queued installation change or explicit reload waits while work can still reach
a plugin: a running, paused or stopping execution, a prepared run, the gap before
automatic continuation starts its next case, or plugin streams still draining.
The plugin shows **Restart Pending** and the change applies automatically once
that work completes. Completed means the execution outcome is terminal, plugin
cleanup and stream drain have settled, and the report's catalog inputs have been
frozen durably. The provider task protects that boundary and automatic continuation.
A cycle left open and idle holds no plugin. Sealing and report publication after
completion, including a failed or stuck seal, hold no plugin either. Executions
are never stopped or disturbed for a change, and further
changes to the same package join the one that waits, applying the latest.
**Reload** of a plugin that does not run starts it at once, even while a cycle is
active, because nothing running stops; reloading a running plugin restarts it once
no work can reach a plugin.

Finalization, Retry Seal and history reads use the frozen documents, appearance,
record and telemetry schemas, plugin analysis results, execution revision and
stored evidence. They do not require the plugin, its current catalog entry or
its contributions to remain installed. Starting another case in an open cycle after its plugin is removed
fails with `execution_plugin_unavailable`; install its original revision or add a
case from an available plugin. Replacing a plugin with another revision still
requires newly materialized cases for that revision.
Report processing limits remain editable: raising one can repair a failed report
without changing its frozen evidence, presentation or plugin analysis results.

A replacement refused before anything stops, because its bundle, catalog,
contracts, saved settings or secrets do not validate, leaves the previous
generation running untouched. Harmos stops a generation before starting its
replacement, so a replacement whose new program then fails to start brings the
previous bundle back as a fresh generation. Either way the previous installation
stays installed with its saved values, and the refused one is listed as
**Refused** with its failure, reported like a failed startup, until it is deleted,
a later replacement succeeds, or the next start deletes its file. A first installation
that fails stays installed with its failure; **Reload** starts it again once the cause
is fixed. Failed and refused installations always offer **Delete**; a refused
replacement offers neither **Reload** nor an enable switch. Removal stops the
plugin live once no work can reach a plugin, then discards its row and managed file.
Successful replacements discard every earlier installation of the same package,
including failed boots, along with their boot failures and startup notifications.
If deletion fails, the row and its error stay visible and **Delete** can retry;
startup retries too. Unreferenced
files are swept only at startup, so live cleanup cannot race an installation
publishing its bundle before its settings commit.
