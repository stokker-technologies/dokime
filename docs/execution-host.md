# New Execution Host

The independent host is built beside the current command-line and desktop host.
`dokime-runtime-next` composes plugin entries, scoped unary lifecycle dispatch,
controlled orchestration time and cycle execution. `dokime-operations-next` owns
one declaration and handler per supported host operation. `dokime-storage` owns
SQLite execution evidence and Arrow IPC sample batches. These crates have no
normal dependency on the old domain, runtime, server, client or facade.

## Run the Thermal Stories

Build the existing shared SDK plugin, then run either story by its full name:

```powershell
mise run build:host-fixtures
mise run test -- --package dokime-runtime-next --target system --exact thermal_suite_survives_host_restart::thermal_suite_survives_host_restart
mise run test -- --package dokime-runtime-next --target system --exact cancelling_a_thermal_step_runs_cleanup::cancelling_a_thermal_step_runs_cleanup
```

The fixture is `target/debug/dokime-rust-thermal` with the native executable
extension. `DOKIME_THERMAL_PLUGIN` can name an already built copy. Tests never
compile a plugin themselves. Both placements use the unchanged shared thermal
suite at `sdk/rust/examples/thermal.rs`.

The host requires explicit `HostSettings`: placement bounds and a
`SchedulerClock`. `ControlledClock` advances only through requested waits;
observers tick and commit samples before a plugin's host sleep settles.
Prompt arrival and cancellation use events, with no wall-clock sleeps.

## Serve Client gRPC

`services::serve(endpoint, root, host_settings, client_settings)` in
`crates/dokime-runtime-next/src/services/` binds a loopback endpoint and returns
`Server`; `endpoint()` exposes the chosen address and `close().await` cancels
active work, drains cleanup and closes the listener. Client settings supply
read, mutation and stream deadlines, case/page limits, message bytes and stream
buffer capacity.

The public `process::command` constructor starts owned child processes without
opening Windows console windows.

The `dokime-host-next` binary accepts `--endpoint HOST:PORT --storage PATH`.
It announces its bound HTTP URL on stdout, and stdin EOF requests graceful close.
Ordinary runs use UTC time; `--controlled-clock` is the deterministic story mode.
The storage root retains a backend identity; each process has a new boot epoch.
Remote authentication is pending, so this listener accepts loopback addresses.

