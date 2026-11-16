# New Execution Host

The execution host serves the command-line tool and SDK clients.
`dokime-runtime` composes plugin entries, scoped unary lifecycle dispatch,
controlled orchestration time and cycle execution. `dokime-operations` owns
one declaration and handler per supported host operation. `dokime-storage` owns
SQLite execution evidence and Arrow IPC sample batches. The SDK depends only on product protocol and the library SDK.

## Live Ownership

`Host` composes storage, the execution manager, the prompt manager and the report
manager; its former `Inner` is gone. Service `State` is a composition adapter and
owns no execution/progress tasks, pending questions or temporary report files.

The execution manager owns cycle admission, queued/running state, invocation
sequencing and retries through operation policies, Stop/Cancel, restart recovery,
and progress. Cycles group its tasks rather than adding a scheduler. Dropping a
cycle observer leaves admitted work running. Shutdown refuses new admission,
cancels active work and joins every execution and progress task before returning.
Cancel refuses queued work; Stop lets the current invocation reach its safe boundary.

The prompt manager owns question waiters and answers, including durable client
answers. Execution settlement clears its question; shutdown settles waiters and
closes question admission. The report manager owns atomic publication and bounded
temporary report retention. Shutdown releases retained files and closes publication.
Rendering remains a pure storage read into temporary output, and never publishes.
Evidence, Catalog, notes, Aggregates, Plots and Documents have no manager. Media
still explicitly refuses its unsupported operations; this host has no live recording
owner requiring a capture manager. Local file captures are a registry, not a manager:
the execution settles its captures before closing evidence, so they join with it.

Guard **A5 — Managers Own Live Orchestration** is exercised by
`execution_shutdown_joins_runs_after_observer_loss`,
`prompts_settle_answers_and_shutdown_waiters` and
`reports_publish_atomically_and_close_admission` in
`crates/dokime-runtime/tests/system/manager_lifecycles.rs`.
Existing restart, retry, report, evidence and client stories retain their contracts.

## Run the Thermal Stories

Build the existing shared SDK plugin, then run either story by its full name:

```powershell
mise run build:host-fixtures
mise run test -- --package dokime-runtime --target system --exact thermal_suite_survives_host_restart::thermal_suite_survives_host_restart
mise run test -- --package dokime-runtime --target system --exact cancelling_a_thermal_step_runs_cleanup::cancelling_a_thermal_step_runs_cleanup
```

The fixture is `target/debug/dokime-rust-thermal` with the native executable
extension. `DOKIME_THERMAL_PLUGIN` can name an already built copy. Tests never
compile a plugin themselves. Both placements use the unchanged shared thermal
suite at `sdk/rust/examples/thermal.rs`.

The host requires explicit `HostSettings`: placement bounds and a
`SchedulerClock`. `ControlledClock` advances only through requested waits.
Observer samples due by the end of a plugin's host sleep have been answered and
committed when the sleep settles: the sleep's end grants each stream of the
execution one window of every sample due by then, including a sample due at the
same controlled instant.
Prompt arrival and cancellation use events, with no wall-clock sleeps.

## Serve Client gRPC

`services::serve(endpoint, root, host_settings, client_settings)` in
`crates/dokime-runtime/src/services/` binds a loopback endpoint and returns
`Server`; `endpoint()` exposes the chosen address and `close().await` cancels
active work, drains cleanup and closes the listener. Client settings supply
read, mutation and stream deadlines, case/page limits, message bytes and stream
buffer capacity.

The public `process::command` constructor starts owned child processes without
opening Windows console windows. On Windows, `Host::open` assigns the host process
to a kill-on-close job before any plugin starts, so every descendant, including
managed plugins the library spawns, ends when the host is killed. The desktop runs
its host in-process, so the job also owns the desktop's children. What the user
keeps leaves it: `process::open` starts the shell launcher outside the job for
files and links the desktop opens, and the updater's pre-exit hook calls
`process::detach_later_children` so the installer it starts next outlives the
exiting desktop while running plugins still end with it. Unix child termination
belongs to the library.

The `dokime-host` binary accepts `--endpoint HOST:PORT --storage PATH`.
It announces its bound HTTP URL on stdout, and stdin EOF requests graceful close.
Ordinary runs use UTC time; `--controlled-clock` is the deterministic story mode.
The storage root retains a backend identity; each process has a new boot epoch.
Before opening the evidence database, the host takes the root's exclusive
`local.lock` lease. An occupied root returns `Error::OccupiedRoot` with its path;
stop its owner or choose another root. Shutdown releases the lease once owned work
has settled, so the same process can reopen the root; a handle that remains refuses
new plugins and cycles. A host dropped without shutdown releases the lease with its
last handle and its execution workers. A stale lock file after
a crash is harmless: the operating system releases the lock, and no data is reset.
The bootstrap listener accepts loopback addresses. The library's Access.Listener.Get/Set
owns the separate authenticated TLS listener. Enable it with the revision from Get,
then use Access.Pairings.Create to obtain an endpoint, pinned certificate and bearer
credential. Pairings are independent: List returns metadata and Delete revokes one.
Pairing labels retain their existing 256-byte bound.
Access.Certificates.Create rotates live TLS trust and revokes all pairings; existing
connections survive and fresh pairings authorize them. New connections trust the new
certificate. Listener settings no longer belong to product application preferences.
Dokime implements the library CredentialStore with atomic revision checks in product
storage and private keys in the OS credential store. The library is the sole writer.
Linux host-process stories retain one isolated credential-store session across restart.
Windows stories remove their root's protected access and secret slots before deleting
temporary storage, including during failure unwinding. Explicit root disposal uses
`services::remove_credentials(root)` after stopping the host; it retains evidence
and refuses an occupied root. The host executable exposes the same operation as
`--storage PATH --remove-credentials`. Ordinary shutdown preserves credentials for restart.
Runtime stories own their Tokio runtime through `tests/support/credentials.rs::run`.
An occupied temporary root is retained until runtime teardown has cancelled its
detached work; credential removal and root disposal then run outside that runtime.
This prevents cleanup from blocking the runtime that must release the root lease.
Stories pass boxed futures through this boundary so fixture ownership does not
multiply the stack space required by large client journeys.

Plugins, Catalog, Settings, Secrets, Appearance and evidence Health
serve every declared method, including their Run transports. Plugin installation
accepts verified v1 `.dokime` tar bundles with one native sidecar per target,
portable regular-file paths, matching size/digest inventory and a compatible
`dokime.json`. Managed entries start on restart; disabled installations stay
disabled. Replacement retains that state and restores the previous entry when
starting its replacement fails. Removing an entry preserves execution evidence.
The library owns installation deadlines, startup cancellation and rollback.
At the current pin, deleting a managed entry during startup can refuse on Windows
with `Delete Plugin Installation: Access is denied. (os error 5)`: cancellation
requests child termination but does not await exit before removing its files.
The native manager must reap that child before removal; product callers do not
retry deletion or change its deadline to conceal this refusal.
A commit that wins cancellation is acknowledged with its durable result.
Remote entries retain URL identity and descriptions while unavailable, and
reload reconnects them without taking ownership of the remote process. Plugin,
catalog and health reads probe connected entries on demand, retaining stopped or
changed processes as unavailable/failed entries and excluding them from admission.

Settings use revision-checked durable preferences and report bootstrap paths
without making them editable. Plugin settings remain editable while unavailable;
`dokime.settings.read` resolves saved values on demand in the calling execution.
Secret Storage uses generation checks, protected immutable credential slots and
bounded v1 base64 JSON files with explicit replace/skip choices. Secret values
are never written into the SQLite catalog. `dokime.secrets.read` reads the named
protected credential within the calling execution's scope. Headless hosts need
an available OS credential store for secret writes and TLS key persistence.

Catalog suites, schema declarations, prompt definitions, planned cases, pending
prompts, parameters and frozen steps return their complete bounded sets in stable
order, without page controls or continuation. Executions, cycles and accumulated
evidence (records, samples, health/log lines, files and notes) remain paged.
Health paging binds the current process epoch and inventory; installation paging
still belongs to the product copy awaiting library Plugins composition. Generic status now uses the library root:

```text
Status.Get / Watch
Status.Logs.List / Watch
Status.TraceEvents.List / Watch
Status.Diagnostics.Get
```

Get includes application/library identity, stable host ID, availability and health
components. Product health changes notify the library contribution registration.
Logs and trace events are bounded process diagnostics: keep List continuation at
its issued position, and resynchronize explicitly after a Watch retention gap.
Diagnostics.Get freezes a bounded tar archive, then reads ranges by its returned
name, offset and length; verify SHA-256 after the final range. The product contributes
protocol and path entries, and retains no second collector. Secret values are
registered before publication and after reopening their protected storage; sensitive
field names also redact. Capture honors `RUST_LOG`, with INFO as the host default.

Planning, Execution, Observation, Prompts and History serve every declared
operation and its Run form. Evidence, Telemetry, Reports and Notes also serve
every direct and Run method. Content serves artifact, evidence archive and
execution-file methods; its appearance asset download remains refused. Query
ListExecutions is served; its remaining operations and Media are held for the
owner’s API decision. Generic library contracts are discoverable through CatalogEntries.List.
Refused methods return `UNIMPLEMENTED` naming the action; the owning OpenSpec
task lists them. The library gRPC manager registers all services
on both the loopback and remote listeners.
Requests using unserved filter/cursor variants are explicitly refused.
Evidence additionally serves captured-schema lookup and record watches, with
selection-bound cursors, explicit `dokime.log` reads and Run envelopes. Telemetry
serves bounded forward/tail pages, source-time windows and event-driven watches,
including Run envelopes. Plots serves ListAnalyses, QuerySnapshots and Watch with
Run envelopes. Its ten kinds reduce in storage, including row filters, scale/offset/detail
expressions, source-time joins, grouped time series/distributions, logarithmic
contours, sigma intervals, grid highlights, bounded record markers and separate
summary windows. Raw reads refuse oversized selections; LTTB selects bounded
vertices in SQLite. Snapshot batches coarsen complete buckets to the encoded
message budget. Unary plugin analyses resolve one ready provider by ID/version,
read bounded raw columns and validate typed parameters, output axes and units.
Incremental watch buckets, live deltas, higher-order/non-category plot aggregates and
derived analysis defaults remain open; watches currently emit snapshots.

Prompts serves all five operations and their bidirectional Run forms. Catalog
definitions return whole by prompt type; pending prompts return whole by ID with
cycle and execution filters. Respond validates declared actions, fields, answer rules and evidence
conditions, records the authenticated local responder and acknowledges durable
acceptance. GetPrompt retains the question and answer after settlement and
restart. `client_answers_operator_prompts` exercises this operator journey.

Planning serves preview, registration and materialization through both call
forms. Preview preserves selection order and duplicates without registering or
starting. Registration commits the complete bounded plan and its cases
atomically; the same key resolves its original reply after restart, while a
changed plan requires a new key. MaterializeTemplate registers one expanded
selection. Parameter expressions, defaults, Cartesian and zip dimensions, list
parameters and opaque file selections resolve before plugin materialization.
`client_plans_parameter_selections` proves the registration journey.

Execution serves all 28 operations and their Run forms. CreateCycle accepts
explicit origins or creates an empty mutable cycle for registered case bindings.
AppendExecutions atomically admits the supplied execution identities. Manual
cycles wait for StartExecution; automatic cycles admit their runs after append.
Interactive runs accept Next, RunStep and Finish with durable command receipts;
retrying a command ID resolves its existing receipt. Pause and stop apply at step
boundaries; cancel interrupts the active call and drains required cleanup.
Cycle/execution detail, complete ordered steps/parameters and bounded case lists expose
backend-owned capabilities. Labels and archives remain editable presentation
without altering seals. `client_controls_execution_lifecycle` proves the journey.

Observation serves runtime shells, scoped traceability, runtime invalidations and
execution progress through both call forms. Runtime watches resynchronize first,
reject another epoch or a future revision, and settle on graceful shutdown.
Execution progress uses `dokime.v1.executions.Progress.Get/Watch`; each execution
settles independently. Get returns the current whole value, revision and durable
`resolved_at`, also included in execution detail. Watch opens at the requested
returned position and follows changes to that value and the manager's phase;
its finished observation precedes successful stream completion. Unrelated evidence
does not repeat an unchanged progress value. Cancelling a watch leaves its run active.
`client_observes_runtime_state` proves these contracts.

History serves all eleven operations and their Run forms. Checkpoint inspection
returns the existing seal, persistent backend identity and exact evidence cut;
reads never publish or create a newer checkpoint. Cycle browsing retains an
append-only saved frontier. Execution browsing returns a cursor that must be
repeated with unchanged filters and order. Record and sample continuations bind
the checkpoint and table; sample pages use bounded storage reads. Artifact
downloads stream metadata, ordered chunks, completion digest and a final
response; output windows return only the requested bytes. Deletion checks the
exact seal and retains a durable request receipt for retry resolution.
`client_browses_sealed_history` proves this workflow, including restart reads.

The service adapter adds `Host::record_answer`, `changes`, `signal_change`,
`is_shutting_down`, `execution_controls`, `start_execution_mode`,
`pause_execution`, `stop_execution` and `submit_execution_command`.
The storage owners are `crates/dokime-storage/src/plans.rs` (registration and
planned-case reads), `cycles.rs` (cycle headers, execution presentation, command
receipts, recovery and evidence times), and `history.rs` (saved frontiers and
idempotent sealed deletion). `Store::create_executions` admits complete origin
lists atomically. The added tables are `cases`, `registrations`, `cycles`,
`bindings`, `commands`, `cycle_history` and `deletions`, alongside the
`administration` table. Root validation requires the current v1 layout and refuses incompatible roots with
the action to select a fresh storage root. It never migrates or deletes existing data.

Materialization returns frozen origins; CreateCycle admits and starts the
explicit origin list with `continue_automatically`, returning execution IDs.
Progress streams expose queued/prompt/finished events and end with a terminal
response followed by successful status. Cancelling a watch does not cancel the
execution. Pending prompt IDs combine execution identity and the plugin's prompt
name to keep concurrent executions distinct. Exact record and sample payloads
can be decoded using the catalog descriptors frozen in ExecutionDetail.schemas.
ExecutionDetail exposes the verified seal and cycle outcome counts; both detail
and execution listings carry backend-owned action capabilities. Listings count
evaluated, not-evaluated and declared outcomes separately, folding only evaluated
results into the verdict. Plugin listings retain unavailable descriptions after
restart and return Ready after explicit reattachment. Interrupted evidence uses
the same detail, listing, record, telemetry and progress reads.

Reports stream metadata, ordered chunks, SHA-256 completion and the terminal
response. Downloading reads an existing sealed report or renders a read-only
snapshot; neither path publishes. `Reports.PublishExecutionReport` explicitly
selects an execution and document, renders the fixed sealed snapshot, then stores
the frozen document, execution seal and independently sealed HTML. Rendering
failure publishes nothing. Subsequent downloads return the published bytes,
including after restart.

Build prerequisites and run the client stories through the focused task:

```powershell
mise run build:host-fixtures
mise -C sdk/python run prepare
mise run test -- --package dokime-runtime --target system --exact client_runs_thermal_suite_over_grpc::client_runs_thermal_suite_over_grpc
mise run test -- --package dokime-runtime --target system --exact python_client_runs_thermal_suite_over_grpc::python_client_runs_thermal_suite_over_grpc
```

The Rust story uses `dokime::client::HostClient` service conveniences over the
generated clients reexported by `sdk/rust`'s `dokime` package. The Python story uses the built pure wheel in the root `.venv/`;
`DOKIME_PYTHON` selects another explicitly prepared interpreter. Neither story
builds artifacts, polls, or sleeps on the wall clock. The Rust journey also starts
two executions in one cycle, answers their prompts differently and independently
reads their records through the client. The system target covers every host story,
including publication, interrupted recovery and every language-equivalence story.
Every story uses `SYSTEM_HANG_GUARD` from `tests/support/settings.rs`; operation
policies retain their own deadlines.

## Command-Line Tool

`crates/dokime-cli` builds the current `dokime` tool.
Hosting embeds the new runtime listener; remote commands use `sdk/rust`'s client
half. Build with `mise run build -- --package dokime-cli`.

`dokime --endpoint URL operation list --json` returns the complete bounded
admitted operation catalog, including owner, generation and contract. It has no
paging flags. `operation show ID --json` inspects the same contract, including
visibility, policy, protobuf descriptors and readable request/response field shapes.
`operation call ID --json '{}'` validates a unary Public request using its admitted
descriptors and calls its admitted gRPC mapping through the SDK generic transport,
then prints the typed JSON response. Use `--input-file request.json` for file input. Stream contracts require the typed streaming client;
refusals name the requested operation. Policies run on the host; the CLI adds no
mutation retries.

`dokime version --json` reports the compiled package version, source revision
(with `-dirty` for tracked local edits), publication channel, OS and architecture.
Local builds have a null channel; release and Nightly publishers identify theirs
from the build environment. Source archives without Git or CI revision metadata
report a null revision.

`dokime doctor storage --json` probes write, sync, rename, readback and removal
in a disposable scratch directory, then opens and closes a scratch host.
`--storage PATH` (or configured `storage`) additionally diagnoses an existing root
from a bounded copy of its SQLite files under the root lease. Stop the host first.
The original root is never opened for recovery or changed; missing roots are not
created. The copy forgets its credential catalogs before opening, so it never reads
or removes the original's native credentials. `retained_bytes` bounds the database copy. Incompatible roots produce a
failed report and backup/fresh-root or explicit-reset guidance.

Export access with `dokime --endpoint URL access pairing export --label Operator
--out operator.dokime-remote --json`. It enables the host's saved TLS listener if
needed, creates an independent Access pairing, and atomically writes a new secret
file. Existing destinations are refused before creating a pairing. Publication
failure revokes the pairing it just created; stdout contains only its ID and path.

All remote commands accept `--remote operator.dokime-remote`, or read
`DOKIME_REMOTE` when no explicit endpoint is supplied. The six-field JSON v1 format
is `version`, `endpoints`, `backend_identifier`, `label`, `ca_certificate_pem` and
`access_key`, bounded to 1 MiB. This candidate reads `.dokime-remote` files directly. Keep the file private. Endpoints are attempted in order; pinned TLS and
backend identity must both pass before any command runs. No local credential or
profile is created. Use the host's Access pairing deletion operation to revoke it.

`plugin dev` serves an owned built process, registers it through an endpoint or
pairing when supplied, and removes that entry before closing on stdin EOF or Ctrl+C.
Detecting an unexpected exit and plugin-specific `.proto` source export await the
host/API requests listed in the parity checklist; embedded product export remains
`proto export --out proto`.

```sh
dokime new first-plugin --lang rust
dokime new remote-plugin --lang python
dokime new managed-plugin --lang csharp
dokime new native-plugin --lang cpp
dokime plugin bundle --executable first-plugin/test/target/debug/dokime-example-test --out first.dokime
dokime plugin inspect --bundle first.dokime --json
dokime --endpoint 127.0.0.1:7000 server --storage ./evidence
dokime --endpoint http://127.0.0.1:7000 plugin install first.dokime
dokime --endpoint http://127.0.0.1:7000 plugin add http://127.0.0.1:7001
dokime --endpoint http://127.0.0.1:7000 catalog list --json
dokime --endpoint http://127.0.0.1:7000 plan materialize --suite SUITE --template example.capture --out cases.pb
dokime --endpoint http://127.0.0.1:7000 cycle create --cases cases.pb --json
dokime --endpoint http://127.0.0.1:7000 execution prompt next EXECUTION --json
dokime --endpoint http://127.0.0.1:7000 execution prompt answer EXECUTION --action yes --accepted
dokime --endpoint http://127.0.0.1:7000 execution watch EXECUTION
dokime --endpoint http://127.0.0.1:7000 execution report EXECUTION --out report.html
dokime proto export --out proto
```

Bundles include `dokime.json` with an exact requirement for the bundling tool's
Dokime version. Run each generated project's `mise run build` before bundling its executable.
Rust's test executable is `dokime-example-test` (`.exe` on Windows). Supply each
required runtime file with a repeated `--asset PATH`; assets are installed beside
the executable. C# must be published self-contained, with its supporting runtime
files supplied as assets, or as a single-file executable. Python
stays remote. C++ generation retrieves the versioned SDK archive's published
SHA-256 sidecar before creating the project; an unpublished candidate requires
paired `--sdk-url` and `--sdk-sha256` overrides. Existing destinations are refused.

`plugin dev --executable PATH` starts a built plugin remotely and prints its bound
URL; adding `--endpoint` attaches it to that host. Stdin EOF or Ctrl+C stops the
owned process. `plugin inspect --url URL` describes an independently running
plugin. Hosting uses the same EOF/interrupt close contract as `dokime-host`.

`--config PATH` reads explicit TOML settings; endpoint and storage may be supplied
there or on the command line. Unknown keys, zero bounds and zero deadlines fail.
`retained_bytes` also bounds CLI case files. The bootstrap host listener remains loopback; Access.Listener manages authenticated TLS access.