The first served methods are Plugins Add/List; Catalog ListSuites/GetSuite;
Planning MaterializeTemplate; Execution CreateCycle/GetExecution; Observation
WatchExecutionProgress; Prompts ListPending/GetPrompt/Respond; Query
ListExecutions; Evidence ListRecords; Telemetry ListSamples; Reports
DownloadExecutionReport/PublishExecutionReport; Health ListHealth; and Backend identity, health and
availability. Backend availability advertises only these methods. Other methods
return `UNIMPLEMENTED` naming the action; the owning OpenSpec task lists them.
Requests using unserved filter/cursor variants are explicitly refused.
Evidence additionally serves captured-schema lookup and record watches, with
selection-bound cursors, explicit `dokime.log` reads and Run envelopes. Telemetry
serves bounded forward/tail pages, source-time windows and event-driven watches,
including Run envelopes. Plots serves ListAnalyses, QuerySnapshots and Watch with
Run envelopes. Its ten kind base paths reduce in storage; advanced expressions,
filters, grouped distributions, XY intervals and live bucket deltas remain open
in the owning change's tasks.

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
mise -C sdk/python run sdk:prepare
mise -C sdk/python run sdk:build
mise -C sdk/python run sdk:install
mise run test -- --package dokime-runtime-next --target system --exact client_runs_thermal_suite_over_grpc::client_runs_thermal_suite_over_grpc
mise run test -- --package dokime-runtime-next --target system --exact python_client_runs_thermal_suite_over_grpc::python_client_runs_thermal_suite_over_grpc
```

The Rust story uses `dokime::client::HostClient` service conveniences over the
generated clients reexported by `sdk/rust`'s `dokime` package. The Python story uses the built pure wheel in the root `.venv/`;
`DOKIME_PYTHON` selects another explicitly prepared interpreter. Neither story
builds artifacts, polls, or sleeps on the wall clock. The Rust journey also starts
two executions in one cycle, answers their prompts differently and independently
reads their records through the client. The system target contains nineteen stories,
including publication, interrupted recovery and three language-equivalence stories.
Every story uses `SYSTEM_HANG_GUARD` from `tests/support/settings.rs`; operation
policies retain their own deadlines.

## Compare Every SDK

The single thermal procedure is authored in each SDK: Rust `examples/thermal.rs`,
Python `examples/thermal/suite.py`, C# `tests/SystemPlugin` and C++ `tests/system`.
The host compares the complete frozen origin/test case, ordered evidence and
samples (including source time, sequence, position and exact payload), outcome
and cycle counts. Only run-specific execution identity, plugin digest/start time
and receipt timestamps are excluded. Sealed evidence must reopen unchanged.
Python and C++ also decode Door from retained schemas. C# cannot yet register a
record-only descriptor through the pinned Harmos API; its named
`csharp_frozen_record_schema_decodes_door` assertion is pending upstream support,
tracked under Requests to Harmos in the gRPC change design.

Build the shared fixtures and install the Python wheel with the tasks above.
`build:host-fixtures` invokes each compiled SDK's own tasks and exports the
fixtures under `target/build/<profile>/<target>/fixtures/host/`. C# includes its
runtime. C++ builds Harmos's pinned dependencies; `HARMOS_SOURCE` and
`DOKIME_HARMOS_CACHE` can select existing workstation inputs. Tests never compile
fixtures. Tests Build includes the same producer; the Linux and Windows pull
request and Nightly jobs consume its exports and run all execution-host stories.

```sh
mise run test -- --package dokime-runtime-next --target system --exact python_plugin_matches_rust_thermal_evidence::python_plugin_matches_rust_thermal_evidence
mise run test -- --package dokime-runtime-next --target system --exact csharp_plugin_matches_rust_thermal_evidence::csharp_plugin_matches_rust_thermal_evidence
mise run test -- --package dokime-runtime-next --target system --exact cpp_plugin_matches_rust_thermal_evidence::cpp_plugin_matches_rust_thermal_evidence
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
Frequency transforms require a source-time window within the configured input
bound. Snapshot batches fit one configured message; Watch emits one plot per
frame in request order, with no fixed plot-count ceiling.

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
synchronous durability. Streams close before evidence closes. A seal hashes the
frozen execution and ordered evidence bytes, and reopened reads verify it.
Writes after close are refused. Open plugin-run executions are recovered as
interrupted and sealed; import origins are never run-recovery targets.

Storage remains at v1. An unrecognized nonempty root is refused with instructions
to back it up and choose fresh storage or explicitly reset it. No migration,
implicit reset or deletion occurs.

## Content and Notes

Content serves ordered artifact transfers, execution/cycle evidence exports and
registered execution-file inventory, bounded windows, downloads and follows.
Client uploads require an explicit SHA-256 completion before input finish;
cancelling a staged upload publishes nothing. Plugin artifact calls are unary
and preserve sequence, exact retry, prefix replacement and terminal abort.
File names select retained content identities, never arbitrary filesystem paths.
A resumed follow requires the original identity and committed offset.
Notes support create/edit/list and Run envelopes, durable idempotent creation,
selection-bound paging and closed-owner refusal. The loopback listener uses one
local client principal until remote authentication lands.

Build the content-story fixture before its client journey:

```sh
mise run build -- --cargo build --package dokime-runtime-next --example content_fixture
mise run test -- --package dokime-runtime-next --target system --exact client_transfers_execution_content::client_transfers_execution_content
mise run test -- --package dokime-runtime-next --target system --exact client_keeps_execution_notes::client_keeps_execution_notes
```

Appearance asset downloads and Media capture return named refusals pending their
owning API/storage decisions. Query remains at its previously served surface.

## Cutover

CLI and desktop still use the old host. The integration cutover must migrate
those consumers, delete the superseded loop, crates and tests, and rename the
`-next` crates. The console-free process constructor is copied from `dokime-domain/src/process/`
into the new runtime. Delete its old owner at cutover; no old crate is imported.
The dispatcher uses the published `harmos.v1.Request.scope` field directly;
it does not call the old Harmos runtime's scope-losing invocation API and needs
no local path override. The native-runtime scoped-call adoption remains with
its owning cutover.