```toml
endpoint = "http://127.0.0.1:7000"
storage = "./evidence"
connect_timeout_s = 30
read_timeout_s = 30
mutation_timeout_s = 30
watch_timeout_s = 3600
reconnect_delay_ms = 25
interval_window_ms = 100
interval_lag_windows = 10
message_bytes = 4194304
retained_bytes = 67108864
stream_buffer = 8
max_cases = 1000
page_items = 1000
capture_roots = ["D:/bench/device"]
capture_files = 64
capture_bytes = 268435456
captures = 256
```

`interval_window_ms` is the sampling time one interval window covers
(`PlacementSettings::interval_window`, default `dokime_runtime::INTERVAL_WINDOW`,
100 ms). The host grants each interval stream one window at a time and the plugin
answers it with columns, so a longer window stores more samples per message and
batch row, and a shorter one shows live samples sooner. The number of samples in a
window follows from its length and the stream's interval; it is never set directly.
A stream overrides the host window through a plugin Duration setting named by its
observer's window setting (Rust `.window_setting("window")`, Python
`window_setting="window"`, C# `windowSetting: "window"`, C++ `.window_setting("window")`);
the plugin's settings then edit it per stream like any other plugin setting.

`interval_lag_windows` is how many lengths of a stream's window its demand may fall
behind that window's end (`PlacementSettings::interval_lag`, default
`dokime_runtime::INTERVAL_LAG`, 10). A host that cannot keep up with a stream's rate
fails the stream instead of granting ever longer windows. The execution ends
Interrupted with a `streams:failure` entry that names the stream and the action: lower
its rate or the number of streams, shorten its window, or raise
`interval_lag_windows`. A window larger than the shared plugin publication pool, and a
pool full of rows storage has not taken in, end the execution the same way. Dokime
holds no demand on a plugin stream outside an execution: scoped sampling is all it
stores, and the library grants no idle windows.

Materialized case files contain the authoritative `PreviewSelectionReply`
protobuf, preserving frozen origins. `--parameters PATH` reads a JSON object
mapping parameter names to arrays of canonical protobuf `Value` objects. For the
Python starter, supply `{"limit":[{"number":10}]}`. The host resolves the
selection and retains each registered case identity with its frozen origin.
`cycle append --request append.pb` reads an encoded
`ExecutionAppendExecutionsRequest` with the destination cycle and explicit
execution/case bindings. Cases come from materialization or registration;
automatic continuation starts appended executions, while manual cycles leave
them queued for `execution start EXECUTION`. `execution stop EXECUTION` requests
stopping at the next step boundary; answer an outstanding prompt so its current
step can settle. `cycle seal CYCLE` requires every execution to be sealed, and
`cycle rerun CYCLE` starts a new cycle from its frozen runnable origins.

The CLI system journeys exercise managed Rust installation/removal, remote Rust
and Python admission, prompt settlement, reports, append/start/stop and
seal/rerun. Build the generated starter fixture and run every story:

```sh
mise run build:cli-fixtures
mise run build:cli-fixtures -- --language python
mise run build:cli-fixtures -- --language csharp
mise run build:cli-fixtures -- --language cpp
mise run test -- --package dokime-cli --target system
```

Reports validate metadata, ordered offsets, total bytes, SHA-256 completion and
terminal status before creating the requested output; existing output is retained.
`execution export EXECUTION --out FILE.zip` and `cycle export CYCLE --out FILE.zip`
download the Evidence.Archive ZIP with the same verified download, refusing an
existing destination. `--id-file PATH` on cycle create/rerun/append and prompt next
writes confirmed UUIDs, one per line. `server --env-file PATH` (and
`dokime-host --env-file`) reads `RUST_LOG` from an explicit file; process variables
win and unknown names are refused.
Human output is default and inspection/read commands offer `--json` with proto
field names. Diagnostics go to stderr; exit codes are 0 success, 2 invalid
arguments/configuration, 3 operational failure and 4 resource-capacity refusal.

The base host refuses managed `Plugins.Install`/`Plugins.Remove`, cycle
`AppendExecutions`/`SealCycle`/`RerunCycle` and execution
`StartExecution`/`StopExecution`. Their commands send real SDK requests and report
the host's named refusal. The service owners complete these independently;
`first_hour_from_rust_bundle_to_python_remote` requires their convergence.
`remote_first_hour_runs_rust_and_python` exercises the currently served workflow.
`scaffold_languages_and_export_protocol_without_a_host` covers all four starters,
existing-destination preservation and embedded proto export.

```sh
mise run build:cli-fixtures
mise run test -- --package dokime-cli --target system
```

The fixture producer uses `dokime new` and defaults to Rust for Tests Build.
`--language python|csharp|cpp` selects another generated candidate. Build host
fixtures and each SDK's package first. Python runs the generated project's test
task against the prepared host; C# runs `test:starter` against the generated
executable; C++ runs both managed and remote SDK journeys against its installed
package consumer. Stories compare generated source bytes and never compile plugins.
Installed binary renaming and desktop integration remain on the combined restoration checklist. Installer packaging is not selected in this candidate.

## Canonical Selection Requests

`dokime plan preview --input-file preview.json --json` accepts a
PlanningPreviewSelectionRequest in protobuf JSON. `plan register --input-file
register.json --out cases.pb --json` accepts PlanningRegisterSelectionRequest,
including the caller-retained idempotency_key. Replaying the same request returns
the original registered reply; changing it under that key conflicts. `--out` saves
the protobuf reply for `cycle create --cases cases.pb`.

```json
{"plan":{"selections":[{"suite_id":"<catalog suite ID>","template_id":"example.capture"}],"max_cases":1},"idempotency_key":"my-selection"}
```

Omit idempotency_key in a preview request. `cycle report <cycle-id> --out cycle.html`
downloads a verified HTML preview and refuses to overwrite an existing output.

## Compare Every SDK

The single thermal procedure is authored in each SDK: Rust `examples/thermal.rs`,
Python `examples/thermal/suite.py`, C# `sdk/tests/fixtures/csharp/SystemPlugin` and
C++ `sdk/tests/fixtures/cpp/thermal` (the fixture paths are repository-relative).
The host compares the complete frozen origin/test case, ordered evidence and
samples (including source time, sequence, position and exact payload), outcome
and cycle counts. Only run-specific execution identity, plugin digest/start time
and receipt timestamps are excluded. Sealed evidence must reopen unchanged.
Python and C++ also decode Door from retained schemas. C# registers record-only descriptors through Harmos Message<T>(); its named
`csharp_frozen_record_schema_decodes_door` assertion now decodes Door from the
frozen catalog at the exact 26010c03 Harmos revision.

Build the shared fixtures and install the Python wheel with the tasks above.
`build:host-fixtures` invokes each compiled SDK's own tasks and exports the
fixtures under `target/build/<profile>/<target>/fixtures/host/`. C# includes its
runtime. C++ resolves its pinned native graph through vcpkg and fetches the
pinned Harmos source through CMake FetchContent. Tests never compile
fixtures. Local Tests Build includes every available compiled SDK fixture and the
full selection runs every story. Pull requests build Rust, Python and C# fixtures,
run every story except the C++ equivalence story, which runs in Nightly, and run the
C# equivalence story on Linux and Windows. Nightly
builds each language independently on Linux and Windows, caching the C++ SDK's
pinned native dependencies and installing .NET through mise, then runs every
story against exported artifacts. Suite failures are reported alongside the
publication result and do not block publication.

```sh
mise run test -- --package dokime-runtime --target system --exact python_plugin_matches_rust_thermal_evidence::python_plugin_matches_rust_thermal_evidence
mise run test -- --package dokime-runtime --target system --exact csharp_plugin_matches_rust_thermal_evidence::csharp_plugin_matches_rust_thermal_evidence
mise run test -- --package dokime-runtime --target system --exact cpp_plugin_matches_rust_thermal_evidence::cpp_plugin_matches_rust_thermal_evidence
```

Python runs remote; C# and C++ run remote and managed. Set
`DOKIME_CSHARP_THERMAL_PLUGIN` or `DOKIME_CPP_THERMAL_PLUGIN` for a prepared
executable outside the exported fixture path. `DOKIME_PYTHON` selects the
interpreter with the built wheel installed.

## Bounded Client Reads

`Store::metadata`, `evidence_frontier`, `execution_evidence_frontier`,
`cycle_evidence_frontier`, `evidence_entries` and `subscribe` supply
metadata and committed evidence pages without decoding streams. Evidence cursors
bind the storage-root backend identity and exact selection; sealed scoped pages survive restart unchanged. Structured records remain the default;
select `dokime.log` explicitly for retained execution logs. Captured schema lookup
uses frozen descriptors even when the producing plugin is disconnected.

`Store::sample_batch` commits Arrow batches, exact protobuf row indexes and numeric
field indexes together. `stream_names` and `sample_window` read bounded pages;
`numeric_bounds`, `numeric_bins`, `time_series_window`, `numeric_statistics`,
`histogram_window`, `box_window`, `cartesian_window`, `category_window`,
`contour_window`, `grid_window` and `spectrum_window` own plot reductions.
Time-series windows examine each matching indexed row once and retain only the
requested buckets. The reply states `examined_rows` and `resolution_nanos`.
Frequency transforms select the latest declared `input_samples` within the source
window; omission uses the configured transform input bound. An explicit count
outside that bound refuses, while long history remains readable. Grid previews
retain the newest complete frames fitting their axes and output resolution.
Snapshot batches use retained pages within the configured message limit. The
desktop groups visible plots into bounded requests; 32 is its per-request bound,
with no document-size ceiling. Identical selections within a batch share work and
frozen schema preparation. Aligned time-series replies use a packed shared time
axis. Watch refreshes when selected saved evidence advances. Document reads return
plot definitions, with optional frame-count and byte-bounded pages; plot queries
perform their own validation when requested.

`client_browses_execution_evidence`, `client_reads_bounded_telemetry`,
`client_queries_plot_kinds` and `client_queries_plot_windows` are focused Rust SDK
journeys against the real host process. The plot-window fixture retains 2,000,003
samples and requests forty plots. Each selected window consumes 1,000 source rows,
for a counted 40,000 rows per snapshot batch; zoom requests report their finer
resolution. These assertions measure work rather than elapsed time.

## Evidence and Recovery

Each execution freezes the complete PluginRun origin and catalog descriptors.
The manager owns live queued, initializing, stepping, evaluating and cleaning-up
state; those phases are not persisted as a Run. Step boundaries, acknowledged
records, logs, progress and criterion reads are durable evidence. Sample batches
retain source time, host receipt time, publication sequence and a durable position
shared with evidence entries. Interval streams use source UTC nanoseconds as
their axis. Step queries use the evidence positions at the step's time boundaries,
including equal-time events without admitting evidence from another step.

SQLite commits each Arrow IPC batch and its position atomically with FULL
synchronous durability. Streams close before evidence closes, and only after the
run's plugin binding has drained every accepted sample. A run that panics or is
dropped hands its binding to a host task that drains it before releasing the
run's sample scope; the failed worker's settlement waits for that release before
it closes and seals the evidence. When a run fails and its binding close fails
too, the run's error is reported first with the close error attached. A seal hashes the
frozen execution and ordered evidence bytes, and reopened reads verify it.
Writes after close are refused. Open plugin-run executions are recovered as
interrupted and sealed; import origins are never run-recovery targets.

Storage remains at v1. An unrecognized nonempty root is refused with instructions
to back it up and choose fresh storage or explicitly reset it. No migration,
implicit reset or deletion occurs.

## Content and Notes

The library transfer replacement is blocked by its missing product content-source
registration port, recorded in Operation Grammar Library Requests. The current
content journeys below still exercise the product implementation. HostValues also
waits for library-backed plugin admission; settings and secret value delivery keep
their current invocation-scoped handlers until that replacement is available.


Content serves ordered artifact transfers, execution/cycle evidence exports and
registered execution-file inventory, bounded windows, downloads and follows.
Client uploads require an explicit SHA-256 completion before input finish;
cancelling a staged upload publishes nothing. Plugin artifact calls are unary
and preserve sequence, exact retry, prefix replacement and terminal abort.
File names select retained content identities, never arbitrary filesystem paths.
A resumed follow requires the original identity and committed offset.
Evidence downloads are ZIP archives. `evidence.pb` contains the v1 protobuf
`ExecutionArchive` defined in `crates/dokime-storage/src/execution.rs`, including
product artifact metadata and its original native references in field 9.
`telemetry/<stream>.arrow` retains each finalized Arrow file unchanged.
Native file histories are selected through the registered backend at a retained
snapshot; `files/<reference-digest>` contains the unchanged bytes and the adjacent
`.pb` member retains the original file reference, including its history.
`manifest.json` records member paths, lengths and SHA-256 digests. Cycle ZIPs
contain `executions/<id>.zip` members and their own manifest. The final archive
is published through native uploads and read through `Files.Get/Read/Watch`.
The shape stays at v1 under the development storage policy. Archive import and
the older execution-file presentation fixtures remain open consumer work.
Notes support create/edit/list and Run envelopes, durable idempotent creation,
selection-bound paging and closed-owner refusal. The loopback listener uses one
local client principal; the separate Access listener authenticates paired callers.

Build the content-story fixture before its client journey:

```sh
mise run build -- --cargo build --package dokime-runtime --example content_fixture
mise run test -- --package dokime-runtime --target system --exact client_transfers_execution_content::client_transfers_execution_content
mise run test -- --package dokime-runtime --target system --exact client_keeps_execution_notes::client_keeps_execution_notes
```

Appearance asset downloads and Media capture return named refusals pending their
owning API/storage decisions. Query remains at its previously served surface.

## Document Snapshots and Reports

Documents serves capture and plot-preference updates with direct and Run calls.
Captures stream structure, complete authored table rows, summary tiles and plot
bindings. Each plot request carries an execution-qualified fixed storage cursor;
the Plots service evaluates it with the same bounded reducers used by HTML.
`Host::current_document` reads the frozen catalog and stored preferences;
`Host::snapshot_cut` renders an explicit evidence cut. Storage adds
`read_session`, `plot_selection`, `plot_group_values`, `sigma_window`,
`contour_scaled_window`, `document_value` and plot-preference reads/writes.

Reports serves execution/cycle downloads, materialization and explicit execution
publication, including every Run envelope. Materialization returns a temporary
file reference; download its `metadata.id` with Content.DownloadArtifact.
The temporary output is excluded from artifact inventory and expires on restart
or when the bounded cache evicts it. Cycle downloads use identifier `cycle`.
Publication freezes the selected document, execution seal and independent HTML;
changing plot preferences cannot alter already published HTML. Report verification
hashes the retained document bytes rather than re-encoding decoded maps.

Repeated sections use bounded distinct-value pages and propagate selection
filters to summaries and plots. Continuations bind the backend epoch, document,
selection and evidence frontier. Cycle capture serves the default records,
artifacts and media document. Summaries support record/stream scalars, nested
aggregates, exact percentiles and scalar transforms. HTML contour cells and bands
preserve nonuniform edges and validity masks. Evidence-card filter evaluation,
cross-execution aggregate expressions, artifact expressions and complete
trajectory/grid playback remain in the owning task inventory.

The focused journeys are `client_renders_fixed_documents` and
`client_queries_advanced_plots` in the runtime’s system target. The protocol crate’s
generation task owns the additive `PlotRequest.at` field, unary
`AnalysisRequest`/`AnalysisSamples`/`AnalysisResponse` contracts and generated output.
An analysis operation uses its metadata identifier, accepts one bounded raw
series plus typed parameters and returns its declared `AnalysisValues`.

### Host and Storage Additions

| Owner | Added Methods |
| --- | --- |
| Host | `current_document`, `snapshot_cut`, `analyze`; service adapters also provide `render_report`, `retain_report` and `temporary_report` |
| Storage Content | `publish_artifact_chunk`, `begin_artifact`, `abort_artifact`, `append_content`, `finish_artifact`, `register_execution_file`, `finish_execution_file`, `content`, `execution_file`, `content_window`, `artifacts_page`, `files_page`, `write_execution_archive`, `content_fingerprint` |
| Storage Notes | `create_note`, `edit_note`, `note_owner`, `notes_page`, `notes_fingerprint` |
| Storage Documents | `plot_preferences`, `save_plot_preferences`, `document_value`, `document_value_filtered`, `document_distinct` |
| Storage Queries | `read_session`, `cycle_execution_page`, `cycle_is_sealed`, `plot_selection`, `plot_group_values`, `select_plot_group`, `sigma_window`, `contour_scaled_window`, `sampled_time_series_window` |

## Desktop Integration

The desktop is a Cargo workspace and default member on the execution host.
Its native connection owner uses the product SDK for embedded and remote hosts;
Product builds, fixtures, native stories and both WebView lanes include it.

## Final Library Definition Contract

The definitions pin Harmos at `82e394dac656bf6b28a31f7429360d84f1701051`.

```text
proto/dokime/
  model/v1/   dokime.model.v1   exchanged values; no operations
  host/v1/    dokime.host.v1    host executor; callers on every method
  plugin/v1/  dokime.plugin.v1  plugin executor; called by its host
```

Every resource owns one file, service and its operation messages in service order.
Nested resources use subfolders. Caller options are mandatory only on host
operations and independent of visibility, grants and inherited scope. Plugin
executor methods have no callers option. Only the host invokes their Protected
bindings through Instance; they never become application client/context resources.

### Before and After Inventory

| Product Definitions | Before | After |
| --- | ---: | ---: |
| Proto files | 76 | 58 |
| Declared resources (services) | 38 | 43 |
| Declared operations (RPCs) | 123 | 124 |
| Host resources / operations | split over two boundaries | 38 / 118 |
| Plugin resources / operations | undeclared payload pairs | 5 / 6 |
| Model files / operations | 21 / 0 | 15 / 0 |

The before count includes duplicate Files/Uploads RPCs and the retired product
Plugins wrappers, and excludes bare context/lifecycle payload pairs. The restored Files.Capture declaration adds one RPC; the inventory otherwise
preserves the agreed capabilities. The prior boundary inventory
identified Files and files.Uploads as identical declarations; they merge once with
ALL. Backend, Documents, Evidence, Records, Traceability, Progress and Prompts have
complementary client/context operations. Artifacts.Create, Statistics, Records.List,
Traceability.Get, Progress.Get and Secrets.Get use one richer shape. None needs a
caller-specific operation name. Media capture remains declared and unimplemented;
FTP capture and the effects journal are not restored; local file capture is.

### Harmonized Request and Response Shapes

| Operation (ALL) | Unified Request | Unified Response and Admission |
| --- | --- | --- |
| content.Artifacts.Create | file, optional scope, filename, content_type, request_id, metadata | ArtifactMetadata and receipt; plugin scope defaults to its execution and cannot escape it. content_type replaces media_type. |
| Evidence.Statistics | optional scope, stream, versions, between, outputs, field, step, optional at | Complete Statistics and resolved_at; plugins use their own execution. |
| evidence.Records.List | optional scope, record_type, versions, between, step, selection, optional at/page | records, schema, count, last, resolved_at, next_page_token; CapturedRecord retains payload, source timestamp and sequence. |
| evidence.Traceability.Get | optional scope, page, at, entry | snapshots and page metadata, plus selected entry value/revision; scope selects the execution, entry selects the independently revisioned entry. Foreign selectors refuse. |
| executions.Progress.Get | optional name, optional at | Complete value, revision and resolved_at; clients give name, plugins default to and are restricted to their execution. |
| Secrets.Get | name, optional at | secret bytes, optional administrative value/revision and resolved_at; plugins read only declared names, each the identifier of a host-wide secret, and receive no administrative metadata. |

Message presence makes omitted selectors explicit. Client generators require scope
or execution name where the operation needs it; plugin generators permit omission.
A provided selector is checked against trusted caller identity before any read or
write, never accepted as proof of that identity. Secret bytes are authoritative;
the existing text writable value is administrative input/output only.

### Exact Values and Document States

The merged dev contract retains field names and numbers in the new homes:
`dokime.model.v1.Value.exact = 10`; `dokime.host.v1.Statistics` uses Value for
min/max/mean/median/p05/p95/p99/first/last and adds `sum = 13`.
The former plugin EvidenceStatisticsResponse now uses that same Statistics payload
inside the harmonized host response, with its read cut owned by the response.
PlotBoxSummary min/q1/median/q3/max, PlotHistogramStatistics mean/median/min/max
and PlotSeriesSummary min/max/mean/median are Value messages with presence.
PlotColumn adds exact `totals = 6` separately from floating drawing coordinates.
Standard deviation, RMS, whiskers, outliers and drawing axes remain floating point.

The final library has no storage wire model. The product model therefore owns
ExactNumber in `model/v1/scalar.proto`, preserving the former library payload's
`negative = 1`, `numerator = 2`, `denominator = 3`, minimal base-10 strings,
reduced fractions, positive nonzero denominators and unsigned zero.
Consumers map it losslessly to native query values; no storage wire API returns.

Analyses.Compute consumes bounded raw samples with one source-time coordinate per
value, using aligned frame time when present and normalizing the captured origin.
The shape remains `dokime.plugin.v1.AnalysisSamples`; the correction is in the
moved analysis handler. Documents retains pending/result bindings, summary states
and CardComplete under `dokime.host.v1`; literal table rows settle at completion.
Desktop group loading retains bounded consecutive kind groups, independent visible
requests, deferred lookahead and generation fencing. These behavioral fixes were
carried by the merge and are not consumer work to reimplement.

### Operation Identity and Caller Mapping

The first column is the pre-pin wire identity or hand-bound context identity.
Library composition rows name the library operation directly, without forwarding.
The complete library identity/type mapping is in the pinned
[Harmos Architecture](https://github.com/stokker-technologies/harmos/blob/82e394dac656bf6b28a31f7429360d84f1701051/docs/architecture.md).

| Old Identity | Final Identity | Callers | Request | Response |
| --- | --- | --- | --- | --- |
| `AnalysisRequest/AnalysisResponse` | `dokime.plugin.v1.Analyses/Compute` | Host Only (Protected; No Callers Option) | `dokime.plugin.v1.AnalysesComputeRequest` | `dokime.plugin.v1.AnalysesComputeResponse` |
| `dokime.client.v1.Aggregates/Get` | `dokime.host.v1.Aggregates/Get` | CLIENTS | `dokime.host.v1.AggregatesGetRequest` | `dokime.host.v1.AggregatesGetResponse` |
| `dokime.client.v1.Analyses/List` | `dokime.host.v1.Analyses/List` | CLIENTS | `dokime.host.v1.AnalysesListRequest` | `dokime.host.v1.AnalysesListResponse` |
| `dokime.client.v1.Backend/Appearance` | `dokime.host.v1.Backend/Appearance` | CLIENTS | `dokime.host.v1.BackendAppearanceRequest` | `dokime.host.v1.BackendAppearanceResponse` |
| `dokime.client.v1.Cases/Create` | `dokime.host.v1.Cases/Create` | CLIENTS | `dokime.host.v1.CasesCreateRequest` | `dokime.host.v1.CasesCreateResponse` |
| `dokime.client.v1.Cases/Delete` | `dokime.host.v1.Cases/Delete` | CLIENTS | `dokime.host.v1.CasesDeleteRequest` | `dokime.host.v1.CasesDeleteResponse` |
| `dokime.client.v1.Cases/Get` | `dokime.host.v1.Cases/Get` | CLIENTS | `dokime.host.v1.CasesGetRequest` | `dokime.host.v1.CasesGetResponse` |
| `dokime.client.v1.Cases/List` | `dokime.host.v1.Cases/List` | CLIENTS | `dokime.host.v1.CasesListRequest` | `dokime.host.v1.CasesListResponse` |
| `dokime.client.v1.Cases/Parameters` | `dokime.host.v1.Cases/Parameters` | CLIENTS | `dokime.host.v1.CasesParametersRequest` | `dokime.host.v1.CasesParametersResponse` |
| `dokime.client.v1.Cases/Watch` | `dokime.host.v1.Cases/Watch` | CLIENTS | `dokime.host.v1.CasesWatchRequest` | `dokime.host.v1.CasesWatchResponse` |
| `dokime.client.v1.Cycles/Create` | `dokime.host.v1.Cycles/Create` | CLIENTS | `dokime.host.v1.CyclesCreateRequest` | `dokime.host.v1.CyclesCreateResponse` |
| `dokime.client.v1.Cycles/Get` | `dokime.host.v1.Cycles/Get` | CLIENTS | `dokime.host.v1.CyclesGetRequest` | `dokime.host.v1.CyclesGetResponse` |
| `dokime.client.v1.Cycles/List` | `dokime.host.v1.Cycles/List` | CLIENTS | `dokime.host.v1.CyclesListRequest` | `dokime.host.v1.CyclesListResponse` |
| `dokime.client.v1.Cycles/Seal` | `dokime.host.v1.Cycles/Seal` | CLIENTS | `dokime.host.v1.CyclesSealRequest` | `dokime.host.v1.CyclesSealResponse` |
| `dokime.client.v1.Cycles/Set` | `dokime.host.v1.Cycles/Set` | CLIENTS | `dokime.host.v1.CyclesSetRequest` | `dokime.host.v1.CyclesSetResponse` |
| `dokime.client.v1.Cycles/Watch` | `dokime.host.v1.Cycles/Watch` | CLIENTS | `dokime.host.v1.CyclesWatchRequest` | `dokime.host.v1.CyclesWatchResponse` |
| `dokime.client.v1.Documents/Get` | `dokime.host.v1.Documents/Get` | CLIENTS | `dokime.host.v1.DocumentsGetRequest` | `dokime.host.v1.DocumentsGetResponse` |
| `dokime.client.v1.Documents/Watch` | `dokime.host.v1.Documents/Watch` | CLIENTS | `dokime.host.v1.DocumentsWatchRequest` | `dokime.host.v1.DocumentsWatchResponse` |
| `dokime.client.v1.Evidence/Archive` | `dokime.host.v1.Evidence/Archive` | CLIENTS | `dokime.host.v1.EvidenceArchiveRequest` | `dokime.host.v1.EvidenceArchiveResponse` |
| `dokime.client.v1.Evidence/Statistics` | `dokime.host.v1.Evidence/Statistics` | ALL | `dokime.host.v1.EvidenceStatisticsRequest` | `dokime.host.v1.EvidenceStatisticsResponse` |
| `dokime.client.v1.Executions/Cancel` | `dokime.host.v1.Executions/Cancel` | CLIENTS | `dokime.host.v1.ExecutionsCancelRequest` | `dokime.host.v1.ExecutionsCancelResponse` |
| `dokime.client.v1.Executions/Create` | `dokime.host.v1.Executions/Create` | CLIENTS | `dokime.host.v1.ExecutionsCreateRequest` | `dokime.host.v1.ExecutionsCreateResponse` |
| `dokime.client.v1.Executions/Delete` | `dokime.host.v1.Executions/Delete` | CLIENTS | `dokime.host.v1.ExecutionsDeleteRequest` | `dokime.host.v1.ExecutionsDeleteResponse` |
| `dokime.client.v1.Executions/Get` | `dokime.host.v1.Executions/Get` | CLIENTS | `dokime.host.v1.ExecutionsGetRequest` | `dokime.host.v1.ExecutionsGetResponse` |
| `dokime.client.v1.Executions/List` | `dokime.host.v1.Executions/List` | CLIENTS | `dokime.host.v1.ExecutionsListRequest` | `dokime.host.v1.ExecutionsListResponse` |
| `dokime.client.v1.Executions/Parameters` | `dokime.host.v1.Executions/Parameters` | CLIENTS | `dokime.host.v1.ExecutionsParametersRequest` | `dokime.host.v1.ExecutionsParametersResponse` |
| `dokime.client.v1.Executions/Set` | `dokime.host.v1.Executions/Set` | CLIENTS | `dokime.host.v1.ExecutionsSetRequest` | `dokime.host.v1.ExecutionsSetResponse` |
| `dokime.client.v1.Executions/Start` | `dokime.host.v1.Executions/Start` | CLIENTS | `dokime.host.v1.ExecutionsStartRequest` | `dokime.host.v1.ExecutionsStartResponse` |
| `dokime.client.v1.Executions/Steps` | `dokime.host.v1.Executions/Steps` | CLIENTS | `dokime.host.v1.ExecutionsStepsRequest` | `dokime.host.v1.ExecutionsStepsResponse` |
| `dokime.client.v1.Executions/Stop` | `dokime.host.v1.Executions/Stop` | CLIENTS | `dokime.host.v1.ExecutionsStopRequest` | `dokime.host.v1.ExecutionsStopResponse` |
| `dokime.client.v1.Executions/Watch` | `dokime.host.v1.Executions/Watch` | CLIENTS | `dokime.host.v1.ExecutionsWatchRequest` | `dokime.host.v1.ExecutionsWatchResponse` |
| `dokime.client.v1.Files/Delete` | `dokime.host.v1.Files/Delete` | ALL | `dokime.host.v1.FilesDeleteRequest` | `dokime.host.v1.FilesDeleteResponse` |
| `dokime.client.v1.Files/Get` | `dokime.host.v1.Files/Get` | ALL | `dokime.host.v1.FilesGetRequest` | `dokime.host.v1.FilesGetResponse` |
| `dokime.client.v1.Files/Read` | `dokime.host.v1.Files/Read` | ALL | `dokime.host.v1.FilesReadRequest` | `dokime.host.v1.FilesReadResponse` |
| `dokime.client.v1.Files/Watch` | `dokime.host.v1.Files/Watch` | ALL | `dokime.host.v1.FilesWatchRequest` (target: file or upload) | `dokime.host.v1.FilesWatchResponse` |
| `dokime.client.v1.Plugins/Add` | `harmos.host.v1.Plugins/Create` | CLIENTS | `harmos.host.v1.PluginsCreateRequest` | `harmos.host.v1.PluginsCreateResponse` |
| `dokime.client.v1.Plugins/GetInstallation` | `harmos.host.v1.Plugins/Get` | CLIENTS | `harmos.host.v1.PluginsGetRequest` | `harmos.host.v1.PluginsGetResponse` |
| `dokime.client.v1.Plugins/Install` | `harmos.host.v1.Plugins/Create` | CLIENTS | `harmos.host.v1.PluginsCreateRequest` | `harmos.host.v1.PluginsCreateResponse` |
| `dokime.client.v1.Plugins/List` | `harmos.host.v1.Plugins/List` | CLIENTS | `harmos.host.v1.PluginsListRequest` | `harmos.host.v1.PluginsListResponse` |
| `dokime.client.v1.Plugins/ListAdmittedPlugins` | `harmos.host.v1.Plugins/List` | CLIENTS | `harmos.host.v1.PluginsListRequest` | `harmos.host.v1.PluginsListResponse` |
| `dokime.client.v1.Plugins/ListInstallations` | `harmos.host.v1.Plugins/List` | CLIENTS | `harmos.host.v1.PluginsListRequest` | `harmos.host.v1.PluginsListResponse` |
| `dokime.client.v1.Plugins/Reload` | `harmos.host.v1.Plugins/Restart` | CLIENTS | `harmos.host.v1.PluginsRestartRequest` | `harmos.host.v1.PluginsRestartResponse` |
| `dokime.client.v1.Plugins/Remove` | `harmos.host.v1.Plugins/Delete` | CLIENTS | `harmos.host.v1.PluginsDeleteRequest` | `harmos.host.v1.PluginsDeleteResponse` |
| `dokime.client.v1.Plugins/SetEnabled` | `harmos.host.v1.Plugins/Set` | CLIENTS | `harmos.host.v1.PluginsSetRequest` | `harmos.host.v1.PluginsSetResponse` |
| `dokime.client.v1.PromptDefinitions/Get` | `dokime.host.v1.PromptDefinitions/Get` | CLIENTS | `dokime.host.v1.PromptDefinitionsGetRequest` | `dokime.host.v1.PromptDefinitionsGetResponse` |
| `dokime.client.v1.PromptDefinitions/List` | `dokime.host.v1.PromptDefinitions/List` | CLIENTS | `dokime.host.v1.PromptDefinitionsListRequest` | `dokime.host.v1.PromptDefinitionsListResponse` |
| `dokime.client.v1.Prompts/Get` | `dokime.host.v1.Prompts/Get` | CLIENTS | `dokime.host.v1.PromptsGetRequest` | `dokime.host.v1.PromptsGetResponse` |
| `dokime.client.v1.Prompts/List` | `dokime.host.v1.Prompts/List` | CLIENTS | `dokime.host.v1.PromptsListRequest` | `dokime.host.v1.PromptsListResponse` |
| `dokime.client.v1.Prompts/Watch` | `dokime.host.v1.Prompts/Watch` | CLIENTS | `dokime.host.v1.PromptsWatchRequest` | `dokime.host.v1.PromptsWatchResponse` |
| `dokime.client.v1.RecordSchemas/Get` | `dokime.host.v1.RecordSchemas/Get` | CLIENTS | `dokime.host.v1.RecordSchemasGetRequest` | `dokime.host.v1.RecordSchemasGetResponse` |
| `dokime.client.v1.RecordSchemas/List` | `dokime.host.v1.RecordSchemas/List` | CLIENTS | `dokime.host.v1.RecordSchemasListRequest` | `dokime.host.v1.RecordSchemasListResponse` |
| `dokime.client.v1.Reports/Create` | `dokime.host.v1.Reports/Create` | CLIENTS | `dokime.host.v1.ReportsCreateRequest` | `dokime.host.v1.ReportsCreateResponse` |
| `dokime.client.v1.Reports/Get` | `dokime.host.v1.Reports/Get` | CLIENTS | `dokime.host.v1.ReportsGetRequest` | `dokime.host.v1.ReportsGetResponse` |
| `dokime.client.v1.Reports/Publish` | `dokime.host.v1.Reports/Publish` | CLIENTS | `dokime.host.v1.ReportsPublishRequest` | `dokime.host.v1.ReportsPublishResponse` |
| `dokime.client.v1.Secrets/Delete` | `dokime.host.v1.Secrets/Delete` | CLIENTS | `dokime.host.v1.SecretsDeleteRequest` | `dokime.host.v1.SecretsDeleteResponse` |
| `dokime.client.v1.Secrets/Export` | `dokime.host.v1.Secrets/Export` | CLIENTS | `dokime.host.v1.SecretsExportRequest` | `dokime.host.v1.SecretsExportResponse` |
| `dokime.client.v1.Secrets/Get` | `dokime.host.v1.Secrets/Get` | ALL | `dokime.host.v1.SecretsGetRequest` | `dokime.host.v1.SecretsGetResponse` |
| `dokime.client.v1.Secrets/Import` | `dokime.host.v1.Secrets/Import` | CLIENTS | `dokime.host.v1.SecretsImportRequest` | `dokime.host.v1.SecretsImportResponse` |
| `dokime.client.v1.Secrets/List` | `dokime.host.v1.Secrets/List` | CLIENTS | `dokime.host.v1.SecretsListRequest` | `dokime.host.v1.SecretsListResponse` |
| `dokime.client.v1.Secrets/Set` | `dokime.host.v1.Secrets/Set` | CLIENTS | `dokime.host.v1.SecretsSetRequest` | `dokime.host.v1.SecretsSetResponse` |
| `dokime.client.v1.Secrets/Watch` | `dokime.host.v1.Secrets/Watch` | CLIENTS | `dokime.host.v1.SecretsWatchRequest` | `dokime.host.v1.SecretsWatchResponse` |
| `dokime.client.v1.Settings/Get` | `dokime.host.v1.Settings/Get` | CLIENTS | `dokime.host.v1.SettingsGetRequest` | `dokime.host.v1.SettingsGetResponse` |
| `dokime.client.v1.Settings/Schema` | `dokime.host.v1.Settings/Schema` | CLIENTS | `dokime.host.v1.SettingsSchemaRequest` | `dokime.host.v1.SettingsSchemaResponse` |
| `dokime.client.v1.Settings/Set` | `dokime.host.v1.Settings/Set` | CLIENTS | `dokime.host.v1.SettingsSetRequest` | `dokime.host.v1.SettingsSetResponse` |
| `dokime.client.v1.StreamSchemas/Get` | `dokime.host.v1.StreamSchemas/Get` | CLIENTS | `dokime.host.v1.StreamSchemasGetRequest` | `dokime.host.v1.StreamSchemasGetResponse` |
| `dokime.client.v1.StreamSchemas/List` | `dokime.host.v1.StreamSchemas/List` | CLIENTS | `dokime.host.v1.StreamSchemasListRequest` | `dokime.host.v1.StreamSchemasListResponse` |
| `dokime.client.v1.Suites/Get` | `dokime.host.v1.Suites/Get` | CLIENTS | `dokime.host.v1.SuitesGetRequest` | `dokime.host.v1.SuitesGetResponse` |
| `dokime.client.v1.Suites/List` | `dokime.host.v1.Suites/List` | CLIENTS | `dokime.host.v1.SuitesListRequest` | `dokime.host.v1.SuitesListResponse` |
| `dokime.client.v1.backend.Overview/Get` | `dokime.host.v1.backend.Overview/Get` | CLIENTS | `dokime.host.v1.backend.OverviewGetRequest` | `dokime.host.v1.backend.OverviewGetResponse` |
| `dokime.client.v1.backend.Overview/Watch` | `dokime.host.v1.backend.Overview/Watch` | CLIENTS | `dokime.host.v1.backend.OverviewWatchRequest` | `dokime.host.v1.backend.OverviewWatchResponse` |
| `dokime.client.v1.content.Artifacts/Create` | `dokime.host.v1.content.Artifacts/Create` | ALL | `dokime.host.v1.content.ArtifactsCreateRequest` | `dokime.host.v1.content.ArtifactsCreateResponse` |
| `dokime.client.v1.content.Artifacts/Get` | `dokime.host.v1.content.Artifacts/Get` | CLIENTS | `dokime.host.v1.content.ArtifactsGetRequest` | `dokime.host.v1.content.ArtifactsGetResponse` |
| `dokime.client.v1.content.Artifacts/List` | `dokime.host.v1.content.Artifacts/List` | CLIENTS | `dokime.host.v1.content.ArtifactsListRequest` | `dokime.host.v1.content.ArtifactsListResponse` |
| `dokime.client.v1.content.Artifacts/Watch` | `dokime.host.v1.content.Artifacts/Watch` | CLIENTS | `dokime.host.v1.content.ArtifactsWatchRequest` | `dokime.host.v1.content.ArtifactsWatchResponse` |
| `dokime.client.v1.content.ExecutionFiles/Get` | `dokime.host.v1.content.ExecutionFiles/Get` | CLIENTS | `dokime.host.v1.content.ExecutionFilesGetRequest` | `dokime.host.v1.content.ExecutionFilesGetResponse` |
| `dokime.client.v1.content.ExecutionFiles/List` | `dokime.host.v1.content.ExecutionFiles/List` | CLIENTS | `dokime.host.v1.content.ExecutionFilesListRequest` | `dokime.host.v1.content.ExecutionFilesListResponse` |
| `dokime.client.v1.content.ExecutionFiles/Watch` | `dokime.host.v1.content.ExecutionFiles/Watch` | CLIENTS | `dokime.host.v1.content.ExecutionFilesWatchRequest` | `dokime.host.v1.content.ExecutionFilesWatchResponse` |
| `dokime.client.v1.evidence.HealthEvents/List` | `dokime.host.v1.evidence.HealthEvents/List` | CLIENTS | `dokime.host.v1.evidence.HealthEventsListRequest` | `dokime.host.v1.evidence.HealthEventsListResponse` |
| `dokime.client.v1.evidence.HealthEvents/Watch` | `dokime.host.v1.evidence.HealthEvents/Watch` | CLIENTS | `dokime.host.v1.evidence.HealthEventsWatchRequest` | `dokime.host.v1.evidence.HealthEventsWatchResponse` |
| `dokime.client.v1.evidence.Notes/Create` | `dokime.host.v1.evidence.Notes/Create` | CLIENTS | `dokime.host.v1.evidence.NotesCreateRequest` | `dokime.host.v1.evidence.NotesCreateResponse` |
| `dokime.client.v1.evidence.Notes/Get` | `dokime.host.v1.evidence.Notes/Get` | CLIENTS | `dokime.host.v1.evidence.NotesGetRequest` | `dokime.host.v1.evidence.NotesGetResponse` |
| `dokime.client.v1.evidence.Notes/List` | `dokime.host.v1.evidence.Notes/List` | CLIENTS | `dokime.host.v1.evidence.NotesListRequest` | `dokime.host.v1.evidence.NotesListResponse` |
| `dokime.client.v1.evidence.Notes/Set` | `dokime.host.v1.evidence.Notes/Set` | CLIENTS | `dokime.host.v1.evidence.NotesSetRequest` | `dokime.host.v1.evidence.NotesSetResponse` |
| `dokime.client.v1.evidence.Notes/Watch` | `dokime.host.v1.evidence.Notes/Watch` | CLIENTS | `dokime.host.v1.evidence.NotesWatchRequest` | `dokime.host.v1.evidence.NotesWatchResponse` |
| `dokime.client.v1.evidence.Records/List` | `dokime.host.v1.evidence.Records/List` | ALL | `dokime.host.v1.evidence.RecordsListRequest` | `dokime.host.v1.evidence.RecordsListResponse` |
| `dokime.client.v1.evidence.Records/Watch` | `dokime.host.v1.evidence.Records/Watch` | CLIENTS | `dokime.host.v1.evidence.RecordsWatchRequest` | `dokime.host.v1.evidence.RecordsWatchResponse` |
| `dokime.client.v1.evidence.Samples/List` | `dokime.host.v1.evidence.Samples/List` | CLIENTS | `dokime.host.v1.evidence.SamplesListRequest` | `dokime.host.v1.evidence.SamplesListResponse` |
| `dokime.client.v1.evidence.Samples/Watch` | `dokime.host.v1.evidence.Samples/Watch` | CLIENTS | `dokime.host.v1.evidence.SamplesWatchRequest` | `dokime.host.v1.evidence.SamplesWatchResponse` |
| `dokime.client.v1.evidence.Traceability/Get` | `dokime.host.v1.evidence.Traceability/Get` | ALL | `dokime.host.v1.evidence.TraceabilityGetRequest` | `dokime.host.v1.evidence.TraceabilityGetResponse` |
| `dokime.client.v1.evidence.Traceability/Watch` | `dokime.host.v1.evidence.Traceability/Watch` | CLIENTS | `dokime.host.v1.evidence.TraceabilityWatchRequest` | `dokime.host.v1.evidence.TraceabilityWatchResponse` |
| `dokime.client.v1.executions.Commands/Create` | `dokime.host.v1.executions.Commands/Create` | CLIENTS | `dokime.host.v1.executions.CommandsCreateRequest` | `dokime.host.v1.executions.CommandsCreateResponse` |
| `dokime.client.v1.executions.Commands/Get` | `dokime.host.v1.executions.Commands/Get` | CLIENTS | `dokime.host.v1.executions.CommandsGetRequest` | `dokime.host.v1.executions.CommandsGetResponse` |
| `dokime.client.v1.executions.Progress/Get` | `dokime.host.v1.executions.Progress/Get` | ALL | `dokime.host.v1.executions.ProgressGetRequest` | `dokime.host.v1.executions.ProgressGetResponse` |
| `dokime.client.v1.executions.Progress/Watch` | `dokime.host.v1.executions.Progress/Watch` | CLIENTS | `dokime.host.v1.executions.ProgressWatchRequest` | `dokime.host.v1.executions.ProgressWatchResponse` |
| `dokime.client.v1.executions.Schemas/Get` | `dokime.host.v1.executions.Schemas/Get` | CLIENTS | `dokime.host.v1.executions.SchemasGetRequest` | `dokime.host.v1.executions.SchemasGetResponse` |
| `dokime.client.v1.files.Uploads/Append` | `dokime.host.v1.files.Uploads/Append` | ALL | `dokime.host.v1.files.UploadsAppendRequest` | `dokime.host.v1.files.UploadsAppendResponse` |
| `dokime.client.v1.files.Uploads/Create` | `dokime.host.v1.files.Uploads/Create` | ALL | `dokime.host.v1.files.UploadsCreateRequest` | `dokime.host.v1.files.UploadsCreateResponse` |
| `dokime.client.v1.files.Uploads/Delete` | `dokime.host.v1.files.Uploads/Delete` | ALL | `dokime.host.v1.files.UploadsDeleteRequest` | `dokime.host.v1.files.UploadsDeleteResponse` |
| `dokime.client.v1.files.Uploads/Get` | `dokime.host.v1.files.Uploads/Get` | ALL | `dokime.host.v1.files.UploadsGetRequest` | `dokime.host.v1.files.UploadsGetResponse` |
| `dokime.client.v1.files.Uploads/Publish` | `dokime.host.v1.files.Uploads/Publish` | ALL | `dokime.host.v1.files.UploadsPublishRequest` | `dokime.host.v1.files.UploadsPublishResponse` |
| `dokime.client.v1.files.Uploads/Resume` | `dokime.host.v1.files.Uploads/Resume` | ALL | `dokime.host.v1.files.UploadsResumeRequest` | `dokime.host.v1.files.UploadsResumeResponse` |
| `dokime.client.v1.media.Captures/Create` | `dokime.host.v1.media.Captures/Create` | CLIENTS | `dokime.host.v1.media.CapturesCreateRequest` | `dokime.host.v1.media.CapturesCreateResponse` |
| `dokime.client.v1.media.Captures/Get` | `dokime.host.v1.media.Captures/Get` | CLIENTS | `dokime.host.v1.media.CapturesGetRequest` | `dokime.host.v1.media.CapturesGetResponse` |
| `dokime.client.v1.media.Captures/Stop` | `dokime.host.v1.media.Captures/Stop` | CLIENTS | `dokime.host.v1.media.CapturesStopRequest` | `dokime.host.v1.media.CapturesStopResponse` |
| `dokime.client.v1.media.Captures/Watch` | `dokime.host.v1.media.Captures/Watch` | CLIENTS | `dokime.host.v1.media.CapturesWatchRequest` | `dokime.host.v1.media.CapturesWatchResponse` |
| `dokime.client.v1.plots.Settings/Get` | `dokime.host.v1.plots.Settings/Get` | CLIENTS | `dokime.host.v1.plots.SettingsGetRequest` | `dokime.host.v1.plots.SettingsGetResponse` |
| `dokime.client.v1.plots.Settings/Set` | `dokime.host.v1.plots.Settings/Set` | CLIENTS | `dokime.host.v1.plots.SettingsSetRequest` | `dokime.host.v1.plots.SettingsSetResponse` |
| `dokime.client.v1.plots.Snapshots/List` | `dokime.host.v1.plots.Snapshots/List` | CLIENTS | `dokime.host.v1.plots.SnapshotsListRequest` | `dokime.host.v1.plots.SnapshotsListResponse` |
| `dokime.client.v1.plots.Snapshots/Watch` | `dokime.host.v1.plots.Snapshots/Watch` | CLIENTS | `dokime.host.v1.plots.SnapshotsWatchRequest` | `dokime.host.v1.plots.SnapshotsWatchResponse` |
| `dokime.client.v1.plugins.Settings/Get` | `harmos.host.v1.plugins.Settings/List` | ALL | `harmos.host.v1.plugins.SettingsListRequest` | `harmos.host.v1.plugins.SettingsListResponse` |
| `dokime.client.v1.plugins.Settings/Set` | `harmos.host.v1.plugins.Settings/Set` | CLIENTS | `harmos.host.v1.plugins.SettingsSetRequest` | `harmos.host.v1.plugins.SettingsSetResponse` |
| `dokime.client.v1.prompts.Answers/Create` | `dokime.host.v1.prompts.Answers/Create` | CLIENTS | `dokime.host.v1.prompts.AnswersCreateRequest` | `dokime.host.v1.prompts.AnswersCreateResponse` |
| `dokime.client.v1.storage.Sources/Get` | `dokime.host.v1.storage.Sources/Get` | CLIENTS | `dokime.host.v1.storage.SourcesGetRequest` | `dokime.host.v1.storage.SourcesGetResponse` |
| `dokime.client.v1.storage.Sources/Set` | `dokime.host.v1.storage.Sources/Set` | CLIENTS | `dokime.host.v1.storage.SourcesSetRequest` | `dokime.host.v1.storage.SourcesSetResponse` |
| `dokime.plugin.host.v1.Backend/Environment` | `dokime.host.v1.Backend/Environment` | PLUGINS | `dokime.host.v1.BackendEnvironmentRequest` | `dokime.host.v1.BackendEnvironmentResponse` |
| `dokime.plugin.host.v1.Documents/AppendParts` | `dokime.host.v1.Documents/AppendParts` | PLUGINS | `dokime.host.v1.DocumentsAppendPartsRequest` | `dokime.host.v1.DocumentsAppendPartsResponse` |
| `dokime.plugin.host.v1.Effects/Create` | `dokime.host.v1.Effects/Create` | PLUGINS | `dokime.host.v1.EffectsCreateRequest` | `dokime.host.v1.EffectsCreateResponse` |
| `dokime.plugin.host.v1.Effects/Get` | `dokime.host.v1.Effects/Get` | PLUGINS | `dokime.host.v1.EffectsGetRequest` | `dokime.host.v1.EffectsGetResponse` |
| `dokime.plugin.host.v1.Effects/Set` | `dokime.host.v1.Effects/Set` | PLUGINS | `dokime.host.v1.EffectsSetRequest` | `dokime.host.v1.EffectsSetResponse` |
| `dokime.plugin.host.v1.Evidence/Statistics` | `dokime.host.v1.Evidence/Statistics` | ALL | `dokime.host.v1.EvidenceStatisticsRequest` | `dokime.host.v1.EvidenceStatisticsResponse` |
| `dokime.plugin.host.v1.Files/Delete` | `dokime.host.v1.Files/Delete` | ALL | `dokime.host.v1.FilesDeleteRequest` | `dokime.host.v1.FilesDeleteResponse` |
| `dokime.plugin.host.v1.Files/Get` | `dokime.host.v1.Files/Get` | ALL | `dokime.host.v1.FilesGetRequest` | `dokime.host.v1.FilesGetResponse` |
| `dokime.plugin.host.v1.Files/Read` | `dokime.host.v1.Files/Read` | ALL | `dokime.host.v1.FilesReadRequest` | `dokime.host.v1.FilesReadResponse` |
| `dokime.plugin.host.v1.Files/Watch` | `dokime.host.v1.Files/Watch` | ALL | `dokime.host.v1.FilesWatchRequest` (target: file or upload) | `dokime.host.v1.FilesWatchResponse` |
| `dokime.plugin.host.v1.Prompts/Create` | `dokime.host.v1.Prompts/Create` | PLUGINS | `dokime.host.v1.PromptsCreateRequest` | `dokime.host.v1.PromptsCreateResponse` |
| `dokime.plugin.host.v1.content.Artifacts/Create` | `dokime.host.v1.content.Artifacts/Create` | ALL | `dokime.host.v1.content.ArtifactsCreateRequest` | `dokime.host.v1.content.ArtifactsCreateResponse` |
| `dokime.plugin.host.v1.evidence.Logs/Append` | `dokime.host.v1.evidence.Logs/Append` | PLUGINS | `dokime.host.v1.evidence.LogsAppendRequest` | `dokime.host.v1.evidence.LogsAppendResponse` |
| `dokime.plugin.host.v1.evidence.Records/Append` | `dokime.host.v1.evidence.Records/Append` | PLUGINS | `dokime.host.v1.evidence.RecordsAppendRequest` | `dokime.host.v1.evidence.RecordsAppendResponse` |
| `dokime.plugin.host.v1.evidence.Records/List` | `dokime.host.v1.evidence.Records/List` | ALL | `dokime.host.v1.evidence.RecordsListRequest` | `dokime.host.v1.evidence.RecordsListResponse` |
| `dokime.plugin.host.v1.evidence.Traceability/Get` | `dokime.host.v1.evidence.Traceability/Get` | ALL | `dokime.host.v1.evidence.TraceabilityGetRequest` | `dokime.host.v1.evidence.TraceabilityGetResponse` |
| `dokime.plugin.host.v1.evidence.Traceability/Set` | `dokime.host.v1.evidence.Traceability/Set` | PLUGINS | `dokime.host.v1.evidence.TraceabilitySetRequest` | `dokime.host.v1.evidence.TraceabilitySetResponse` |
| `dokime.plugin.host.v1.executions.Progress/Get` | `dokime.host.v1.executions.Progress/Get` | ALL | `dokime.host.v1.executions.ProgressGetRequest` | `dokime.host.v1.executions.ProgressGetResponse` |
| `dokime.plugin.host.v1.executions.Progress/Set` | `dokime.host.v1.executions.Progress/Set` | PLUGINS | `dokime.host.v1.executions.ProgressSetRequest` | `dokime.host.v1.executions.ProgressSetResponse` |
| `dokime.plugin.host.v1.files.Captures/Start` | `dokime.host.v1.files.Captures/Start` | PLUGINS | `dokime.host.v1.files.CapturesStartRequest` | `dokime.host.v1.files.CapturesStartResponse` |
| `dokime.plugin.host.v1.files.Captures/Stop` | `dokime.host.v1.files.Captures/Stop` | PLUGINS | `dokime.host.v1.files.CapturesStopRequest` | `dokime.host.v1.files.CapturesStopResponse` |
| `dokime.plugin.host.v1.files.Uploads/Append` | `dokime.host.v1.files.Uploads/Append` | ALL | `dokime.host.v1.files.UploadsAppendRequest` | `dokime.host.v1.files.UploadsAppendResponse` |
| `dokime.plugin.host.v1.files.Uploads/Create` | `dokime.host.v1.files.Uploads/Create` | ALL | `dokime.host.v1.files.UploadsCreateRequest` | `dokime.host.v1.files.UploadsCreateResponse` |
| `dokime.plugin.host.v1.files.Uploads/Delete` | `dokime.host.v1.files.Uploads/Delete` | ALL | `dokime.host.v1.files.UploadsDeleteRequest` | `dokime.host.v1.files.UploadsDeleteResponse` |
| `dokime.plugin.host.v1.files.Uploads/Get` | `dokime.host.v1.files.Uploads/Get` | ALL | `dokime.host.v1.files.UploadsGetRequest` | `dokime.host.v1.files.UploadsGetResponse` |
| `dokime.plugin.host.v1.files.Uploads/Publish` | `dokime.host.v1.files.Uploads/Publish` | ALL | `dokime.host.v1.files.UploadsPublishRequest` | `dokime.host.v1.files.UploadsPublishResponse` |
| `dokime.plugin.host.v1.files.Uploads/Resume` | `dokime.host.v1.files.Uploads/Resume` | ALL | `dokime.host.v1.files.UploadsResumeRequest` | `dokime.host.v1.files.UploadsResumeResponse` |
| `dokime.secrets.read` | `dokime.host.v1.Secrets/Get` | ALL | `dokime.host.v1.SecretsGetRequest` | `dokime.host.v1.SecretsGetResponse` |
| `dokime.settings.read` | `harmos.host.v1.plugins.Settings/List` | ALL | `harmos.host.v1.plugins.SettingsListRequest` | `harmos.host.v1.plugins.SettingsListResponse` |
| `plugin-authored Protected binding (Criteria/Evaluate)` | `dokime.plugin.v1.Criteria/Evaluate` | Host Only (Protected; No Callers Option) | `dokime.plugin.v1.CriteriaEvaluateRequest` | `dokime.plugin.v1.CriteriaEvaluateResponse` |
| `plugin-authored Protected binding (Executions/Cleanup)` | `dokime.plugin.v1.Executions/Cleanup` | Host Only (Protected; No Callers Option) | `dokime.plugin.v1.ExecutionsCleanupRequest` | `dokime.plugin.v1.ExecutionsCleanupResponse` |
| `plugin-authored Protected binding (Executions/Initialize)` | `dokime.plugin.v1.Executions/Initialize` | Host Only (Protected; No Callers Option) | `dokime.plugin.v1.ExecutionsInitializeRequest` | `dokime.plugin.v1.ExecutionsInitializeResponse` |
| `plugin-authored Protected binding (Steps/Execute)` | `dokime.plugin.v1.Steps/Execute` | Host Only (Protected; No Callers Option) | `dokime.plugin.v1.StepsExecuteRequest` | `dokime.plugin.v1.StepsExecuteResponse` |
| `plugin-authored Protected binding (TestCases/Materialize)` | `dokime.plugin.v1.TestCases/Materialize` | Host Only (Protected; No Callers Option) | `dokime.plugin.v1.TestCasesMaterializeRequest` | `dokime.plugin.v1.TestCasesMaterializeResponse` |
| new | `dokime.host.v1.plugins.Secrets/List` | CLIENTS | `dokime.host.v1.plugins.SecretsListRequest` | `dokime.host.v1.plugins.SecretsListResponse` |

New resource: `dokime.host.v1.plugins.Secrets` has List only, for CLIENTS.
It returns each `SecretRequirement { declaration, available }`: a declared
identifier and whether Secret Storage holds a secret under exactly that identifier,
never values. The model contribution is `dokime.model.v1.SecretsDeclarations`
containing `SecretDeclaration { metadata, required }`. There is no binding,
mapping or revisioned catalog between a declaration and its secret (owner decision
of 2026-11-15). A plugin's Secrets.Get of an undeclared name is refused; a declared
name with no stored secret fails, naming the action and identifier and asking to
add it in Secret Storage.

Decision: product plugin methods have no callers option; Protected admission
restricts their bindings to the host. Upload replaces native wire FileProducer/FileSession/StagedFile/FileProgress;
Files owns FileBatch/FileCompletion. Native storage remains PostcardV1 and must
receive losslessly encoded native values, never relabeled protobuf bytes. Verdict summary/status live with Cycles to
avoid a file import cycle. Buf permits package import cycles because nested owner
packages exchange values, while protoc still rejects all file import cycles.

### Message and Enum Identity Mapping

Every authored pre-pin product message/enum is listed; synthetic map-entry types
follow their containing message. Deleted product plugin metadata/cursors/install
frames have no replacement wrapper: use the composed library record and Files upload.

| Old Type | Final Type or Removal |
| --- | --- |
| `dokime.client.v1.AggregateReply` | `dokime.host.v1.AggregateReply` |
| `dokime.client.v1.QueryAggregate` | `dokime.host.v1.QueryAggregate` |
| `dokime.client.v1.QueryFilter` | `dokime.host.v1.QueryFilter` |
| `dokime.client.v1.QueryFilterOperator` | `dokime.host.v1.QueryFilterOperator` |
| `dokime.client.v1.QueryRow` | `dokime.host.v1.QueryRow` |
| `dokime.client.v1.QuerySource` | `dokime.host.v1.QuerySource` |
| `dokime.client.v1.AggregatesGetRequest` | `dokime.host.v1.AggregatesGetRequest` |
| `dokime.client.v1.AggregatesGetResponse` | `dokime.host.v1.AggregatesGetResponse` |
| `dokime.client.v1.AnalysesListRequest` | `dokime.host.v1.AnalysesListRequest` |
| `dokime.client.v1.AnalysesListResponse` | `dokime.host.v1.AnalysesListResponse` |
| `dokime.client.v1.AppearanceSnapshot` | `dokime.host.v1.AppearanceSnapshot` |
| `dokime.client.v1.OrganizationIdentity` | `dokime.host.v1.OrganizationIdentity` |
| `dokime.client.v1.AppearanceLogo` | `dokime.host.v1.AppearanceLogo` |
| `dokime.client.v1.ThemeFamily` | `dokime.host.v1.ThemeFamily` |
| `dokime.client.v1.ThemeCornerRadii` | `dokime.host.v1.ThemeCornerRadii` |
| `dokime.client.v1.ThemeVariant` | `dokime.host.v1.ThemeVariant` |
| `dokime.client.v1.AppearanceFailure` | `dokime.host.v1.AppearanceFailure` |
| `dokime.client.v1.LogoVariant` | `dokime.host.v1.LogoVariant` |
| `dokime.client.v1.ThemeModePolicy` | `dokime.host.v1.ThemeModePolicy` |
| `dokime.client.v1.AppearanceFailureKind` | `dokime.host.v1.AppearanceFailureKind` |
| `dokime.client.v1.BackendAppearanceRequest` | `dokime.host.v1.BackendAppearanceRequest` |
| `dokime.client.v1.BackendAppearanceResponse` | `dokime.host.v1.BackendAppearanceResponse` |
| `dokime.client.v1.backend.EvidenceCounts` | `dokime.host.v1.backend.EvidenceCounts` |
| `dokime.client.v1.backend.RuntimeShellSnapshot` | `dokime.host.v1.backend.RuntimeShellSnapshot` |
| `dokime.client.v1.backend.OverviewGetRequest` | `dokime.host.v1.backend.OverviewGetRequest` |
| `dokime.client.v1.backend.OverviewGetResponse` | `dokime.host.v1.backend.OverviewGetResponse` |
| `dokime.client.v1.backend.OverviewWatchRequest` | `dokime.host.v1.backend.OverviewWatchRequest` |
| `dokime.client.v1.backend.OverviewWatchResponse` | `dokime.host.v1.backend.OverviewWatchResponse` |
| `dokime.client.v1.CaseActions` | `dokime.host.v1.CaseActions` |
| `dokime.client.v1.CaseDetail` | `dokime.host.v1.CaseDetail` |
| `dokime.client.v1.CaseParameter` | `dokime.host.v1.CaseParameter` |
| `dokime.client.v1.CaseStatus` | `dokime.host.v1.CaseStatus` |
| `dokime.client.v1.CaseSummary` | `dokime.host.v1.CaseSummary` |
| `dokime.client.v1.PlannedCase` | `dokime.host.v1.PlannedCase` |
| `dokime.client.v1.RegisteredSelection` | `dokime.host.v1.RegisteredSelection` |
| `dokime.client.v1.RegisteredSelectionFilesEntry` | `dokime.host.v1.RegisteredSelectionFilesEntry` |
| `dokime.client.v1.RegisteredSelectionParametersEntry` | `dokime.host.v1.RegisteredSelectionParametersEntry` |
| `dokime.client.v1.SelectedFile` | `dokime.host.v1.SelectedFile` |
| `dokime.client.v1.SelectedFileKind` | `dokime.host.v1.SelectedFileKind` |
| `dokime.client.v1.SelectionPlan` | `dokime.host.v1.SelectionPlan` |
| `dokime.client.v1.SelectionValues` | `dokime.host.v1.SelectionValues` |
| `dokime.client.v1.CasesCreateRequest` | `dokime.host.v1.CasesCreateRequest` |
| `dokime.client.v1.CasesCreateResponse` | `dokime.host.v1.CasesCreateResponse` |
| `dokime.client.v1.CasesGetRequest` | `dokime.host.v1.CasesGetRequest` |
| `dokime.client.v1.CasesGetResponse` | `dokime.host.v1.CasesGetResponse` |
| `dokime.client.v1.CasesListRequest` | `dokime.host.v1.CasesListRequest` |
| `dokime.client.v1.CasesListResponse` | `dokime.host.v1.CasesListResponse` |
| `dokime.client.v1.CasesWatchRequest` | `dokime.host.v1.CasesWatchRequest` |
| `dokime.client.v1.CasesWatchResponse` | `dokime.host.v1.CasesWatchResponse` |
| `dokime.client.v1.CasesDeleteRequest` | `dokime.host.v1.CasesDeleteRequest` |
| `dokime.client.v1.CasesDeleteResponse` | `dokime.host.v1.CasesDeleteResponse` |
| `dokime.client.v1.CasesParametersRequest` | `dokime.host.v1.CasesParametersRequest` |
| `dokime.client.v1.CasesParametersResponse` | `dokime.host.v1.CasesParametersResponse` |
| `dokime.client.v1.content.ArtifactsGetRequest` | `dokime.host.v1.content.ArtifactsGetRequest` |
| `dokime.client.v1.content.ArtifactsGetResponse` | `dokime.host.v1.content.ArtifactsGetResponse` |
| `dokime.client.v1.content.ArtifactsCreateRequest` | `dokime.host.v1.content.ArtifactsCreateRequest` |
| `dokime.client.v1.content.ArtifactsCreateResponse` | `dokime.host.v1.content.ArtifactsCreateResponse` |
| `dokime.client.v1.content.ArtifactsListRequest` | `dokime.host.v1.content.ArtifactsListRequest` |
| `dokime.client.v1.content.ArtifactsListResponse` | `dokime.host.v1.content.ArtifactsListResponse` |
| `dokime.client.v1.content.ArtifactsWatchRequest` | `dokime.host.v1.content.ArtifactsWatchRequest` |
| `dokime.client.v1.content.ArtifactsWatchResponse` | `dokime.host.v1.content.ArtifactsWatchResponse` |
| `dokime.client.v1.content.ExecutionFileKind` | `dokime.host.v1.content.ExecutionFileKind` |
| `dokime.client.v1.content.ExecutionFileMetadata` | `dokime.host.v1.content.ExecutionFileMetadata` |
| `dokime.client.v1.content.ExecutionFileState` | `dokime.host.v1.content.ExecutionFileState` |
| `dokime.client.v1.content.ExecutionFilesGetRequest` | `dokime.host.v1.content.ExecutionFilesGetRequest` |
| `dokime.client.v1.content.ExecutionFilesGetResponse` | `dokime.host.v1.content.ExecutionFilesGetResponse` |
| `dokime.client.v1.content.ExecutionFilesListRequest` | `dokime.host.v1.content.ExecutionFilesListRequest` |
| `dokime.client.v1.content.ExecutionFilesListResponse` | `dokime.host.v1.content.ExecutionFilesListResponse` |
| `dokime.client.v1.content.ExecutionFilesWatchRequest` | `dokime.host.v1.content.ExecutionFilesWatchRequest` |
| `dokime.client.v1.content.ExecutionFilesWatchResponse` | `dokime.host.v1.content.ExecutionFilesWatchResponse` |
| `dokime.client.v1.CycleFilter` | `dokime.host.v1.CycleFilter` |
| `dokime.client.v1.CycleOrder` | `dokime.host.v1.CycleOrder` |
| `dokime.client.v1.CycleOrderKey` | `dokime.host.v1.CycleOrderKey` |
| `dokime.client.v1.CycleSelection` | `dokime.host.v1.CycleSelection` |
| `dokime.client.v1.RerunCaseNotice` | `dokime.host.v1.RerunCaseNotice` |
| `dokime.client.v1.CycleWritable` | `dokime.host.v1.CycleWritable` |
| `dokime.client.v1.CyclesGetRequest` | `dokime.host.v1.CyclesGetRequest` |
| `dokime.client.v1.CyclesGetResponse` | `dokime.host.v1.CyclesGetResponse` |
| `dokime.client.v1.CyclesSetRequest` | `dokime.host.v1.CyclesSetRequest` |
| `dokime.client.v1.CyclesSetResponse` | `dokime.host.v1.CyclesSetResponse` |
| `dokime.client.v1.CyclesListRequest` | `dokime.host.v1.CyclesListRequest` |
| `dokime.client.v1.CyclesListResponse` | `dokime.host.v1.CyclesListResponse` |
| `dokime.client.v1.CyclesWatchRequest` | `dokime.host.v1.CyclesWatchRequest` |
| `dokime.client.v1.CyclesWatchResponse` | `dokime.host.v1.CyclesWatchResponse` |
| `dokime.client.v1.CyclesCreateRequest` | `dokime.host.v1.CyclesCreateRequest` |
| `dokime.client.v1.CyclesCreateResponse` | `dokime.host.v1.CyclesCreateResponse` |
| `dokime.client.v1.CyclesSealRequest` | `dokime.host.v1.CyclesSealRequest` |
| `dokime.client.v1.CyclesSealResponse` | `dokime.host.v1.CyclesSealResponse` |
| `dokime.client.v1.DocumentBinding` | `dokime.host.v1.DocumentBinding` |
| `dokime.client.v1.DocumentBindingResult` | `dokime.host.v1.DocumentBindingResult` |
| `dokime.client.v1.DocumentCard` | `dokime.host.v1.DocumentCard` |
| `dokime.client.v1.DocumentCardBody` | `dokime.host.v1.DocumentCardBody` |
| `dokime.client.v1.DocumentCardBudget` | `dokime.host.v1.DocumentCardBudget` |
| `dokime.client.v1.DocumentCardFailure` | `dokime.host.v1.DocumentCardFailure` |
| `dokime.client.v1.DocumentCardKind` | `dokime.host.v1.DocumentCardKind` |
| `dokime.client.v1.DocumentCompletion` | `dokime.host.v1.DocumentCompletion` |
| `dokime.client.v1.DocumentDensity` | `dokime.host.v1.DocumentDensity` |
| `dokime.client.v1.DocumentEvidenceCut` | `dokime.host.v1.DocumentEvidenceCut` |
| `dokime.client.v1.DocumentFrame` | `dokime.host.v1.DocumentFrame` |
| `dokime.client.v1.DocumentFrameFrame` | `dokime.host.v1.DocumentFrameFrame` |
| `dokime.client.v1.DocumentFrontiers` | `dokime.host.v1.DocumentFrontiers` |
| `dokime.client.v1.DocumentHeader` | `dokime.host.v1.DocumentHeader` |
| `dokime.client.v1.DocumentLayoutNode` | `dokime.host.v1.DocumentLayoutNode` |
| `dokime.client.v1.DocumentLayoutNodeNode` | `dokime.host.v1.DocumentLayoutNodeNode` |
| `dokime.client.v1.DocumentOwnerReference` | `dokime.host.v1.DocumentOwnerReference` |
| `dokime.client.v1.DocumentPage` | `dokime.host.v1.DocumentPage` |
| `dokime.client.v1.DocumentPlacement` | `dokime.host.v1.DocumentPlacement` |
| `dokime.client.v1.DocumentPlotDefinition` | `dokime.host.v1.DocumentPlotDefinition` |
| `dokime.client.v1.DocumentRail` | `dokime.host.v1.DocumentRail` |
| `dokime.client.v1.DocumentRepeat` | `dokime.host.v1.DocumentRepeat` |
| `dokime.client.v1.DocumentRepeatInstance` | `dokime.host.v1.DocumentRepeatInstance` |
| `dokime.client.v1.DocumentRepeatSelection` | `dokime.host.v1.DocumentRepeatSelection` |
| `dokime.client.v1.DocumentSection` | `dokime.host.v1.DocumentSection` |
| `dokime.client.v1.DocumentSectionIdentity` | `dokime.host.v1.DocumentSectionIdentity` |
| `dokime.client.v1.DocumentSpan` | `dokime.host.v1.DocumentSpan` |
| `dokime.client.v1.DocumentSummary` | `dokime.host.v1.DocumentSummary` |
| `dokime.client.v1.DocumentSummaryCondition` | `dokime.host.v1.DocumentSummaryCondition` |
| `dokime.client.v1.DocumentSummaryFormat` | `dokime.host.v1.DocumentSummaryFormat` |
| `dokime.client.v1.DocumentSummaryGroup` | `dokime.host.v1.DocumentSummaryGroup` |
| `dokime.client.v1.DocumentSummaryGroupDefinition` | `dokime.host.v1.DocumentSummaryGroupDefinition` |
| `dokime.client.v1.DocumentSummaryPresentation` | `dokime.host.v1.DocumentSummaryPresentation` |
| `dokime.client.v1.DocumentSummaryStat` | `dokime.host.v1.DocumentSummaryStat` |
| `dokime.client.v1.DocumentSummaryState` | `dokime.host.v1.DocumentSummaryState` |
| `dokime.client.v1.DocumentSummaryTile` | `dokime.host.v1.DocumentSummaryTile` |
| `dokime.client.v1.DocumentSummaryTileDefinition` | `dokime.host.v1.DocumentSummaryTileDefinition` |
| `dokime.client.v1.DocumentTable` | `dokime.host.v1.DocumentTable` |
| `dokime.client.v1.DocumentTableRow` | `dokime.host.v1.DocumentTableRow` |
| `dokime.client.v1.DocumentTelemetryCut` | `dokime.host.v1.DocumentTelemetryCut` |
| `dokime.client.v1.DocumentWarning` | `dokime.host.v1.DocumentWarning` |
| `dokime.client.v1.PlotAxisKind` | `dokime.host.v1.PlotAxisKind` |
| `dokime.client.v1.PlotPresentation` | `dokime.host.v1.PlotPresentation` |
| `dokime.client.v1.PlotReferenceBand` | `dokime.host.v1.PlotReferenceBand` |
| `dokime.client.v1.PlotReferenceLine` | `dokime.host.v1.PlotReferenceLine` |
| `dokime.client.v1.PlotReferenceValue` | `dokime.host.v1.PlotReferenceValue` |
| `dokime.client.v1.DocumentsGetRequest` | `dokime.host.v1.DocumentsGetRequest` |
| `dokime.client.v1.DocumentsGetResponse` | `dokime.host.v1.DocumentsGetResponse` |
| `dokime.client.v1.DocumentsWatchRequest` | `dokime.host.v1.DocumentsWatchRequest` |
| `dokime.client.v1.DocumentsWatchResponse` | `dokime.host.v1.DocumentsWatchResponse` |
| `dokime.client.v1.Statistics` | `dokime.host.v1.Statistics` |
| `dokime.client.v1.EvidenceSubject` | `dokime.host.v1.EvidenceSubject` |
| `dokime.client.v1.Output` | `dokime.host.v1.Output` |
| `dokime.client.v1.EvidenceStatisticsRequest` | `dokime.host.v1.EvidenceStatisticsRequest` |
| `dokime.client.v1.EvidenceStatisticsResponse` | `dokime.host.v1.EvidenceStatisticsResponse` |
| `dokime.client.v1.EvidenceArchiveRequest` | `dokime.host.v1.EvidenceArchiveRequest` |
| `dokime.client.v1.EvidenceArchiveResponse` | `dokime.host.v1.EvidenceArchiveResponse` |
| `dokime.client.v1.evidence.HealthItem` | `dokime.host.v1.evidence.HealthItem` |
| `dokime.client.v1.evidence.HealthEventsListRequest` | `dokime.host.v1.evidence.HealthEventsListRequest` |
| `dokime.client.v1.evidence.HealthEventsListResponse` | `dokime.host.v1.evidence.HealthEventsListResponse` |
| `dokime.client.v1.evidence.HealthEventsWatchRequest` | `dokime.host.v1.evidence.HealthEventsWatchRequest` |
| `dokime.client.v1.evidence.HealthEventsWatchResponse` | `dokime.host.v1.evidence.HealthEventsWatchResponse` |
| `dokime.client.v1.evidence.OperatorNote` | `dokime.host.v1.evidence.OperatorNote` |
| `dokime.client.v1.evidence.NoteWritable` | `dokime.host.v1.evidence.NoteWritable` |
| `dokime.client.v1.evidence.NotesGetRequest` | `dokime.host.v1.evidence.NotesGetRequest` |
| `dokime.client.v1.evidence.NotesGetResponse` | `dokime.host.v1.evidence.NotesGetResponse` |
| `dokime.client.v1.evidence.NotesSetRequest` | `dokime.host.v1.evidence.NotesSetRequest` |
| `dokime.client.v1.evidence.NotesSetResponse` | `dokime.host.v1.evidence.NotesSetResponse` |
| `dokime.client.v1.evidence.NotesListRequest` | `dokime.host.v1.evidence.NotesListRequest` |
| `dokime.client.v1.evidence.NotesListResponse` | `dokime.host.v1.evidence.NotesListResponse` |
| `dokime.client.v1.evidence.NotesWatchRequest` | `dokime.host.v1.evidence.NotesWatchRequest` |
| `dokime.client.v1.evidence.NotesWatchResponse` | `dokime.host.v1.evidence.NotesWatchResponse` |
| `dokime.client.v1.evidence.NotesCreateRequest` | `dokime.host.v1.evidence.NotesCreateRequest` |
| `dokime.client.v1.evidence.NotesCreateResponse` | `dokime.host.v1.evidence.NotesCreateResponse` |
| `dokime.client.v1.evidence.CapturedErrorCause` | `dokime.host.v1.evidence.CapturedErrorCause` |
| `dokime.client.v1.evidence.CapturedRecord` | `dokime.host.v1.evidence.CapturedRecord` |
| `dokime.client.v1.evidence.CapturedRecordError` | `dokime.host.v1.evidence.CapturedRecordError` |
| `dokime.client.v1.evidence.CycleScope` | `dokime.host.v1.evidence.CycleScope` |
| `dokime.client.v1.evidence.RecordScope` | `dokime.host.v1.evidence.RecordScope` |
| `dokime.client.v1.evidence.CapturedRecordSelection` | `dokime.host.v1.evidence.CapturedRecordSelection` |
| `dokime.client.v1.evidence.RecordsListRequest` | `dokime.host.v1.evidence.RecordsListRequest` |
| `dokime.client.v1.evidence.RecordsListResponse` | `dokime.host.v1.evidence.RecordsListResponse` |
| `dokime.client.v1.evidence.RecordsWatchRequest` | `dokime.host.v1.evidence.RecordsWatchRequest` |
| `dokime.client.v1.evidence.RecordsWatchResponse` | `dokime.host.v1.evidence.RecordsWatchResponse` |
| `dokime.client.v1.evidence.Bucket` | `dokime.host.v1.evidence.Bucket` |
| `dokime.client.v1.evidence.BucketRequest` | `dokime.host.v1.evidence.BucketRequest` |
| `dokime.client.v1.evidence.BucketValue` | `dokime.host.v1.evidence.BucketValue` |
| `dokime.client.v1.evidence.SampleRow` | `dokime.host.v1.evidence.SampleRow` |
| `dokime.client.v1.evidence.StatisticValue` | `dokime.host.v1.evidence.StatisticValue` |
| `dokime.client.v1.evidence.TelemetrySample` | `dokime.host.v1.evidence.TelemetrySample` |
| `dokime.client.v1.evidence.SamplesListRequest` | `dokime.host.v1.evidence.SamplesListRequest` |
| `dokime.client.v1.evidence.SamplesListResponse` | `dokime.host.v1.evidence.SamplesListResponse` |
| `dokime.client.v1.evidence.SamplesWatchRequest` | `dokime.host.v1.evidence.SamplesWatchRequest` |
| `dokime.client.v1.evidence.SamplesWatchResponse` | `dokime.host.v1.evidence.SamplesWatchResponse` |
| `dokime.client.v1.evidence.TraceabilitySnapshot` | `dokime.host.v1.evidence.TraceabilitySnapshot` |
| `dokime.client.v1.evidence.TraceabilityGetRequest` | `dokime.host.v1.evidence.TraceabilityGetRequest` |
| `dokime.client.v1.evidence.TraceabilityGetResponse` | `dokime.host.v1.evidence.TraceabilityGetResponse` |
| `dokime.client.v1.evidence.TraceabilityWatchRequest` | `dokime.host.v1.evidence.TraceabilityWatchRequest` |
| `dokime.client.v1.evidence.TraceabilityWatchResponse` | `dokime.host.v1.evidence.TraceabilityWatchResponse` |
| `dokime.client.v1.FrozenSchemas` | `dokime.host.v1.FrozenSchemas` |
| `dokime.client.v1.LiveExecutionState` | `dokime.host.v1.LiveExecutionState` |
| `dokime.client.v1.CriterionStatus` | `dokime.host.v1.CriterionStatus` |
| `dokime.client.v1.Diagnostic` | `dokime.host.v1.Diagnostic` |
| `dokime.client.v1.ExecutionBinding` | `dokime.host.v1.ExecutionBinding` |
| `dokime.client.v1.ExecutionControls` | `dokime.host.v1.ExecutionControls` |
| `dokime.client.v1.ExecutionOutcomeCounts` | `dokime.host.v1.ExecutionOutcomeCounts` |
| `dokime.client.v1.ExecutionDetail` | `dokime.host.v1.ExecutionDetail` |
| `dokime.client.v1.ExecutionFilter` | `dokime.host.v1.ExecutionFilter` |
| `dokime.client.v1.ExecutionMode` | `dokime.host.v1.ExecutionMode` |
| `dokime.client.v1.ExecutionOrder` | `dokime.host.v1.ExecutionOrder` |
| `dokime.client.v1.ExecutionOrderKey` | `dokime.host.v1.ExecutionOrderKey` |
| `dokime.client.v1.ExecutionParameterSet` | `dokime.host.v1.ExecutionParameterSet` |
| `dokime.client.v1.ExecutionSelection` | `dokime.host.v1.ExecutionSelection` |
| `dokime.client.v1.ExecutionStep` | `dokime.host.v1.ExecutionStep` |
| `dokime.client.v1.ExecutionSummary` | `dokime.host.v1.ExecutionSummary` |
| `dokime.client.v1.MaterializedParameter` | `dokime.host.v1.MaterializedParameter` |
| `dokime.client.v1.ParameterSource` | `dokime.host.v1.ParameterSource` |
| `dokime.client.v1.ReportState` | `dokime.host.v1.ReportState` |
| `dokime.client.v1.ReportStatus` | `dokime.host.v1.ReportStatus` |
| `dokime.client.v1.StepCriterion` | `dokime.host.v1.StepCriterion` |
| `dokime.client.v1.StepStatus` | `dokime.host.v1.StepStatus` |
| `dokime.client.v1.ExecutionWritable` | `dokime.host.v1.ExecutionWritable` |
| `dokime.client.v1.ExecutionsGetRequest` | `dokime.host.v1.ExecutionsGetRequest` |
| `dokime.client.v1.ExecutionsGetResponse` | `dokime.host.v1.ExecutionsGetResponse` |
| `dokime.client.v1.ExecutionsSetRequest` | `dokime.host.v1.ExecutionsSetRequest` |
| `dokime.client.v1.ExecutionsSetResponse` | `dokime.host.v1.ExecutionsSetResponse` |
| `dokime.client.v1.ExecutionsListRequest` | `dokime.host.v1.ExecutionsListRequest` |
| `dokime.client.v1.ExecutionsListResponse` | `dokime.host.v1.ExecutionsListResponse` |
| `dokime.client.v1.ExecutionsWatchRequest` | `dokime.host.v1.ExecutionsWatchRequest` |
| `dokime.client.v1.ExecutionsWatchResponse` | `dokime.host.v1.ExecutionsWatchResponse` |
| `dokime.client.v1.ExecutionsCreateRequest` | `dokime.host.v1.ExecutionsCreateRequest` |
| `dokime.client.v1.ExecutionsCreateResponse` | `dokime.host.v1.ExecutionsCreateResponse` |
| `dokime.client.v1.ExecutionsStartRequest` | `dokime.host.v1.ExecutionsStartRequest` |
| `dokime.client.v1.ExecutionsStartResponse` | `dokime.host.v1.ExecutionsStartResponse` |
| `dokime.client.v1.ExecutionsStopRequest` | `dokime.host.v1.ExecutionsStopRequest` |
| `dokime.client.v1.ExecutionsStopResponse` | `dokime.host.v1.ExecutionsStopResponse` |
| `dokime.client.v1.ExecutionsCancelRequest` | `dokime.host.v1.ExecutionsCancelRequest` |
| `dokime.client.v1.ExecutionsCancelResponse` | `dokime.host.v1.ExecutionsCancelResponse` |
| `dokime.client.v1.ExecutionsDeleteRequest` | `dokime.host.v1.ExecutionsDeleteRequest` |
| `dokime.client.v1.ExecutionsDeleteResponse` | `dokime.host.v1.ExecutionsDeleteResponse` |
| `dokime.client.v1.ExecutionsParametersRequest` | `dokime.host.v1.ExecutionsParametersRequest` |
| `dokime.client.v1.ExecutionsParametersResponse` | `dokime.host.v1.ExecutionsParametersResponse` |
| `dokime.client.v1.ExecutionsStepsRequest` | `dokime.host.v1.ExecutionsStepsRequest` |
| `dokime.client.v1.ExecutionsStepsResponse` | `dokime.host.v1.ExecutionsStepsResponse` |
| `dokime.client.v1.executions.ExecutionCommandAction` | `dokime.host.v1.executions.ExecutionCommandAction` |
| `dokime.client.v1.executions.ExecutionCommandPhase` | `dokime.host.v1.executions.ExecutionCommandPhase` |
| `dokime.client.v1.executions.ExecutionCommandReceipt` | `dokime.host.v1.executions.ExecutionCommandReceipt` |
| `dokime.client.v1.executions.CommandsGetRequest` | `dokime.host.v1.executions.CommandsGetRequest` |
| `dokime.client.v1.executions.CommandsGetResponse` | `dokime.host.v1.executions.CommandsGetResponse` |
| `dokime.client.v1.executions.CommandsCreateRequest` | `dokime.host.v1.executions.CommandsCreateRequest` |
| `dokime.client.v1.executions.CommandsCreateResponse` | `dokime.host.v1.executions.CommandsCreateResponse` |
| `dokime.client.v1.executions.ProgressGetRequest` | `dokime.host.v1.executions.ProgressGetRequest` |
| `dokime.client.v1.executions.ProgressGetResponse` | `dokime.host.v1.executions.ProgressGetResponse` |
| `dokime.client.v1.executions.ProgressWatchRequest` | `dokime.host.v1.executions.ProgressWatchRequest` |
| `dokime.client.v1.executions.ProgressWatchResponse` | `dokime.host.v1.executions.ProgressWatchResponse` |
| `dokime.client.v1.executions.SchemasGetRequest` | `dokime.host.v1.executions.SchemasGetRequest` |
| `dokime.client.v1.executions.SchemasGetResponse` | `dokime.host.v1.executions.SchemasGetResponse` |
| `dokime.client.v1.FilesGetRequest` | `dokime.host.v1.FilesGetRequest` |
| `dokime.client.v1.FilesGetResponse` | `dokime.host.v1.FilesGetResponse` |
| `dokime.client.v1.FilesReadRequest` | `dokime.host.v1.FilesReadRequest` |
| `dokime.client.v1.FilesReadResponse` | `dokime.host.v1.FilesReadResponse` |
| `dokime.client.v1.FilesWatchRequest` | `dokime.host.v1.FilesWatchRequest` |
| `dokime.client.v1.FilesWatchResponse` | `dokime.host.v1.FilesWatchResponse` |
| `dokime.client.v1.FilesDeleteRequest` | `dokime.host.v1.FilesDeleteRequest` |
| `dokime.client.v1.FilesDeleteResponse` | `dokime.host.v1.FilesDeleteResponse` |
| `dokime.client.v1.files.UploadsCreateRequest` | `dokime.host.v1.files.UploadsCreateRequest` |
| `dokime.client.v1.files.UploadsCreateResponse` | `dokime.host.v1.files.UploadsCreateResponse` |
| `dokime.client.v1.files.UploadsGetRequest` | `dokime.host.v1.files.UploadsGetRequest` |
| `dokime.client.v1.files.UploadsGetResponse` | `dokime.host.v1.files.UploadsGetResponse` |
| `dokime.client.v1.files.UploadsResumeRequest` | `dokime.host.v1.files.UploadsResumeRequest` |
| `dokime.client.v1.files.UploadsResumeResponse` | `dokime.host.v1.files.UploadsResumeResponse` |
| `dokime.client.v1.files.UploadsAppendRequest` | `dokime.host.v1.files.UploadsAppendRequest` |
| `dokime.client.v1.files.UploadsAppendResponse` | `dokime.host.v1.files.UploadsAppendResponse` |
| `dokime.client.v1.files.UploadsPublishRequest` | `dokime.host.v1.files.UploadsPublishRequest` |
| `dokime.client.v1.files.UploadsPublishResponse` | `dokime.host.v1.files.UploadsPublishResponse` |
| `dokime.client.v1.files.UploadsDeleteRequest` | `dokime.host.v1.files.UploadsDeleteRequest` |
| `dokime.client.v1.files.UploadsDeleteResponse` | `dokime.host.v1.files.UploadsDeleteResponse` |
| `dokime.client.v1.media.RecordingAudio` | `dokime.host.v1.media.RecordingAudio` |
| `dokime.client.v1.media.RecordingProgress` | `dokime.host.v1.media.RecordingProgress` |
| `dokime.client.v1.media.CaptureKind` | `dokime.host.v1.media.CaptureKind` |
| `dokime.client.v1.media.Capture` | `dokime.host.v1.media.Capture` |
| `dokime.client.v1.media.CapturesCreateRequest` | `dokime.host.v1.media.CapturesCreateRequest` |
| `dokime.client.v1.media.CapturesCreateResponse` | `dokime.host.v1.media.CapturesCreateResponse` |
| `dokime.client.v1.media.CapturesGetRequest` | `dokime.host.v1.media.CapturesGetRequest` |
| `dokime.client.v1.media.CapturesGetResponse` | `dokime.host.v1.media.CapturesGetResponse` |
| `dokime.client.v1.media.CapturesWatchRequest` | `dokime.host.v1.media.CapturesWatchRequest` |
| `dokime.client.v1.media.CapturesWatchResponse` | `dokime.host.v1.media.CapturesWatchResponse` |
| `dokime.client.v1.media.CapturesStopRequest` | `dokime.host.v1.media.CapturesStopRequest` |
| `dokime.client.v1.media.CapturesStopResponse` | `dokime.host.v1.media.CapturesStopResponse` |
| `dokime.client.v1.plots.PlotSettingsValue` | `dokime.host.v1.plots.PlotSettingsValue` |
| `dokime.client.v1.plots.SettingsGetRequest` | `dokime.host.v1.plots.SettingsGetRequest` |
| `dokime.client.v1.plots.SettingsGetResponse` | `dokime.host.v1.plots.SettingsGetResponse` |
| `dokime.client.v1.plots.SettingsSetRequest` | `dokime.host.v1.plots.SettingsSetRequest` |
| `dokime.client.v1.plots.SettingsSetResponse` | `dokime.host.v1.plots.SettingsSetResponse` |
| `dokime.client.v1.plots.AnalysisComputation` | `dokime.host.v1.plots.AnalysisComputation` |
| `dokime.client.v1.plots.AnalysisEvidence` | `dokime.host.v1.plots.AnalysisEvidence` |
| `dokime.client.v1.plots.AnalysisSeries` | `dokime.host.v1.plots.AnalysisSeries` |
| `dokime.client.v1.plots.ContourGrid` | `dokime.host.v1.plots.ContourGrid` |
| `dokime.client.v1.plots.GridCell` | `dokime.host.v1.plots.GridCell` |
| `dokime.client.v1.plots.GridFrame` | `dokime.host.v1.plots.GridFrame` |
| `dokime.client.v1.plots.GridFrames` | `dokime.host.v1.plots.GridFrames` |
| `dokime.client.v1.plots.NullableNumber` | `dokime.host.v1.plots.NullableNumber` |
| `dokime.client.v1.plots.PlotAnalysis` | `dokime.host.v1.plots.PlotAnalysis` |
| `dokime.client.v1.plots.PlotAnalysisError` | `dokime.host.v1.plots.PlotAnalysisError` |
| `dokime.client.v1.plots.PlotAnalysisResult` | `dokime.host.v1.plots.PlotAnalysisResult` |
| `dokime.client.v1.plots.PlotBoxSummary` | `dokime.host.v1.plots.PlotBoxSummary` |
| `dokime.client.v1.plots.PlotBoxSummaryAnalysis` | `dokime.host.v1.plots.PlotBoxSummaryAnalysis` |
| `dokime.client.v1.plots.PlotCartesianMode` | `dokime.host.v1.plots.PlotCartesianMode` |
| `dokime.client.v1.plots.PlotColumn` | `dokime.host.v1.plots.PlotColumn` |
| `dokime.client.v1.plots.PlotDomain` | `dokime.host.v1.plots.PlotDomain` |
| `dokime.client.v1.plots.PlotEnvelope` | `dokime.host.v1.plots.PlotEnvelope` |
| `dokime.client.v1.plots.PlotFrame` | `dokime.host.v1.plots.PlotFrame` |
| `dokime.client.v1.plots.PlotHistogramAnalysis` | `dokime.host.v1.plots.PlotHistogramAnalysis` |
| `dokime.client.v1.plots.PlotHistogramBin` | `dokime.host.v1.plots.PlotHistogramBin` |
| `dokime.client.v1.plots.PlotHistogramSeries` | `dokime.host.v1.plots.PlotHistogramSeries` |
| `dokime.client.v1.plots.PlotHistogramStatistics` | `dokime.host.v1.plots.PlotHistogramStatistics` |
| `dokime.client.v1.plots.PlotMarker` | `dokime.host.v1.plots.PlotMarker` |
| `dokime.client.v1.plots.PlotPayload` | `dokime.host.v1.plots.PlotPayload` |
| `dokime.client.v1.plots.PlotPayloadDiagnostics` | `dokime.host.v1.plots.PlotPayloadDiagnostics` |
| `dokime.client.v1.plots.PlotSeries` | `dokime.host.v1.plots.PlotSeries` |
| `dokime.client.v1.plots.PlotSeriesSummary` | `dokime.host.v1.plots.PlotSeriesSummary` |
| `dokime.client.v1.plots.PlotSigmaBin` | `dokime.host.v1.plots.PlotSigmaBin` |
| `dokime.client.v1.plots.PlotSigmaIntervalAnalysis` | `dokime.host.v1.plots.PlotSigmaIntervalAnalysis` |
| `dokime.client.v1.plots.PlotSigmaSeries` | `dokime.host.v1.plots.PlotSigmaSeries` |
| `dokime.client.v1.plots.PlotStreamKind` | `dokime.host.v1.plots.PlotStreamKind` |
| `dokime.client.v1.plots.PlotStreamUpdate` | `dokime.host.v1.plots.PlotStreamUpdate` |
| `dokime.client.v1.plots.SnapshotsListRequest` | `dokime.host.v1.plots.SnapshotsListRequest` |
| `dokime.client.v1.plots.SnapshotsListResponse` | `dokime.host.v1.plots.SnapshotsListResponse` |
| `dokime.client.v1.plots.SnapshotsWatchRequest` | `dokime.host.v1.plots.SnapshotsWatchRequest` |
| `dokime.client.v1.plots.SnapshotsWatchResponse` | `dokime.host.v1.plots.SnapshotsWatchResponse` |
| `dokime.client.v1.ListPlugins` | `harmos.host.v1.PluginsListRequest` |
| `dokime.client.v1.PluginEntries` | `harmos.host.v1.PluginsListResponse` |
| `dokime.client.v1.AddRemotePlugin` | `harmos.host.v1.PluginsCreateRequest` |
| `dokime.client.v1.RemovePlugin` | `harmos.host.v1.PluginsDeleteRequest` |
| `dokime.client.v1.RemovePluginResponse` | `harmos.host.v1.PluginsDeleteResponse` |
| `dokime.client.v1.Plugin` | `harmos.host.v1.PluginEntry` |
| `dokime.client.v1.ManagedPlugin` | `harmos.host.v1.PluginManaged` |
| `dokime.client.v1.RemotePlugin` | `harmos.host.v1.PluginRemote` |
| `dokime.client.v1.PluginState` | `harmos.host.v1.PluginState` |
| `dokime.client.v1.AdmittedPlugin` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.AdmittedPluginCursor` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.AdmittedPluginPage` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.InstallPluginChunk` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.InstallPluginFrame` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.InstallPluginFrameFrame` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.InstallPluginHeader` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.InstallationCursor` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.InstallationPage` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.PluginIdentity` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.PluginInstallation` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.PluginInstallationState` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.PluginMetadata` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.PluginsGetInstallationRequest` | `harmos.host.v1.PluginsGetRequest` |
| `dokime.client.v1.PluginsInstallRequest` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.PluginsInstallInput` | `deleted; use harmos Plugins or plugins.Settings` |
| `dokime.client.v1.PluginsListAdmittedPluginsRequest` | `harmos.host.v1.PluginsListRequest` |
| `dokime.client.v1.PluginsListInstallationsRequest` | `harmos.host.v1.PluginsListRequest` |
| `dokime.client.v1.PluginsReloadRequest` | `harmos.host.v1.PluginsRestartRequest` |
| `dokime.client.v1.PluginsSetEnabledRequest` | `harmos.host.v1.PluginsSetRequest` |
| `dokime.client.v1.plugins.SecretAvailability` | `dokime.host.v1.plugins.SecretRequirement` |
| `dokime.client.v1.plugins.SettingsGetRequest` | `harmos.host.v1.plugins.SettingsListRequest` |
| `dokime.client.v1.plugins.SettingsGetResponse` | `harmos.host.v1.plugins.SettingsListResponse` |
| `dokime.client.v1.plugins.SettingsSetRequest` | `harmos.host.v1.plugins.SettingsSetRequest` |
| `dokime.client.v1.plugins.SettingsSetResponse` | `harmos.host.v1.plugins.SettingsSetResponse` |
| `dokime.client.v1.PromptDefinitionsGetRequest` | `dokime.host.v1.PromptDefinitionsGetRequest` |
| `dokime.client.v1.PromptDefinitionsGetResponse` | `dokime.host.v1.PromptDefinitionsGetResponse` |
| `dokime.client.v1.PromptDefinitionsListRequest` | `dokime.host.v1.PromptDefinitionsListRequest` |
| `dokime.client.v1.PromptDefinitionsListResponse` | `dokime.host.v1.PromptDefinitionsListResponse` |
| `dokime.client.v1.PromptInstance` | `dokime.host.v1.PromptInstance` |
| `dokime.client.v1.PromptKind` | `dokime.host.v1.PromptKind` |
| `dokime.client.v1.PromptView` | `dokime.host.v1.PromptView` |
| `dokime.client.v1.PromptsGetRequest` | `dokime.host.v1.PromptsGetRequest` |
| `dokime.client.v1.PromptsGetResponse` | `dokime.host.v1.PromptsGetResponse` |
| `dokime.client.v1.PromptsListRequest` | `dokime.host.v1.PromptsListRequest` |
| `dokime.client.v1.PromptsListResponse` | `dokime.host.v1.PromptsListResponse` |
| `dokime.client.v1.PromptsWatchRequest` | `dokime.host.v1.PromptsWatchRequest` |
| `dokime.client.v1.PromptsWatchResponse` | `dokime.host.v1.PromptsWatchResponse` |
| `dokime.client.v1.prompts.AnswersCreateRequest` | `dokime.host.v1.prompts.AnswersCreateRequest` |
| `dokime.client.v1.prompts.AnswersCreateResponse` | `dokime.host.v1.prompts.AnswersCreateResponse` |
| `dokime.client.v1.RecordSchemasGetRequest` | `dokime.host.v1.RecordSchemasGetRequest` |
| `dokime.client.v1.RecordSchemasGetResponse` | `dokime.host.v1.RecordSchemasGetResponse` |
| `dokime.client.v1.RecordSchemasListRequest` | `dokime.host.v1.RecordSchemasListRequest` |
| `dokime.client.v1.RecordSchemasListResponse` | `dokime.host.v1.RecordSchemasListResponse` |
| `dokime.client.v1.ReportMetadata` | `dokime.host.v1.ReportMetadata` |
| `dokime.client.v1.ReportOrigin` | `dokime.host.v1.ReportOrigin` |
| `dokime.client.v1.ReportSubject` | `dokime.host.v1.ReportSubject` |
| `dokime.client.v1.ReportsGetRequest` | `dokime.host.v1.ReportsGetRequest` |
| `dokime.client.v1.ReportsGetResponse` | `dokime.host.v1.ReportsGetResponse` |
| `dokime.client.v1.ReportsCreateRequest` | `dokime.host.v1.ReportsCreateRequest` |
| `dokime.client.v1.ReportsCreateResponse` | `dokime.host.v1.ReportsCreateResponse` |
| `dokime.client.v1.ReportsPublishRequest` | `dokime.host.v1.ReportsPublishRequest` |
| `dokime.client.v1.ReportsPublishResponse` | `dokime.host.v1.ReportsPublishResponse` |
| `dokime.client.v1.SecretArchive` | `dokime.host.v1.SecretArchive` |
| `dokime.client.v1.SecretImportAction` | `dokime.host.v1.SecretImportAction` |
| `dokime.client.v1.SecretImportChoice` | `dokime.host.v1.SecretImportChoice` |
| `dokime.client.v1.SecretImportEntry` | `dokime.host.v1.SecretImportEntry` |
| `dokime.client.v1.SecretImportPreview` | `dokime.host.v1.SecretImportPreview` |
| `dokime.client.v1.SecretSnapshot` | `dokime.host.v1.SecretSnapshot` |
| `dokime.client.v1.SecretSummary` | `dokime.host.v1.SecretSummary` |
| `dokime.client.v1.SecretWritable` | `dokime.host.v1.SecretWritable` |
| `dokime.client.v1.SecretsGetRequest` | `dokime.host.v1.SecretsGetRequest` |
| `dokime.client.v1.SecretsGetResponse` | `dokime.host.v1.SecretsGetResponse` |
| `dokime.client.v1.SecretsSetRequest` | `dokime.host.v1.SecretsSetRequest` |
| `dokime.client.v1.SecretsSetResponse` | `dokime.host.v1.SecretsSetResponse` |
| `dokime.client.v1.SecretsListRequest` | `dokime.host.v1.SecretsListRequest` |
| `dokime.client.v1.SecretsListResponse` | `dokime.host.v1.SecretsListResponse` |
| `dokime.client.v1.SecretsWatchRequest` | `dokime.host.v1.SecretsWatchRequest` |
| `dokime.client.v1.SecretsWatchResponse` | `dokime.host.v1.SecretsWatchResponse` |
| `dokime.client.v1.SecretsDeleteRequest` | `dokime.host.v1.SecretsDeleteRequest` |
| `dokime.client.v1.SecretsDeleteResponse` | `dokime.host.v1.SecretsDeleteResponse` |
| `dokime.client.v1.SecretsExportRequest` | `dokime.host.v1.SecretsExportRequest` |
| `dokime.client.v1.SecretsExportResponse` | `dokime.host.v1.SecretsExportResponse` |
| `dokime.client.v1.SecretsImportRequest` | `dokime.host.v1.SecretsImportRequest` |
| `dokime.client.v1.SecretsImportResponse` | `dokime.host.v1.SecretsImportResponse` |
| `dokime.client.v1.ApplicationAppearance` | `dokime.host.v1.ApplicationAppearance` |
| `dokime.client.v1.ApplicationSettings` | `dokime.host.v1.ApplicationSettings` |
| `dokime.client.v1.BootstrapPaths` | `dokime.host.v1.BootstrapPaths` |
| `dokime.client.v1.CachedRefresh` | `dokime.host.v1.CachedRefresh` |
| `dokime.client.v1.ClientPreferences` | `dokime.host.v1.ClientPreferences` |
| `dokime.client.v1.Density` | `dokime.host.v1.Density` |
| `dokime.client.v1.HostCardBlock` | `dokime.host.v1.HostCardBlock` |
| `dokime.client.v1.HostCardBlockKind` | `dokime.host.v1.HostCardBlockKind` |
| `dokime.client.v1.HostCardBody` | `dokime.host.v1.HostCardBody` |
| `dokime.client.v1.HostCardKeyValue` | `dokime.host.v1.HostCardKeyValue` |
| `dokime.client.v1.HostCardKeyValueList` | `dokime.host.v1.HostCardKeyValueList` |
| `dokime.client.v1.HostCardMetric` | `dokime.host.v1.HostCardMetric` |
| `dokime.client.v1.HostCardMetrics` | `dokime.host.v1.HostCardMetrics` |
| `dokime.client.v1.HostCardStatus` | `dokime.host.v1.HostCardStatus` |
| `dokime.client.v1.HostCardTone` | `dokime.host.v1.HostCardTone` |
| `dokime.client.v1.ManualRefresh` | `dokime.host.v1.ManualRefresh` |
| `dokime.client.v1.OnDemandRefresh` | `dokime.host.v1.OnDemandRefresh` |
| `dokime.client.v1.RemoteListenerSettings` | `dokime.host.v1.RemoteListenerSettings` |
| `dokime.client.v1.ReportDefaults` | `dokime.host.v1.ReportDefaults` |
| `dokime.client.v1.SettingsCard` | `dokime.host.v1.SettingsCard` |
| `dokime.client.v1.SettingsCardRefreshPolicy` | `dokime.host.v1.SettingsCardRefreshPolicy` |
| `dokime.client.v1.SettingsCardRefreshPolicyPolicy` | `dokime.host.v1.SettingsCardRefreshPolicyPolicy` |
| `dokime.client.v1.SettingsCardState` | `dokime.host.v1.SettingsCardState` |
| `dokime.client.v1.SettingsFrame` | `dokime.host.v1.SettingsFrame` |
| `dokime.client.v1.SettingsSection` | `dokime.host.v1.SettingsSection` |
| `dokime.client.v1.SettingsSectionOwner` | `dokime.host.v1.SettingsSectionOwner` |
| `dokime.client.v1.TelemetryPreferences` | `dokime.host.v1.TelemetryPreferences` |
| `dokime.client.v1.ThemeMode` | `dokime.host.v1.ThemeMode` |
| `dokime.client.v1.SettingsGetRequest` | `dokime.host.v1.SettingsGetRequest` |
| `dokime.client.v1.SettingsGetResponse` | `dokime.host.v1.SettingsGetResponse` |
| `dokime.client.v1.SettingsSetRequest` | `dokime.host.v1.SettingsSetRequest` |
| `dokime.client.v1.SettingsSetResponse` | `dokime.host.v1.SettingsSetResponse` |
| `dokime.client.v1.SettingsSchemaRequest` | `dokime.host.v1.SettingsSchemaRequest` |
| `dokime.client.v1.SettingsSchemaResponse` | `dokime.host.v1.SettingsSchemaResponse` |
| `dokime.client.v1.storage.StorageSource` | `dokime.host.v1.storage.StorageSource` |
| `dokime.client.v1.storage.StorageSources` | `dokime.host.v1.storage.StorageSources` |
| `dokime.client.v1.storage.SourcesGetRequest` | `dokime.host.v1.storage.SourcesGetRequest` |
| `dokime.client.v1.storage.SourcesGetResponse` | `dokime.host.v1.storage.SourcesGetResponse` |
| `dokime.client.v1.storage.SourcesSetRequest` | `dokime.host.v1.storage.SourcesSetRequest` |
| `dokime.client.v1.storage.SourcesSetResponse` | `dokime.host.v1.storage.SourcesSetResponse` |
| `dokime.client.v1.StreamSchemasGetRequest` | `dokime.host.v1.StreamSchemasGetRequest` |
| `dokime.client.v1.StreamSchemasGetResponse` | `dokime.host.v1.StreamSchemasGetResponse` |
| `dokime.client.v1.StreamSchemasListRequest` | `dokime.host.v1.StreamSchemasListRequest` |
| `dokime.client.v1.StreamSchemasListResponse` | `dokime.host.v1.StreamSchemasListResponse` |
| `dokime.client.v1.SuitesGetRequest` | `dokime.host.v1.SuitesGetRequest` |
| `dokime.client.v1.SuitesGetResponse` | `dokime.host.v1.SuitesGetResponse` |
| `dokime.client.v1.SuitesListRequest` | `dokime.host.v1.SuitesListRequest` |
| `dokime.client.v1.SuitesListResponse` | `dokime.host.v1.SuitesListResponse` |
| `dokime.plugin.host.v1.BackendEnvironmentRequest` | `dokime.host.v1.BackendEnvironmentRequest` |
| `dokime.plugin.host.v1.BackendEnvironmentResponse` | `dokime.host.v1.BackendEnvironmentResponse` |
| `dokime.plugin.host.v1.content.ArtifactsCreateRequest` | `dokime.host.v1.content.ArtifactsCreateRequest` |
| `dokime.plugin.host.v1.content.ArtifactsCreateResponse` | `dokime.host.v1.content.ArtifactsCreateResponse` |
| `dokime.plugin.host.v1.DocumentsAppendPartsRequest` | `dokime.host.v1.DocumentsAppendPartsRequest` |
| `dokime.plugin.host.v1.DocumentsAppendPartsResponse` | `dokime.host.v1.DocumentsAppendPartsResponse` |
| `dokime.plugin.host.v1.EffectWritable` | `dokime.host.v1.EffectWritable` |
| `dokime.plugin.host.v1.EffectsCreateRequest` | `dokime.host.v1.EffectsCreateRequest` |
| `dokime.plugin.host.v1.EffectsCreateResponse` | `dokime.host.v1.EffectsCreateResponse` |
| `dokime.plugin.host.v1.EffectsGetRequest` | `dokime.host.v1.EffectsGetRequest` |
| `dokime.plugin.host.v1.EffectsGetResponse` | `dokime.host.v1.EffectsGetResponse` |
| `dokime.plugin.host.v1.EffectsSetRequest` | `dokime.host.v1.EffectsSetRequest` |
| `dokime.plugin.host.v1.EffectsSetResponse` | `dokime.host.v1.EffectsSetResponse` |
| `dokime.plugin.host.v1.EvidenceStatisticsRequest` | `dokime.host.v1.EvidenceStatisticsRequest` |
| `dokime.plugin.host.v1.EvidenceStatisticsResponse` | `dokime.host.v1.EvidenceStatisticsResponse` |
| `dokime.plugin.host.v1.evidence.LogsAppendRequest` | `dokime.host.v1.evidence.LogsAppendRequest` |
| `dokime.plugin.host.v1.evidence.LogsAppendResponse` | `dokime.host.v1.evidence.LogsAppendResponse` |
| `dokime.plugin.host.v1.evidence.Record` | `dokime.host.v1.evidence.CapturedRecord` |
| `dokime.plugin.host.v1.evidence.RecordsAppendRequest` | `dokime.host.v1.evidence.RecordsAppendRequest` |
| `dokime.plugin.host.v1.evidence.RecordsAppendResponse` | `dokime.host.v1.evidence.RecordsAppendResponse` |
| `dokime.plugin.host.v1.evidence.RecordsListRequest` | `dokime.host.v1.evidence.RecordsListRequest` |
| `dokime.plugin.host.v1.evidence.RecordsListResponse` | `dokime.host.v1.evidence.RecordsListResponse` |
| `dokime.plugin.host.v1.evidence.TraceabilityGetRequest` | `dokime.host.v1.evidence.TraceabilityGetRequest` |
| `dokime.plugin.host.v1.evidence.TraceabilityGetResponse` | `dokime.host.v1.evidence.TraceabilityGetResponse` |
| `dokime.plugin.host.v1.evidence.TraceabilitySetRequest` | `dokime.host.v1.evidence.TraceabilitySetRequest` |
| `dokime.plugin.host.v1.evidence.TraceabilitySetResponse` | `dokime.host.v1.evidence.TraceabilitySetResponse` |
| `dokime.plugin.host.v1.executions.ProgressGetRequest` | `dokime.host.v1.executions.ProgressGetRequest` |
| `dokime.plugin.host.v1.executions.ProgressGetResponse` | `dokime.host.v1.executions.ProgressGetResponse` |
| `dokime.plugin.host.v1.executions.ProgressSetRequest` | `dokime.host.v1.executions.ProgressSetRequest` |
| `dokime.plugin.host.v1.executions.ProgressSetResponse` | `dokime.host.v1.executions.ProgressSetResponse` |
| `dokime.plugin.host.v1.FilesGetRequest` | `dokime.host.v1.FilesGetRequest` |
| `dokime.plugin.host.v1.FilesGetResponse` | `dokime.host.v1.FilesGetResponse` |
| `dokime.plugin.host.v1.FilesReadRequest` | `dokime.host.v1.FilesReadRequest` |
| `dokime.plugin.host.v1.FilesReadResponse` | `dokime.host.v1.FilesReadResponse` |
| `dokime.plugin.host.v1.FilesWatchRequest` | `dokime.host.v1.FilesWatchRequest` |
| `dokime.plugin.host.v1.FilesWatchResponse` | `dokime.host.v1.FilesWatchResponse` |
| `dokime.plugin.host.v1.FilesDeleteRequest` | `dokime.host.v1.FilesDeleteRequest` |
| `dokime.plugin.host.v1.FilesDeleteResponse` | `dokime.host.v1.FilesDeleteResponse` |
| `dokime.plugin.host.v1.FilesCaptureRequest` | `dokime.host.v1.FilesCaptureRequest` |
| `dokime.plugin.host.v1.FilesCaptureResponse` | `dokime.host.v1.FilesCaptureResponse` |
| `dokime.plugin.host.v1.files.CaptureMode` | `dokime.host.v1.files.CaptureMode` |
| `dokime.plugin.host.v1.files.StreamCapture` | `dokime.host.v1.files.StreamCapture` |
| `dokime.plugin.host.v1.files.MirrorCapture` | `dokime.host.v1.files.MirrorCapture` |
| `dokime.plugin.host.v1.files.LineMatcher` | `dokime.host.v1.files.LineMatcher` |
| `dokime.plugin.host.v1.files.LineField` | `dokime.host.v1.files.LineField` |
| `dokime.plugin.host.v1.files.LineMarker` | `dokime.host.v1.files.LineMarker` |
| `dokime.plugin.host.v1.files.CapturesStartRequest` | `dokime.host.v1.files.CapturesStartRequest` |
| `dokime.plugin.host.v1.files.CapturesStartResponse` | `dokime.host.v1.files.CapturesStartResponse` |
| `dokime.plugin.host.v1.files.CapturesStopRequest` | `dokime.host.v1.files.CapturesStopRequest` |
| `dokime.plugin.host.v1.files.CapturesStopResponse` | `dokime.host.v1.files.CapturesStopResponse` |
| `dokime.plugin.host.v1.files.UploadsCreateRequest` | `dokime.host.v1.files.UploadsCreateRequest` |
| `dokime.plugin.host.v1.files.UploadsCreateResponse` | `dokime.host.v1.files.UploadsCreateResponse` |
| `dokime.plugin.host.v1.files.UploadsGetRequest` | `dokime.host.v1.files.UploadsGetRequest` |
| `dokime.plugin.host.v1.files.UploadsGetResponse` | `dokime.host.v1.files.UploadsGetResponse` |
| `dokime.plugin.host.v1.files.UploadsResumeRequest` | `dokime.host.v1.files.UploadsResumeRequest` |
| `dokime.plugin.host.v1.files.UploadsResumeResponse` | `dokime.host.v1.files.UploadsResumeResponse` |
| `dokime.plugin.host.v1.files.UploadsAppendRequest` | `dokime.host.v1.files.UploadsAppendRequest` |
| `dokime.plugin.host.v1.files.UploadsAppendResponse` | `dokime.host.v1.files.UploadsAppendResponse` |
| `dokime.plugin.host.v1.files.UploadsPublishRequest` | `dokime.host.v1.files.UploadsPublishRequest` |
| `dokime.plugin.host.v1.files.UploadsPublishResponse` | `dokime.host.v1.files.UploadsPublishResponse` |
| `dokime.plugin.host.v1.files.UploadsDeleteRequest` | `dokime.host.v1.files.UploadsDeleteRequest` |
| `dokime.plugin.host.v1.files.UploadsDeleteResponse` | `dokime.host.v1.files.UploadsDeleteResponse` |
| `dokime.plugin.host.v1.PromptsCreateRequest` | `dokime.host.v1.PromptsCreateRequest` |
| `dokime.plugin.host.v1.PromptsCreateResponse` | `dokime.host.v1.PromptsCreateResponse` |
| `dokime.plugin.host.v1.ReadSecret` | `dokime.host.v1.SecretsGetRequest` |
| `dokime.plugin.host.v1.Secret` | `dokime.host.v1.SecretsGetResponse` |
| `dokime.plugin.host.v1.ReadSettings` | `harmos.host.v1.plugins.SettingsListRequest` |
| `dokime.plugin.host.v1.SavedSettings` | `harmos.host.v1.plugins.SettingsListResponse` |
| `dokime.plugin.v1.CriteriaEvaluateRequest` | `dokime.plugin.v1.CriteriaEvaluateRequest` |
| `dokime.plugin.v1.CriteriaEvaluateResponse` | `dokime.plugin.v1.CriteriaEvaluateResponse` |
| `dokime.plugin.v1.ExecutionsInitializeRequest` | `dokime.plugin.v1.ExecutionsInitializeRequest` |
| `dokime.plugin.v1.ExecutionsInitializeResponse` | `dokime.plugin.v1.ExecutionsInitializeResponse` |
| `dokime.plugin.v1.ExecutionsCleanupRequest` | `dokime.plugin.v1.ExecutionsCleanupRequest` |
| `dokime.plugin.v1.ExecutionsCleanupResponse` | `dokime.plugin.v1.ExecutionsCleanupResponse` |
| `dokime.plugin.v1.StepsExecuteRequest` | `dokime.plugin.v1.StepsExecuteRequest` |
| `dokime.plugin.v1.StepsExecuteResponse` | `dokime.plugin.v1.StepsExecuteResponse` |
| `dokime.plugin.v1.TestCasesMaterializeRequest` | `dokime.plugin.v1.TestCasesMaterializeRequest` |
| `dokime.plugin.v1.TestCasesMaterializeResponse` | `dokime.plugin.v1.TestCasesMaterializeResponse` |
| `dokime.v1.Analyses` | `dokime.model.v1.Analyses` |
| `dokime.v1.Analysis` | `dokime.model.v1.Analysis` |
| `dokime.v1.AnalysisOutput` | `dokime.model.v1.AnalysisOutput` |
| `dokime.v1.AnalysisRequest` | `dokime.plugin.v1.AnalysesComputeRequest` |
| `dokime.v1.AnalysisSamples` | `dokime.plugin.v1.AnalysisSamples` |
| `dokime.v1.AnalysisResponse` | `dokime.plugin.v1.AnalysesComputeResponse` |
| `dokime.v1.AnalysisSelection` | `dokime.model.v1.AnalysisSelection` |
| `dokime.v1.AnalysisValues` | `dokime.model.v1.AnalysisValues` |
| `dokime.v1.AppearanceAssetReference` | `dokime.host.v1.AppearanceAssetReference` |
| `dokime.v1.AppearanceAssetMetadata` | `dokime.host.v1.AppearanceAssetMetadata` |
| `dokime.v1.ArchiveScope` | `dokime.host.v1.ArchiveScope` |
| `dokime.v1.CyclePhase` | `dokime.host.v1.CyclePhase` |
| `dokime.v1.CycleStatus` | `dokime.host.v1.CycleStatus` |
| `dokime.v1.CycleSummary` | `dokime.host.v1.CycleSummary` |
| `dokime.v1.Document` | `dokime.model.v1.Document` |
| `dokime.v1.Section` | `dokime.model.v1.Section` |
| `dokime.v1.SectionRepeat` | `dokime.model.v1.SectionRepeat` |
| `dokime.v1.Card` | `dokime.model.v1.Card` |
| `dokime.v1.CardLayout` | `dokime.model.v1.CardLayout` |
| `dokime.v1.Table` | `dokime.model.v1.Table` |
| `dokime.v1.TableRow` | `dokime.model.v1.TableRow` |
| `dokime.v1.Summary` | `dokime.model.v1.Summary` |
| `dokime.v1.SummaryTile` | `dokime.model.v1.SummaryTile` |
| `dokime.v1.EvidenceCard` | `dokime.model.v1.EvidenceCard` |
| `dokime.v1.DocumentScope` | `dokime.host.v1.DocumentScope` |
| `dokime.v1.ClientEmpty` | `deleted` |
| `dokime.v1.Severity` | `dokime.model.v1.Severity` |
| `dokime.v1.EvidenceCondition` | `dokime.model.v1.EvidenceCondition` |
| `dokime.v1.EvidenceConditions` | `dokime.model.v1.EvidenceConditions` |
| `dokime.v1.RecordMatch` | `dokime.model.v1.RecordMatch` |
| `dokime.v1.StreamMatch` | `dokime.model.v1.StreamMatch` |
| `dokime.v1.StreamMatchOperator` | `dokime.model.v1.StreamMatchOperator` |
| `dokime.v1.ArtifactMetadata` | `dokime.model.v1.ArtifactMetadata` |
| `dokime.v1.ArtifactScope` | `dokime.model.v1.ArtifactScope` |
| `dokime.v1.ArtifactState` | `dokime.model.v1.ArtifactState` |
| `dokime.v1.EvidenceOrigin` | `dokime.model.v1.EvidenceOrigin` |
| `dokime.v1.EvidenceOriginKind` | `dokime.model.v1.EvidenceOriginKind` |
| `dokime.v1.EvidenceScope` | `dokime.model.v1.EvidenceScope` |
| `dokime.v1.Window` | `dokime.model.v1.Window` |
| `dokime.v1.TestCase` | `dokime.model.v1.TestCase` |
| `dokime.v1.Origin` | `dokime.model.v1.Origin` |
| `dokime.v1.PluginRun` | `dokime.model.v1.PluginRun` |
| `dokime.v1.Import` | `dokime.model.v1.Import` |
| `dokime.v1.ImportSource` | `dokime.model.v1.ImportSource` |
| `dokime.v1.Step` | `dokime.model.v1.Step` |
| `dokime.v1.Criterion` | `dokime.model.v1.Criterion` |
| `dokime.v1.Termination` | `dokime.model.v1.Termination` |
| `dokime.v1.ExecutionProgressValue` | `dokime.model.v1.ExecutionProgressValue` |
| `dokime.v1.ExecutionState` | `dokime.host.v1.ExecutionState` |
| `dokime.v1.ExecutionOutcome` | `dokime.host.v1.ExecutionOutcome` |
| `dokime.v1.UnevaluatedOutcome` | `dokime.host.v1.UnevaluatedOutcome` |
| `dokime.v1.EvaluatedOutcome` | `dokime.host.v1.EvaluatedOutcome` |
| `dokime.v1.DeclaredOutcome` | `dokime.host.v1.DeclaredOutcome` |
| `dokime.v1.ResultStatus` | `dokime.host.v1.ResultStatus` |
| `dokime.v1.ResultSummary` | `dokime.host.v1.ResultSummary` |
| `dokime.v1.DataExpression` | `dokime.model.v1.DataExpression` |
| `dokime.v1.FieldReference` | `dokime.model.v1.FieldReference` |
| `dokime.v1.AggregateExpression` | `dokime.model.v1.AggregateExpression` |
| `dokime.v1.Aggregate` | `dokime.model.v1.Aggregate` |
| `dokime.v1.TransformExpression` | `dokime.model.v1.TransformExpression` |
| `dokime.v1.DocumentFilter` | `dokime.model.v1.DocumentFilter` |
| `dokime.v1.FilterOperator` | `dokime.model.v1.FilterOperator` |
| `dokime.v1.FileSource` | `dokime.model.v1.FileSource` |
| `dokime.v1.FileLocation` | `dokime.model.v1.FileLocation` |
| `dokime.v1.FtpLocation` | `dokime.model.v1.FtpLocation` |
| `dokime.v1.FileSelection` | `dokime.model.v1.FileSelection` |
| `dokime.v1.FileRule` | `dokime.model.v1.FileRule` |
| `dokime.v1.FileRuleAction` | `dokime.model.v1.FileRuleAction` |
| `dokime.v1.CapturedArtifact` | `dokime.model.v1.CapturedArtifact` |
| `dokime.v1.CaptureGap` | `dokime.model.v1.CaptureGap` |
| `dokime.v1.CaptureGapReason` | `dokime.model.v1.CaptureGapReason` |
| `dokime.v1.AbsoluteTimeFormat` | `dokime.host.v1.AbsoluteTimeFormat` |
| `dokime.v1.ElapsedFormat` | `dokime.host.v1.ElapsedFormat` |
| `dokime.v1.ElapsedUnit` | `dokime.host.v1.ElapsedUnit` |
| `dokime.v1.EngineeringFormat` | `dokime.host.v1.EngineeringFormat` |
| `dokime.v1.FixedFormat` | `dokime.host.v1.FixedFormat` |
| `dokime.v1.MeasurementUnit` | `dokime.host.v1.MeasurementUnit` |
| `dokime.v1.NormalFormat` | `dokime.host.v1.NormalFormat` |
| `dokime.v1.ScientificFormat` | `dokime.host.v1.ScientificFormat` |
| `dokime.v1.SiPrefixFormat` | `dokime.host.v1.SiPrefixFormat` |
| `dokime.v1.UnitFamily` | `dokime.host.v1.UnitFamily` |
| `dokime.v1.ValueFormat` | `dokime.host.v1.ValueFormat` |
| `dokime.v1.ValueFormatKind` | `dokime.host.v1.ValueFormatKind` |
| `dokime.v1.Metadata` | `dokime.model.v1.Metadata` |
| `dokime.v1.AnnotationOwner` | `dokime.host.v1.evidence.AnnotationOwner` |
| `dokime.v1.AnnotationOwnerOwner` | `dokime.host.v1.evidence.AnnotationOwnerOwner` |
| `dokime.v1.AnnotationSelection` | `dokime.host.v1.evidence.AnnotationSelection` |
| `dokime.v1.ExecutionNoteOwner` | `dokime.host.v1.evidence.ExecutionNoteOwner` |
| `dokime.v1.PlannedCaseNoteOwner` | `dokime.host.v1.evidence.PlannedCaseNoteOwner` |
| `dokime.v1.Parameter` | `dokime.model.v1.Parameter` |
| `dokime.v1.ParameterExpansion` | `dokime.model.v1.ParameterExpansion` |
| `dokime.v1.ParameterExpression` | `dokime.model.v1.ParameterExpression` |
| `dokime.v1.ParameterExpressions` | `dokime.model.v1.ParameterExpressions` |
| `dokime.v1.IntegerRange` | `dokime.model.v1.IntegerRange` |
| `dokime.v1.NumberRange` | `dokime.model.v1.NumberRange` |
| `dokime.v1.LinearSpace` | `dokime.model.v1.LinearSpace` |
| `dokime.v1.DynamicDefault` | `dokime.model.v1.DynamicDefault` |
| `dokime.v1.DynamicDefaultKey` | `dokime.model.v1.DynamicDefaultKey` |
| `dokime.v1.ParameterUnary` | `dokime.model.v1.ParameterUnary` |
| `dokime.v1.ParameterUnaryOperator` | `dokime.model.v1.ParameterUnaryOperator` |
| `dokime.v1.ParameterBinary` | `dokime.model.v1.ParameterBinary` |
| `dokime.v1.ParameterBinaryOperator` | `dokime.model.v1.ParameterBinaryOperator` |
| `dokime.v1.FileParameter` | `dokime.model.v1.FileParameter` |
| `dokime.v1.Plot` | `dokime.model.v1.Plot` |
| `dokime.v1.PlotStyle` | `dokime.model.v1.PlotStyle` |
| `dokime.v1.AxisScale` | `dokime.model.v1.AxisScale` |
| `dokime.v1.Reduction` | `dokime.model.v1.Reduction` |
| `dokime.v1.ContourRender` | `dokime.model.v1.ContourRender` |
| `dokime.v1.BarArrangement` | `dokime.model.v1.BarArrangement` |
| `dokime.v1.XyInterval` | `dokime.model.v1.XyInterval` |
| `dokime.v1.GridAxisEntry` | `dokime.model.v1.GridAxisEntry` |
| `dokime.v1.GridHighlight` | `dokime.model.v1.GridHighlight` |
| `dokime.v1.GridChart` | `dokime.model.v1.GridChart` |
| `dokime.v1.ReferenceLine` | `dokime.model.v1.ReferenceLine` |
| `dokime.v1.ReferenceBand` | `dokime.model.v1.ReferenceBand` |
| `dokime.v1.PlotKind` | `dokime.model.v1.PlotKind` |
| `dokime.v1.Downsample` | `dokime.model.v1.Downsample` |
| `dokime.v1.FftDetrend` | `dokime.model.v1.FftDetrend` |
| `dokime.v1.FftWindow` | `dokime.model.v1.FftWindow` |
| `dokime.v1.FrequencyLevelUnit` | `dokime.model.v1.FrequencyLevelUnit` |
| `dokime.v1.GridDefinition` | `dokime.model.v1.GridDefinition` |
| `dokime.v1.LineStyle` | `dokime.model.v1.LineStyle` |
| `dokime.v1.PlotFftOptions` | `dokime.model.v1.PlotFftOptions` |
| `dokime.v1.PlotFilter` | `dokime.model.v1.PlotFilter` |
| `dokime.v1.PlotHistogramOptions` | `dokime.model.v1.PlotHistogramOptions` |
| `dokime.v1.PlotIntervalOptions` | `dokime.model.v1.PlotIntervalOptions` |
| `dokime.v1.PlotRequest` | `dokime.host.v1.plots.PlotRequest` |
| `dokime.v1.PlotWindow` | `dokime.host.v1.plots.PlotWindow` |
| `dokime.v1.PromptDefinitions` | `dokime.model.v1.PromptDefinitions` |
| `dokime.v1.Prompt` | `dokime.model.v1.Prompt` |
| `dokime.v1.PromptValues` | `dokime.model.v1.PromptValues` |
| `dokime.v1.PromptArchetype` | `dokime.model.v1.PromptArchetype` |
| `dokime.v1.PromptField` | `dokime.model.v1.PromptField` |
| `dokime.v1.PromptFieldValidation` | `dokime.model.v1.PromptFieldValidation` |
| `dokime.v1.PromptFieldKind` | `dokime.model.v1.PromptFieldKind` |
| `dokime.v1.PromptAction` | `dokime.model.v1.PromptAction` |
| `dokime.v1.PromptActionEmphasis` | `dokime.model.v1.PromptActionEmphasis` |
| `dokime.v1.AnswerRule` | `dokime.model.v1.AnswerRule` |
| `dokime.v1.AnswerPredicate` | `dokime.model.v1.AnswerPredicate` |
| `dokime.v1.AnswerBinaryPredicate` | `dokime.model.v1.AnswerBinaryPredicate` |
| `dokime.v1.AnswerOperator` | `dokime.model.v1.AnswerOperator` |
| `dokime.v1.PromptResponse` | `dokime.host.v1.PromptResponse` |
| `dokime.v1.RecordContracts` | `dokime.model.v1.RecordContracts` |
| `dokime.v1.RecordContract` | `dokime.model.v1.RecordContract` |
| `dokime.v1.Listeners` | `dokime.model.v1.Listeners` |
| `dokime.v1.RecordListener` | `dokime.model.v1.RecordListener` |
| `dokime.v1.Value` | `dokime.model.v1.Value` |
| `harmos.storage.v1.ExactNumber` (former Value.exact payload) | `dokime.model.v1.ExactNumber` |
| `dokime.v1.Null` | `dokime.model.v1.Null` |
| `dokime.v1.ValueList` | `dokime.model.v1.ValueList` |
| `dokime.v1.ValueType` | `dokime.model.v1.ValueType` |
| `dokime.v1.MapValue` | `dokime.model.v1.MapValue` |
| `dokime.v1.Field` | `dokime.host.v1.Field` |
| `dokime.v1.FieldType` | `dokime.host.v1.FieldType` |
| `dokime.v1.RecordDisplay` | `dokime.host.v1.RecordDisplay` |
| `dokime.v1.RecordField` | `dokime.host.v1.RecordField` |
| `dokime.v1.RecordIcon` | `dokime.host.v1.RecordIcon` |
| `dokime.v1.RecordSchema` | `dokime.host.v1.RecordSchema` |
| `dokime.v1.RecordTone` | `dokime.host.v1.RecordTone` |
| `dokime.v1.Schema` | `dokime.host.v1.Schema` |
| `dokime.v1.TelemetryColumn` | `dokime.host.v1.TelemetryColumn` |
| `dokime.v1.TelemetryRole` | `dokime.host.v1.TelemetryRole` |
| `dokime.v1.TelemetrySchema` | `dokime.host.v1.TelemetrySchema` |
| `dokime.v1.ValidationMode` | `dokime.host.v1.ValidationMode` |
| `dokime.v1.SettingsDeclarations` | `harmos.model.v1.SettingsDeclarations` |
| `dokime.v1.Setting` | `harmos.model.v1.SettingDeclaration` |
| `dokime.v1.BooleanSetting` | `deleted; use harmos.model.v1.SettingDeclaration` |
| `dokime.v1.TextSetting` | `deleted; use harmos.model.v1.SettingDeclaration` |
| `dokime.v1.PathSetting` | `deleted; use harmos.model.v1.SettingDeclaration` |
| `dokime.v1.NumberSetting` | `deleted; use harmos.model.v1.SettingDeclaration` |
| `dokime.v1.ChoiceSetting` | `deleted; use harmos.model.v1.SettingDeclaration` |
| `dokime.v1.SettingChoice` | `deleted; use harmos.model.v1.SettingDeclaration` |
| `dokime.v1.SecretSetting` | `dokime.model.v1.SecretDeclaration` |
| `dokime.v1.RunStart` | `deleted` |
| `dokime.v1.Suites` | `dokime.model.v1.Suites` |
| `dokime.v1.Suite` | `dokime.model.v1.Suite` |
| `dokime.v1.TestCaseTemplate` | `dokime.model.v1.TestCaseTemplate` |
| `dokime.v1.TraceabilityEntry` | `dokime.model.v1.TraceabilityEntry` |

### Library Composition

The runtime and generators read [protocol/composition.toml](../protocol/composition.toml).
The descriptor artifact includes the complete vendored library plus product schema;
package selection composes only the listed library roots and their nested packages.
The runtime validates linked declarations and never opens generator descriptor paths.

```toml
[[libraries]]
descriptors = "../crates/dokime-protocol/src/descriptors.bin"
packages = ["harmos.model.v1", "harmos.host.v1", "harmos.plugin.v1"]

[callers]
"harmos.host.v1.Operations/List" = "CLIENTS"
"harmos.host.v1.Operations/Get" = "CLIENTS"
```

List is narrowed because host-wide catalog browsing is a client capability. Get is
narrowed because a plugin receives its admitted requirement contracts from its host
and need not retrieve arbitrary host contracts. Invocations stays available to both
callers for admitted child/peer calls; Time and a plugin's own plugins.Settings.List
also remain available to both. Existing library CLIENTS-only administration is
unchanged. No library operation is widened or shadowed by a product declaration.

### Generation and Public Barrels

`mise run generate -- --protocol` owns the protocol crate, composed resource table,
all four SDK surfaces, private Python wire modules and C# wire output. Dokime owns
`tools/protocol/src/protocol.rs`, `tools/python/src/dokime_tools/build/{protocol,
protocol_callers,resource_roots,render_resources,csharp_protocol}.py` and
`sdk/python/tools/generate.py`; none imports or executes library tooling.
C++ wire messages/stubs are produced by its own CMake generation rule at build time.
Root model names take precedence; conflicting library value names receive a Harmos
prefix in the composed barrel (for example HarmosSection).

Client resources come from CLIENTS/ALL, plugin context resources from PLUGINS/ALL.
Plugin executor services supply lifecycle contracts, not host resources. Request
content becomes arguments, calls own transport context and optional overrides, and
one-field responses unwrap their payload (including stream items). Empty and
multi-field replies retain their response type. Optional at/page select defaults;
mutation request identities are generated once when omitted. Public roots are
`dokime`, `dokime::client`, `dokime::plugin` and language equivalents, with no public
v1 path. Raw protocol modules are private SDK implementation details. Nested
resource payload names gain their owner prefix where a flat barrel would collide.
The composed root includes `client.plugins.list()` beside
`client.executions.create(...)` and `client.plugins.secrets.list(...)`.

Rust keeps required selectors positional and places defaultable document/evidence
filters in each call's generated Options value. Client options can be cloned for
replay without regenerating their mutation identity. Python's strict stub check
verifies PEP 695 alias values and the generic callable's declared type parameters;
generated-server checks also compare canonical registered gRPC service names.

### Definition Repair Call Shapes

`Artifacts.Create` is ALL; its plugin scope remains optional and trusted by the host.
`Files.Capture(source, output)` is PLUGINS and now has an RPC. Every mapped RPC
exists and its caller annotation matches the mapping above, including the six
harmonized operations in item 0.M. Model values and host replies used by both
callers have one root barrel home. In Rust and Python this moves
EvidenceStatisticsResponse, EvidenceRecordsListResponse, EvidenceRecordScope,
EvidenceCapturedRecordSelection and ContentArtifactsCreateResponse out of their
caller-specific barrels. Use the generated name rather than adding an alias.
C# uses the field's canonical namespace; C++ exports AnalysisSamples under plugin.
The severity value at wire number 4 is now SEVERITY_FAILURE (Rust Severity::Failure,
C# Severity.Failure), avoiding the Windows SDK's SEVERITY_ERROR macro. C++ reserved
operation words become `delete_`, `export_` and `import_` on generated methods.

Python construction is `dokime.client.Client`; plugin authoring is
`dokime.plugin.{Plugin,Context}`. Both surfaces compose only their effective callers;
no inheritance restores operations removed by narrowing. The client implementation
lives under `_generated/client.py`; the retired `generated` tree and resources.json
have no producer. Generated Python passes the authored lint rules and typing;
only protoc's private `_pb2*` files retain the existing exclusion, while their public
barrels and generation consumer are type-checked.

The pinned library serves Python plugins on their canonical gRPC paths. Runtime
consumers must use `Failure::wrong_caller(operation)` for every wrong-caller path:
classification refused, code "-32604", action "Call". No product failure twin is needed.

`mise run check:generated` compiles the five permanent consumers independently of
operations/runtime; `--language rust|python|csharp|cpp|typescript` selects one.
Each SDK also exposes `test:generation`. These are compiler contracts, with content
calls through the public client and plugin barrels, not real-host behavior proof.
C# excludes the authored FileTransfers.cs and writes under target/cs/generation;
its two remaining errors (nullable deadline and ambiguous File) belong to the C#
consumer. C++ compiles both the consumer and generated host implementation as an
object target. The desktop consumer compiles its generated client with the existing
transport seam, without importing product views.

### Handler Interfaces

The common assembly owner owns `host::Context`, `Resources`, `host::Host` and
registration. Canonical host handlers keep `handle(&Context, Request)`; operations
restricted to plugin invocations can take `&Resources`. Both direct client calls
and child dispatch call the same canonical handler. `Context.caller` is `None` for
clients and contains the trusted `Resources` for plugins; never construct identity
from a request. Use `context.resolve_execution(selector, ID)` before storage reads.
`Resources.context` retains the client-independent assembly context and child
dispatch clones it with the trusted caller attached.

`Context.operations` is the composed library `OperationManager`.
`Context.storage::<O>(request)` invokes its native operation, retaining the library
callee policy and action/cause on failure. Import native declarations and requests
from `harmos_runtime::builtins::storage`. Files and uploads use native operations.
Compound execution metadata keeps the product Store transactions described in
Storage Paths After This Wave.

Each supported handler declares its native adapter in its operation source file.
Assembly derives caller restrictions from the frozen service descriptors before
applying the composition restrictions. The library serves direct gRPC and encoded
Invocations calls from those registrations; implemented product resources have no
second handwritten transport service. Native failures retain their action, status
and source chain. Direct hosts use
`Host::open(root, host_settings, operation_settings).await`; served hosts share that
assembly. Assembly owns the backend lease, and closure refuses further backend
calls before releasing it.

Files.Capture and files.Captures.Start/Stop register through those canonical
adapters, enforce their plugin-only callers and run the local capture engine in
`crates/dokime-operations/src/host/files/captures/`. A source must resolve, after
following links, inside one of the host's `capture_roots` (`--capture-root` on
`dokime-host`); an empty list refuses every capture with a permission failure.
Files.Capture copies its File, Tree or Newest source once. Captures.Start takes
the baseline and returns its identifier; the calling execution owns the capture.
Captures.Stop takes the final pass and seals once: Stream keeps bytes written after
the baseline (later files whole), Mirror keeps each file whole as it is at the stop.
A repeated stop returns the same outcome, and an abandoned reply leaves sealing
running. Bytes travel through the native upload path and Artifacts.Create; gaps
and early failures append `capture:gap` and `capture:failure` evidence. A capture
nothing stopped takes its final pass when the execution's binding closes, before
its evidence closes, including after cancellation; no capture outlives its
execution. File, byte and capacity limits come from host settings. Interim passes,
live growing artifacts and Stream line matchers or markers are not restored;
matchers, markers and FTP/FTPS sources refuse as deferred, naming the action.
The same registration owns the unsupported Effects, media.Captures and Aggregates
operations. Direct and encoded calls share their refusals; no handwritten runtime
service duplicates those declarations.

Uploads.Create assigns an omitted file ID from its mutation identity. Plugin
mutation identities include the trusted execution scope, so equal request IDs in
different executions remain independent; retrying one request retains its file ID.
Artifact creation accepts open execution evidence, including imported evidence,
and refuses closed executions before writing metadata.
Artifact metadata, the original native file reference and its mutation receipt
commit in the product Store transaction with execution evidence. Reads and watches
retain product read cuts, and deletion removes these references in the execution
transaction. File bytes remain in native storage. Admission refuses incompatible
product file objects before execution recovery and refuses earlier native artifact
objects with the action to select a fresh storage root; it never resets that data.

Bundle creation and inspection live in `dokime-packaging` and use the library's
canonical manifest and USTAR rules. Native managed preparation takes an executable
and an explicit asset inventory, preserves their filenames, and includes the
C# runtime or C++ shared libraries beside the entrypoint. The command-line bundle
producer accepts each supporting file through `--asset`. Install, add and development commands observe Plugins.Watch and read the
current entry until it is Ready; the desktop installer uses the same admission
condition. Both remote and managed in-process admission take the entry name explicitly;
remote admission lets the library describe its own connection.
Case materialization and analysis invoke the admitted library operation
from the selected entry's catalog. They do not reconnect a second plugin instance.

Execution admission resolves the library PluginEntry and its admitted activation.
Each execution retains one scoped binding, one clock and its frozen stream selection.
Lifecycle calls use the catalogued operation identity through `binding.operations()`;
no product connection or invocation carrier remains. Child calls inherit the trusted
execution resources, and selected Samples publications retain that same scope.
The ingestion consumer caches each scope's row codecs, writes the Arrow samples
before acknowledging their ingestion, and releases its caches when the binding ends.
An interval stream's publication is one stored `harmos.batch.<row>` part: storage
decodes its columns straight into the row's Arrow arrays and gives sample `i` the
time `start_unix_nanos + i * period_nanos`, the next position and the next sequence,
so the Arrow layout, its readers and existing roots are unchanged. Device streams
keep one publication per sample. A finished run reads back only its execution metadata and outcome;
`Cycle::wait` reads each sealed execution's evidence and samples only for a caller
that awaits the cycle, so a long multi-stream run never materializes its samples
when it ends.
Binding opens no sample demand. The driver starts samples once initialize has
registered the unit and stops them when stepping ends, so criteria read settled
streams and no window reaches the plugin after cleanup; a run that stops early still
stops its samples before cleanup. Binding closure settles descendant calls and
accepted ingestion before streams close and the execution is sealed. Stop and close
each settle within `stop_grace`; a plugin that misses it leaves the execution
Interrupted with failure evidence naming the action. `stop_grace` is not a setting:
`dokime_runtime::STOP_GRACE` derives it as one third of the Plugins operation
timeout (`PLUGIN_TEARDOWN`, 30 s, so 10 s), because a managed teardown settles,
stops and then kills, each phase within `stop_grace`. The host binary, the CLI and
the desktop all read it; the desktop offers to exit without finishing once
`PLUGIN_TEARDOWN` plus one `stop_grace` has passed.
The host clock owns transport deadlines and policy retry waits. Each controlled
execution clock owns its plugin sleeps and publication cadence. Before returning
a lifecycle call, the driver awaits quiescence at the current execution time;
it does not abandon an in-flight sample when the plugin reply arrives.
Native storage retains a separate stable root history epoch. Deleting product
evidence invalidates evidence cuts without rotating that native history.

Production units use independent real clocks. `ControlledClock` creates a separate
virtual clock for each execution; awaiting the library Time.Sleep drives its due
sample windows and their ingestion. A scoped sleep settles after every window of its
unit whose samples are due by its end: exactly the slept periods on a controlled
clock; on the real clock every sample due by its end, plus any that fall due while
the grant reads the clock, because the wait's end grants the windows due by then
instead of waiting for a timer. Operator prompts hold their execution's timeline
while other executions can complete. Runtime retry delays remain owned by the
library operation policy. Attempt evidence is observed through the library
invocation middleware, including each actual retry.
The library samples interval streams first at their first due interval, rather than
inserting an extra sample at the binding's origin.

`execution_isolation` and `concurrent_execution_clocks_advance_independently` exercise
one connection shared by independently scoped runs. The latter holds one unit at
its prompt while the other completes, then checks that both retain their own exact
sample values and source times.

For this wave, compound product objects retain their transactional Store paths.
The operations manifest declares the workspace `harmos-runtime`
dependency. Runtime storage registration
combines the library plugin catalog with `host::storage::administration::catalog()`.
The library Plugins watch refreshes the suite projection and reconciles secret
bindings; observer cancellation is awaited before the host closes.

```rust,ignore
use crate::{Result, host::Context};
use harmos_runtime::builtins::storage::{CreateSnapshot, CreateSnapshotRequest, CreateSnapshotResponse};

pub async fn handle(context: &Context, request: CreateSnapshotRequest) -> Result<CreateSnapshotResponse> {
    context.storage::<CreateSnapshot>(request).await
}
```

`host::Host::plugins()` is async and returns the library `PluginEntry` values.
The product Host trait has no installation, enablement, replacement or plugin
settings methods; the library owns them. Declarations come from the frozen
service descriptors, including plugin-only methods, rather than a second manual
method list. Wrong audiences use `harmos_runtime::Failure::wrong_caller`;
`caller_status` retains that structured failure in direct gRPC status details.
Handler compilation and real-host story results are separate proof obligations.

### Storage Paths After This Wave

Product object migration is deferred for this wave. Existing roots are never reset;
an incompatible root is refused with the action to select a fresh storage root.
Native primary and state backends share the product root and history identity.

| Resource | Storage Path | Library Capability Needed for a Later Object Migration |
| --- | --- | --- |
| Executions and seals | Product Store transactions | Atomic compound mutation of execution state, evidence frontier and seal, with a replayable receipt. |
| Cycles and case bindings | Product Store transactions | Atomic admission of the cycle and its complete execution/binding set. |
| Plans and registered cases | Product Store transactions | Atomic plan/case registration and durable request-key replay. |
| Commands and progress | Product Store transactions | Atomic state transition, command receipt and evidence metadata publication. |
| Notes and traceability | Product Store transactions | Conditional whole-value replacement with the original request receipt and execution frontier. |
| Reports and document parts | Product Store transactions | Atomic frozen-document, execution-seal and content-reference publication. |
| Prompts and answers | Product Store transactions | Atomic answer acceptance, evidence publication and durable waiter recovery. |
| Evidence records and metadata | Product Store transactions | Shared compound mutation with execution lifecycle and retained read cuts. |
| Execution deletion | Product Store transactions | Atomic scope deletion with a durable receipt and retained-reader guarantees. |
| Samples | Product Arrow ingestion and registered native backend | Scoped capture must preserve per-sample source times on the window grid, sample order and the execution evidence frontier. |
| Files and uploads | Library native file operations through the registered backend | No product file-transfer store or wire twin. |
| Logical execution-file metadata and retained references | Product Store transactions | Logical paths select original native file references; replacement retains previous immutable bytes and cuts. |
| Artifact metadata and retained file references | Product Store transactions | Atomic execution-open admission, metadata publication and the original mutation receipt; file bytes remain native. |
| Plugin entries and settings | Library Plugins and plugins.Settings | Library-owned objects and lifecycle; no product installation record. |
| Plot preferences | Library native administration objects | Product declarations and authorization remain product-owned. |
| Protected secrets and access credentials | Product credential store and transactional catalog | Credential values remain in the operating system store. |
