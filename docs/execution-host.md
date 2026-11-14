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
owner requiring a capture manager.

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
`SchedulerClock`. `ControlledClock` advances only through requested waits;
observers tick and commit samples before a plugin's host sleep settles.
Prompt arrival and cancellation use events, with no wall-clock sleeps.

## Serve Client gRPC

`services::serve(endpoint, root, host_settings, client_settings)` in
`crates/dokime-runtime/src/services/` binds a loopback endpoint and returns
`Server`; `endpoint()` exposes the chosen address and `close().await` cancels
active work, drains cleanup and closes the listener. Client settings supply
read, mutation and stream deadlines, case/page limits, message bytes and stream
buffer capacity.

The public `process::command` constructor starts owned child processes without
opening Windows console windows.

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

Plugins, Catalog, Settings, Secrets, Appearance and evidence Health
serve every declared method, including their Run transports. Plugin installation
accepts verified v1 `.dokime` tar bundles with one native sidecar per target,
portable regular-file paths, matching size/digest inventory and a compatible
`dokime.json`. Managed entries start on restart; disabled installations stay
disabled. Replacement retains that state and restores the previous entry when
starting its replacement fails. Removing an entry preserves execution evidence.
An installation deadline cancels pending startup and settles rollback before
reporting failure: the owned child is stopped and reaped, its installation is
removed, and a previous installation is restored when present.
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
`administration` table. Root validation requires that combined v1 layout and
refuses incompatible roots with backup and fresh-root/reset guidance; it never
migrates or deletes existing data.

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
mise -C sdk/python run build
mise -C sdk/python run install
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
created. `retained_bytes` bounds the database copy. Incompatible roots produce a
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
dokime plugin bundle --executable first-plugin/target/debug/dokime-starter --out first.dokime
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
Rust's executable is `dokime-starter` (`.exe` on Windows); C# must be published as
a self-contained single-file executable for an executable-only bundle. Python
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
stop_grace_s = 30
message_bytes = 4194304
retained_bytes = 67108864
stream_buffer = 8
max_cases = 1000
page_items = 1000
```

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
full selection runs every story. Pull requests build Rust and Python and
run every story except the C# and C++ equivalence stories, which run in Nightly. Nightly
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
synchronous durability. Streams close before evidence closes. A seal hashes the
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
Evidence downloads use v1 protobuf `ExecutionArchive` bytes defined in
`crates/dokime-storage/src/execution.rs`; cycle archives repeat that message at
field 1. Content metadata and bounded chunks travel in fields 7 and 8 of each
execution archive. The shape stays at v1 under the development storage policy.
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
The remaining adapter work is the final resource grammar migration described in
the consumer work order below; its current adapters still use the earlier names.

## Final Library Definition Contract

The repaired definitions pin Harmos at `299b7ce492e3a82bc4ddf61dc60c2161b6824110`.
It replaces definitions and generators; operations/runtime, storage and SDK consumers
are intentionally awaiting the work lists below. These declarations do not prove
runtime admission, end-to-end behavior or consumer compilation. The existing
`grpc-plugins` delivery remains active; no specification lifecycle command is part
of this pass. The plot/document correctness work from dev is merged through the recorded
Git renames; consumer repairs must preserve it while adopting the final types.

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
FTP capture and the effects journal are not restored.

### Harmonized Request and Response Shapes

| Operation (ALL) | Unified Request | Unified Response and Admission |
| --- | --- | --- |
| content.Artifacts.Create | file, optional scope, filename, content_type, request_id, metadata | ArtifactMetadata and receipt; plugin scope defaults to its execution and cannot escape it. content_type replaces media_type. |
| Evidence.Statistics | optional scope, stream, versions, between, outputs, field, step, optional at | Complete Statistics and resolved_at; plugins use their own execution. |
| evidence.Records.List | optional scope, record_type, versions, between, step, selection, optional at/page | records, schema, count, last, resolved_at, next_page_token; CapturedRecord retains payload, source timestamp and sequence. |
| evidence.Traceability.Get | optional scope, page, at, entry | snapshots and page metadata, plus selected entry value/revision; scope selects the execution, entry selects the independently revisioned entry. Foreign selectors refuse. |
| executions.Progress.Get | optional name, optional at | Complete value, revision and resolved_at; clients give name, plugins default to and are restricted to their execution. |
| Secrets.Get | name, optional at | secret bytes, optional administrative value/revision and resolved_at; plugins read only declared names through their own bindings and receive no administrative metadata. |

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
[Harmos Architecture](https://github.com/stokker-technologies/harmos/blob/299b7ce492e3a82bc4ddf61dc60c2161b6824110/docs/architecture.md).

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
| new | `dokime.host.v1.plugins.Secrets/Set` | CLIENTS | `dokime.host.v1.plugins.SecretsSetRequest` | `dokime.host.v1.plugins.SecretsSetResponse` |
| new | `dokime.host.v1.plugins.Secrets/Delete` | CLIENTS | `dokime.host.v1.plugins.SecretsDeleteRequest` | `dokime.host.v1.plugins.SecretsDeleteResponse` |

New resource: `dokime.host.v1.plugins.Secrets` has List, Set and Delete, all CLIENTS.
Its messages are `Secrets{List,Set,Delete}{Request,Response}` in that package.
The model contribution is `dokime.model.v1.SecretsDeclarations` containing
`SecretDeclaration { metadata, required }`. Binding reads return declarations,
bound host secret identifiers and availability, never secret values. Set replaces
the complete map; Delete removes selected bindings. Both carry revision and retry
identity. Reconcile bindings at startup and remove them when the library Plugins
watch reports deletion.

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
| `dokime.client.v1.plugins.SecretAvailability` | `dokime.host.v1.plugins.SecretBinding` |
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
| `dokime.client.v1.PluginPreferences` | `dokime.host.v1.PluginPreferences` |
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
from `harmos_runtime::builtins::storage`; use the native object, record, sample,
query and file operations instead of `context.host.store()` or `resources.store`.
The remaining direct Store callers are migration obligations, not an alternate
interface. The operations manifest must declare the workspace `harmos-runtime`
dependency; only the manifest owner may edit it. Runtime storage registration
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
The source handoff does not assert that the remaining handlers or runtime compile.

### State After Definition Repair

The source closures below are the consumer reports in this branch's history,
not retrospective claims that their complete lanes passed. Definition proof does
not close application admission, durability or real-host stories.

| Consumer | Reported Source Closure | Remaining After This Repair |
| --- | --- | --- |
| Operations and runtime | None of 1–11 | All 11; registration/dispatch, harmonized handlers, Plugins/settings, callers, native storage and clock. |
| Storage backend | 1–6 and 8–10 | 7: caller mapping and registration with runtime. |
| Rust SDK | 1–6 and 8 | 7: full SDK/installed/host proof; generated consumer now compiles. |
| Python SDK | 1–6 and 8 | 7: installed/host proof; generated consumer, lint and public typing pass. |
| C# SDK | 1–5 and 7 reported closed | 2 reopened for the two authored FileTransfers errors; 6 full proof. Generated consumer passes with that file excluded. |
| C++ SDK | 1–6 and 8 | 7: installed/host proof; generated client/plugin compile. |
| Command-line tool | 1–6 migrated in source | Compile and real-host proof after operations/runtime; fixture producer now selects the exported test subfolder. |
| Desktop | 1–5 and 7 migrated in source | 6: core compilation and both native WebView lanes; generated desktop client compiles. |

## Consumer Work Lists

Each numbered item is one repair obligation; counts below count items, not files.
Generated output has no independent writer: change its generator and regenerate.
The definition pass removes superseded declarations and wrappers; these consumers
must now adopt them together, preserving all existing behavior and fresh-root refusal.

### Operations and Runtime Crates (11 Items)

All items remain open. Renamed frozen symbols are not compatibility APIs: remove
ReadSettings and PluginInstallation while composing library settings and PluginEntry.
The generated layer is now available; its compilation does not close these owners.

1. In `crates/dokime-operations/src/{lib.rs,host/mod.rs,resources.rs}` and `crates/dokime-runtime/src/{lib.rs,host.rs,dispatch.rs,plugin.rs,execution.rs,execution_manager.rs}`, replace old client/context/v1 module imports and registration identities using the complete map; repair references left by the file moves below.
2. In `host/content/artifacts/create.rs`, `host/evidence/{statistics.rs,records/list.rs,traceability/get.rs}` and `host/executions/progress/get.rs`, implement the richer request/reply and trusted plugin selectors. The narrower duplicate handlers are deleted. Preserve statistics logic; apply only admission/shape changes around it while adapting the merged correctness changes.
3. In `host/secrets/{get.rs,store.rs}` and runtime secret-declaration admission, authorize declared secret bindings before reads, return bytes to both audiences and administrative metadata only to clients. Do not infer identity from payload or expose secret administration to plugins.
4. Add the handlers for `plugins.Secrets.{List,Set,Delete}` in `host/plugins/secrets/{list,set,delete}.rs`; catalog SecretsDeclarations, revision complete maps, reconcile at startup and remove bindings from the library Plugins watch.
5. In runtime `services/{mod.rs,state.rs}` and host assembly, register the library Plugins and plugins.Settings directly and use its PluginEntry name as the only identity. Product operations under `host/plugins` and runtime `services/{plugins,plugins_settings,plugin_entries,managed_plugins}.rs` are deleted. Retain `.dokime` validation via `dokime-packaging` and supply the library's content/launch integration instead of restoring product installation records.
6. Apply `setup.composition(...)` before listeners/plugins start; enforce effective callers for direct, encoded and child admission and discovery, with wrong-caller refusal in both directions. CLIENTS and PLUGINS are independent of Protected/Public visibility and resource grants.
7. Route all storage through the library's native operations, including executions, cycles, plans, notes, outcomes and reports as objects/records. Remove direct Store access from the inventory below; do not add a storage gRPC service or an SDK storage twin.
8. Compose library Time.Get/Sleep; product operation files `time.rs` and `sleep.rs` and the settings-read operation are deleted. Adapt `clock.rs`, `services/clock.rs` and scheduler integration to the library execution clock while preserving controlled ticks, cancellation and observer/sample order.
9. Preserve unimplemented media capture, FTP and Effects behavior. Add declarations/admission without restoring their engines or reporting empty success. Replace SDK manual context bindings with descriptor-generated service bindings.
10. Migrate `crates/dokime-protocol/tests` and runtime/operations stories, fixtures and build selectors; repair module registration after rename, prove lifecycle bindings and wrong-caller cases, and leave full hosted validation open until every consumer has handed off.
11. Adapt typed Statistics (including sum), plot summaries and exact category totals to the new host/model packages; retain the merged raw analysis time-axis fix, per-batch shared readers/catalogs, pending document bindings and card completion. Plugin Evidence.Statistics uses the harmonized Statistics payload/read cut rather than the retired flat response.

### Storage Backend (10 Items)

The backend report closes source items 1–6 and 8–10. Item 7 remains with operations/
runtime: map product callers and register primary/state backend declarations.
Keep the backend algorithms and durability changes; do not reimplement them.

1. In `crates/dokime-storage/src/{backend.rs,store.rs,lib.rs,history.rs}`, implement one root-scoped StorageBackend with stable history, selecting executions through product scope. Retire the per-execution Backend facade as consumers move. Add the StateBackend owner with the same history and durable recovery state.
2. Implement every mandatory trait method in the tables below, including resumable-file fencing/checkpoints, original mutation receipts, conditional objects, schema identities and native projected Arrow reads; unsupported default methods do not establish product capability. The pin still supports only native PostcardV1: map product values/schema losslessly at the ports and retain frozen protobuf definitions, never relabel protobuf bytes as Postcard or add a duplicate durable sample representation.
3. Preserve `ipc{,/**}.rs`, `telemetry/**`, `streams.rs`, `stream_*`, catalog/flush and snapshot pinning: one compressed Arrow representation, file sync before catalog commit, rollback/recovery/finalization and accepted-versus-durable separation. No old sample SQL tables return.
4. Store product execution/cycle/plan/note/outcome/report values through native object/record ports with protobuf schema definitions. Remove `Store::save_plugin_administration` (deleted here) and product plugin-entries/plugin-installations/managed-executables stores; generic administration remains only for remaining product owners until migrated.
5. Replace `expressions.rs`'s `ordered_merge::as_of` call with `harmos_query::merge::TimeMergeReader` over retained, projected BatchReaders; use native Query.merge/TimeMerge with Range, AtOrBefore or Intervals as appropriate. Adapt record range, value-at-record, paired intervals, grouping and stream-to-stream callers without rewriting the correctness worker's algorithms.
6. After those callers move, delete `src/ordered_merge.rs`, `mod ordered_merge` in `lib.rs` and its now-redundant merge tests/imports. Sorting, scalar/aggregate execution, merge and downsampling belong to harmos-query, never a second product query engine.
7. At runtime assembly, create StorageBackends { primary, state }, StorageManager::with_catalog and setup.settings().storage(&storage), then register native storage declarations. Both backend handles share the same root identity; storage is in-process only.
8. Preserve archive member paths and unchanged Arrow bytes, retained cuts, content digests, crash-safe deletion, diagnostics and baseline v1. Refuse incompatible existing roots with backup/fresh-root guidance; never migrate, delete or reset them implicitly.
9. Adapt `crates/dokime-storage/tests`, storage/telemetry benches and runtime archive/restart/fault stories. Preserve deterministic durability and exact integer-statistics proof; counts/work evidence remain measurements with no time/load thresholds. Retain the merged plot/document/statistics corrections through final proof.
10. Map product model ExactNumber to native query exact values after the old harmos_protocol::storage wire import disappears. Preserve merged integer-exact reducers, rational ranks/means/sums, full-coverage durable moments and clipped-window behavior; project typed Statistics and category totals without rounding through f64.

### Rust SDK (8 Items)

Source migration is reported closed except proof item 7. Shared replies now import
from the root, EvidenceStatisticsRequest.outputs is a Vec, and empty watch options
are consumed explicitly. The generated compiler story passes; installed and host
journeys still require the repaired operations/runtime.

1. In `sdk/rust/src/{lib.rs,client.rs,context.rs,message.rs,prelude.rs}` and the moved `plugin/mod.rs`, adopt private wire and generated root/client/plugin barrels; root values and plugin authoring helpers have distinct owners. Remove public protocol/v1 imports, not aliases for them.
2. Use generated `client/resources.rs` and `plugin/host.rs`; `host.rs` now derives every Request/Response/ID binding from service descriptors. Remove handwritten resource grafts and caller-specific method twins from `client/host.rs`.
3. Adapt `client/{files.rs,schemas.rs}`, `record.rs`, `prompt.rs`, `stream.rs`, `template.rs`, `suite.rs`, `plugin/mod.rs` and derive output to rich unified replies, public Upload, library PluginEntry/SettingsDeclarations and product secret declarations.
4. Implement typed lifecycle/analysis binding adapters for Criteria.Evaluate, Executions.Initialize/Cleanup, Steps.Execute, TestCases.Materialize and Analyses.Compute; retain independently authored Protected IDs and policies.
5. Keep the generated root composed: library Plugins/Settings/Time and product executions/files/plugins.Secrets share one tree; do not restore plugin wrappers or storage wire calls.
6. Migrate `examples`, `tests/consumer`, fixtures and README/SDK guide imports to content arguments, call options and unwrapped payloads. The rename map below supplies moved operation paths for real-host fixtures.
7. Build/test this SDK and its product/installed consumer with mise, regenerate after any renderer change, and verify deterministic output and caller admission. Definition-only proof does not replace this repair.
8. Regenerate and adopt model Value.exact/ExactNumber, typed host Statistics/plot summaries and PlotColumn.totals; update exhaustive scalar matches and formatting without rounding. Preserve raw AnalysisSamples axes and host Documents binding/summary/card-completion states in client and plugin helpers.

### Python SDK (8 Items)

Source migration is reported closed except proof item 7. Import Client from client
and Plugin/Context from plugin; shared payloads have one root home. Generated client
overrides no longer inherit incompatible library signatures. Public typing and
generated lint pass; installed/host journeys still need consumer proof.

1. In `sdk/python/src/dokime/{_client.py,_context.py,_plugin.py,authoring.py}`, replace old v1/client/plugin.host imports with the generated public barrels and internal `_wire` packages; no old package alias remains.
2. Adopt generated `client.Client` and context HostRoot directly, preserve async close/cancellation/deadline/metadata behavior, and delete the old `_client.py` generic service-map/dynamic graft once its useful conveniences have moved.
3. Migrate `documents.py`, `plot.py`, `values.py`, `schemas.py` and record/prompt/stream adapters to final owner types without rewriting document/plot algorithms beyond schema adaptation.
4. Compose the pinned Harmos PluginEntry/settings/Time APIs and product secret declaration/binding APIs; remove product plugin/settings/clock wrappers and undeclared-secret paths.
5. Migrate plugin lifecycle/analysis bindings, examples and fixture imports; preserve the host-driven examples and original mutation identity on retries.
6. Update `tools/{generated.py,stubs.py}`, wheel/installed stories and README to the private `_wire/surface.pb` and sources manifest, generated argument signatures and payload returns. Retired versioned wire packages are deleted.
7. Run SDK generation/build/installed-wheel and host-driven proof through mise after repair; keep the pure wheel and root `.venv` ownership and the immutable git dependency pin.
8. Regenerate scalar/statistics/plot wire outputs; preserve Fraction conversion in values.py using model ExactNumber, typed statistic presence and exact totals. Adapt analysis and document helpers to the new plugin/host packages while retaining raw aligned axes and pending/result/card-completion handling.

### C# SDK (7 Items)

Source items 1–5 and 7 were reported closed. Item 2 now also owns the two authored
FileTransfers compile errors listed above; do not hide them in a full SDK build.
The generated roots use public library roots instead of internal request types.
Run restore --refresh-lock only when changing dependency inputs; ordinary restore
and generation proof use the tracked locks in locked mode. Item 6 remains proof.

1. In `sdk/csharp/src/Dokime/{Context.cs,Plugin.cs,Authoring.cs,Authoring/**}`, adopt Dokime, Dokime.Clients and Dokime.Plugins payloads and generated Client/HostRoot. The manual Client.cs and old Resources/ClientResources outputs are deleted.
2. Migrate FileTransfers.cs and resource conveniences to Upload, rich replies, optional call context and generated argument methods; preserve explicit channel/disposal ownership and cancellation.
3. Replace product plugin/settings/clock wrappers with the pinned local-feed library APIs and add secret declaration/binding conveniences; keep internal request/stub classes internal.
4. Migrate lifecycle/analysis bindings, examples, tests and shared fixtures to canonical messages, host-only Protected identities and final public namespaces.
5. Run `mise -C sdk/csharp run restore` to rebuild the immutable source feed and recreate local-feed lockfiles; the old revision's lockfiles were retired, not hand-edited. The feed now stages sdk/csharp/src/Harmos including its generated wire output.
6. Regenerate before build, run the installed NuGet consumer and host-driven stories via mise, and update `tools/check_clients.py` and SDK docs. Do not restore Grpc.Tools compilation over the same checked-in Generated/Wire inventory.
7. Regenerate Value.exact/ExactNumber and typed Statistics/plot fields; adapt scalar conversion and displayed statistics without double coercion. Consume exact category totals, raw analysis axes and host document binding/summary/card-completion states from the final namespaces.

### C++ SDK (8 Items)

Source items 1–6 and 8 are reported closed. Generated request aliases include their
resource owner, reserved words gain an underscore, policies/operations/watches use
the library owner, and plugin AnalysisSamples is exported. Generated proof passes;
item 7 still owns installed and real-host proof.

1. In `sdk/cpp/include/dokime/{plugin.hpp,context.hpp,plots.hpp,documents.hpp}` and `src/{plugin,context,client}.cpp`, adopt public root/client/plugin types and private wire includes; move authoring helpers whose names collide with root model values into the plugin barrel and remove old generated package names without changing plot/document logic.
2. Use the generated Client tree directly from client.hpp and messages.hpp; the former resources.hpp forwarding tree is deleted. Migrate file conveniences to Upload and unified replies.
3. Connect generated `include/dokime/_generated/host.hpp` and `src/generated/host.cpp` to context/plugin authoring and requirement declarations; adapt lifecycle and analysis callbacks while preserving independent Protected identities.
4. Consume Harmos through its pinned archive's CMake package and generated private wire layout. `cmake/{Dependencies,Library,Install}.cmake` and DokimeConfig use that owner, not a second handwritten Harmos source list.
5. Keep generated protobuf headers out of public signatures, import named types in source only, and use generated call options with owned ClientContext and payload projection.
6. Migrate examples, installed consumer, conformance fixture, CMake tests and docs; resolve native dependencies from one vcpkg graph and preserve package export/install behavior.
7. Build/package/install and run real-host proof through SDK mise tasks after repair. CMake produces the wire message/stub outputs; committed resource and payload barrels come from the product generator.
8. Regenerate scalar/statistics/plot messages through CMake and update scalar variants/conversion for model ExactNumber, typed field presence and category totals without double coercion. Preserve raw AnalysisSamples axes and host document states in the public projections.

### Command-Line Tool (6 Items)

All six items are reported migrated in source. Regenerated Rust barrels now compile;
CLI/core compilation and real-host proof remain. Exported projects contain plugin
and test subfolders (Plugin/Test for C#); fixture builds select the test subfolder
and its current executable/project names. Do not restore retired starter trees.

1. In `crates/dokime-cli/src/commands/{plugin,catalog,execution,cycle,plan,proto}` and operation.rs, replace raw messages/old forwarding clients with generated content arguments, call options and projected replies.
2. Use the library PluginEntry identity, upload managed bundles through product Files then Plugins.Create, and add declared plugin-secret binding commands without a second installation record.
3. Migrate config.rs, commands/server.rs and client/connect paths to assembled native storage, Time and composition policy; keep explicit fresh-root refusal and controlled-clock behavior.
4. Migrate proto export, `dokime new` and generated starter bindings to final imports and public barrels. The client/plugin/test examples are now the source inventory; keep their complete exported layouts.
5. Update CLI system stories, packaging/installer launch paths, generated completions/help and command docs; verify the renamed dokime executable with focused mise proof.
6. Export the regenerated model/host/plugin scalar and statistics contracts and preserve exact integer/rational JSON and display output; adapt analysis/document reads to raw sample axes and host-owned card states without converting exact values to drawing coordinates.

### Desktop (7 Items)

Items 1–5 and 7 are reported migrated in source. Both generated preference identities
now name the host package, and native dispatch/error identity is regenerated.
The isolated TypeScript consumer passes without compiling runtime-owned preference
defaults; item 6 still owns that export, core and native platform
proof. Preserve the merged plot/document behavior during any remaining adaptation.

1. In `crates/dokime-gui/src-tauri/src/{host,rpc/**}` and `tools/typescript/generate.mjs`, regenerate/model final host descriptors and replace old client/context imports and service registration.
2. Adapt plugin administration/settings views to library PluginEntry/plugins.Settings and product plugins.Secrets; remove product plugin/installation projection adapters and caller-specific operation variants.
3. Adopt the harmonized request/reply shapes, public Upload, explicit read cuts and typed argument calls; keep execution/cycle/case/report commands client-only and preserve authenticated listener ownership.
4. Preserve the merged frontend and storage plot/document/statistics correctness changes through the recorded git moves. Adapt only schema/import paths around their behavior; preserve packed time axes, bounded plot groups and fixed document cuts.
5. Migrate frontend generated transports, UI types, desktop fixtures and stories, updating docs/gui.md when behavior changes; retain unavailable media/effects behavior and meaningful refusal errors.
6. Prove both native WebView lanes and touched deterministic stories via mise after every consumer has handed off; no platform substitutes for the other.
7. Regenerate final descriptors and project model Value.exact/ExactNumber, typed Statistics/plot summaries and PlotColumn.totals without JavaScript Number coercion. Read pending/result bindings, DocumentSummaryState and CardComplete from dokime.host.v1; preserve merged literal-table settlement, parallel bounded group loading, stable card identities and generation fences.

### Remaining Source Ownership Partitions

These counts cover existing Rust source files, not completed tasks or future additions.
They partition paths for a later authorized set of workers; this repair used one writer.

| Owner | Disjoint Paths | Existing Files | Largest Remaining Work |
| --- | --- | ---: | --- |
| Runtime | crates/dokime-runtime/src/** | 65 | Library Plugins/settings, composition/callers and dispatch, clock, admission and native persistence. |
| Evidence and rendering | operations host/{evidence,plots,documents,analyses}/** | 41 | Harmonized evidence replies/selectors, native reads and exact values. |
| Lifecycle and planning | operations host/{cases,cycles,executions,prompts,prompt_definitions,reports}/**, host/{planning,execution_state}.rs | 53 | Native objects/records and lifecycle handlers; Progress.Get belongs here. |
| Content and administration | operations host/{backend,content,files,secrets,settings,storage,record_schemas,stream_schemas,suites}/** | 57 | Artifacts.Create, secret authorization, library file/storage integration and metadata. New plugins/secrets handlers also belong here. |
| Operation assembly | operations src/{lib,resources}.rs, host/{mod,paging,catalog_inventory,projection,views}.rs | 7 | Registration, composition, retired symbols and common handler interfaces. |
| Runtime stories | crates/dokime-runtime/tests/** | 52 | Wrong callers, scope, clock, library integration and deterministic host stories after source owners close. |

The operations prefixes in this table are under crates/dokime-operations/src/.
The common assembly owner alone changes the handler interfaces; handler owners
report required interface changes instead of editing that owner's seven files.

### Operation Source Rename Inventory

Moves use git mv. The old client modules are now host resource folders; handlers
retain their bodies for consumer repair. Helpers remain with their resource or host
composition. Narrow duplicate context handlers and composed library wrappers are
removed, not kept as compatibility modules.

| Old Path | Final Path |
| --- | --- |
| `crates/dokime-operations-next/Cargo.toml` | `crates/dokime-operations/Cargo.toml` |
| `crates/dokime-operations-next/src/error.rs` | `crates/dokime-operations/src/error.rs` |
| `crates/dokime-operations-next/src/client/analyses_list.rs` | `crates/dokime-operations/src/host/analyses/list.rs` |
| `crates/dokime-operations-next/src/client/backend_appearance.rs` | `crates/dokime-operations/src/host/backend/appearance.rs` |
| `crates/dokime-operations-next/src/environment.rs` | `crates/dokime-operations/src/host/backend/environment.rs` |
| `crates/dokime-operations-next/src/client/backend_overview_get.rs` | `crates/dokime-operations/src/host/backend/overview/get.rs` |
| `crates/dokime-operations-next/src/client/backend_overview_watch.rs` | `crates/dokime-operations/src/host/backend/overview/watch.rs` |
| `crates/dokime-operations-next/src/client/cases_create.rs` | `crates/dokime-operations/src/host/cases/create.rs` |
| `crates/dokime-operations-next/src/client/cases_delete.rs` | `crates/dokime-operations/src/host/cases/delete.rs` |
| `crates/dokime-operations-next/src/client/cases_get.rs` | `crates/dokime-operations/src/host/cases/get.rs` |
| `crates/dokime-operations-next/src/client/cases_list.rs` | `crates/dokime-operations/src/host/cases/list.rs` |
| `crates/dokime-operations-next/src/client/cases_parameters.rs` | `crates/dokime-operations/src/host/cases/parameters.rs` |
| `crates/dokime-operations-next/src/client/cases_watch.rs` | `crates/dokime-operations/src/host/cases/watch.rs` |
| `crates/dokime-operations-next/src/client/catalog_inventory.rs` | `crates/dokime-operations/src/host/catalog_inventory.rs` |
| `crates/dokime-operations-next/src/client/content_artifacts_create.rs` | `crates/dokime-operations/src/host/content/artifacts/create.rs` |
| `crates/dokime-operations-next/src/client/content_artifacts_get.rs` | `crates/dokime-operations/src/host/content/artifacts/get.rs` |
| `crates/dokime-operations-next/src/client/content_artifacts_list.rs` | `crates/dokime-operations/src/host/content/artifacts/list.rs` |
| `crates/dokime-operations-next/src/client/content_artifacts_watch.rs` | `crates/dokime-operations/src/host/content/artifacts/watch.rs` |
| `crates/dokime-operations-next/src/client/content_execution_files_get.rs` | `crates/dokime-operations/src/host/content/execution_files/get.rs` |
| `crates/dokime-operations-next/src/client/content_execution_files_list.rs` | `crates/dokime-operations/src/host/content/execution_files/list.rs` |
| `crates/dokime-operations-next/src/client/content_execution_files_watch.rs` | `crates/dokime-operations/src/host/content/execution_files/watch.rs` |
| `crates/dokime-operations-next/src/client/content_transfer.rs` | `crates/dokime-operations/src/host/content/transfer.rs` |
| `crates/dokime-operations-next/src/client/cycles_create.rs` | `crates/dokime-operations/src/host/cycles/create.rs` |
| `crates/dokime-operations-next/src/client/cycles_get.rs` | `crates/dokime-operations/src/host/cycles/get.rs` |
| `crates/dokime-operations-next/src/client/cycles_list.rs` | `crates/dokime-operations/src/host/cycles/list.rs` |
| `crates/dokime-operations-next/src/client/cycles_seal.rs` | `crates/dokime-operations/src/host/cycles/seal.rs` |
| `crates/dokime-operations-next/src/client/cycles_set.rs` | `crates/dokime-operations/src/host/cycles/set.rs` |
| `crates/dokime-operations-next/src/client/cycles_watch.rs` | `crates/dokime-operations/src/host/cycles/watch.rs` |
| `crates/dokime-operations-next/src/client/document_expansion.rs` | `crates/dokime-operations/src/host/documents/expansion.rs` |
| `crates/dokime-operations-next/src/client/documents_get.rs` | `crates/dokime-operations/src/host/documents/get.rs` |
| `crates/dokime-operations-next/src/client/document_plot.rs` | `crates/dokime-operations/src/host/documents/plot.rs` |
| `crates/dokime-operations-next/src/client/documents_watch.rs` | `crates/dokime-operations/src/host/documents/watch.rs` |
| `crates/dokime-operations-next/src/client/evidence_archive.rs` | `crates/dokime-operations/src/host/evidence/archive.rs` |
| `crates/dokime-operations-next/src/client/evidence_health_events_list.rs` | `crates/dokime-operations/src/host/evidence/health_events/list.rs` |
| `crates/dokime-operations-next/src/client/evidence_health_events_watch.rs` | `crates/dokime-operations/src/host/evidence/health_events/watch.rs` |
| `crates/dokime-operations-next/src/log.rs` | `crates/dokime-operations/src/host/evidence/logs/append.rs` |
| `crates/dokime-operations-next/src/client/evidence_notes_create.rs` | `crates/dokime-operations/src/host/evidence/notes/create.rs` |
| `crates/dokime-operations-next/src/client/evidence_notes_get.rs` | `crates/dokime-operations/src/host/evidence/notes/get.rs` |
| `crates/dokime-operations-next/src/client/evidence_notes_list.rs` | `crates/dokime-operations/src/host/evidence/notes/list.rs` |
| `crates/dokime-operations-next/src/client/evidence_notes_set.rs` | `crates/dokime-operations/src/host/evidence/notes/set.rs` |
| `crates/dokime-operations-next/src/client/evidence_notes_watch.rs` | `crates/dokime-operations/src/host/evidence/notes/watch.rs` |
| `crates/dokime-operations-next/src/record.rs` | `crates/dokime-operations/src/host/evidence/records/append.rs` |
| `crates/dokime-operations-next/src/client/evidence_records_list.rs` | `crates/dokime-operations/src/host/evidence/records/list.rs` |
| `crates/dokime-operations-next/src/client/evidence_records_watch.rs` | `crates/dokime-operations/src/host/evidence/records/watch.rs` |
| `crates/dokime-operations-next/src/client/evidence_samples_list.rs` | `crates/dokime-operations/src/host/evidence/samples/list.rs` |
| `crates/dokime-operations-next/src/client/evidence_samples_watch.rs` | `crates/dokime-operations/src/host/evidence/samples/watch.rs` |
| `crates/dokime-operations-next/src/client/evidence_selection.rs` | `crates/dokime-operations/src/host/evidence/selection.rs` |
| `crates/dokime-operations-next/src/statistics.rs` | `crates/dokime-operations/src/host/evidence/statistics.rs` |
| `crates/dokime-operations-next/src/client/evidence_traceability_get.rs` | `crates/dokime-operations/src/host/evidence/traceability/get.rs` |
| `crates/dokime-operations-next/src/traceability_set.rs` | `crates/dokime-operations/src/host/evidence/traceability/set.rs` |
| `crates/dokime-operations-next/src/client/evidence_traceability_watch.rs` | `crates/dokime-operations/src/host/evidence/traceability/watch.rs` |
| `crates/dokime-operations-next/src/client/execution_state.rs` | `crates/dokime-operations/src/host/execution_state.rs` |
| `crates/dokime-operations-next/src/client/executions_cancel.rs` | `crates/dokime-operations/src/host/executions/cancel.rs` |
| `crates/dokime-operations-next/src/client/executions_commands_create.rs` | `crates/dokime-operations/src/host/executions/commands/create.rs` |
| `crates/dokime-operations-next/src/client/executions_commands_get.rs` | `crates/dokime-operations/src/host/executions/commands/get.rs` |
| `crates/dokime-operations-next/src/client/executions_create.rs` | `crates/dokime-operations/src/host/executions/create.rs` |
| `crates/dokime-operations-next/src/client/executions_delete.rs` | `crates/dokime-operations/src/host/executions/delete.rs` |
| `crates/dokime-operations-next/src/client/executions_get.rs` | `crates/dokime-operations/src/host/executions/get.rs` |
| `crates/dokime-operations-next/src/client/executions_list.rs` | `crates/dokime-operations/src/host/executions/list.rs` |
| `crates/dokime-operations-next/src/client/executions_parameters.rs` | `crates/dokime-operations/src/host/executions/parameters.rs` |
| `crates/dokime-operations-next/src/client/executions_progress_get.rs` | `crates/dokime-operations/src/host/executions/progress/get.rs` |
| `crates/dokime-operations-next/src/progress_set.rs` | `crates/dokime-operations/src/host/executions/progress/set.rs` |
| `crates/dokime-operations-next/src/client/executions_progress_watch.rs` | `crates/dokime-operations/src/host/executions/progress/watch.rs` |
| `crates/dokime-operations-next/src/client/executions_schemas_get.rs` | `crates/dokime-operations/src/host/executions/schemas/get.rs` |
| `crates/dokime-operations-next/src/client/executions_set.rs` | `crates/dokime-operations/src/host/executions/set.rs` |
| `crates/dokime-operations-next/src/client/executions_start.rs` | `crates/dokime-operations/src/host/executions/start.rs` |
| `crates/dokime-operations-next/src/client/executions_steps.rs` | `crates/dokime-operations/src/host/executions/steps.rs` |
| `crates/dokime-operations-next/src/client/executions_stop.rs` | `crates/dokime-operations/src/host/executions/stop.rs` |
| `crates/dokime-operations-next/src/client/executions_watch.rs` | `crates/dokime-operations/src/host/executions/watch.rs` |
| `crates/dokime-operations-next/src/client/files_delete.rs` | `crates/dokime-operations/src/host/files/delete.rs` |
| `crates/dokime-operations-next/src/client/files_get.rs` | `crates/dokime-operations/src/host/files/get.rs` |
| `crates/dokime-operations-next/src/client/files_read.rs` | `crates/dokime-operations/src/host/files/read.rs` |
| `crates/dokime-operations-next/src/client/files_transfer.rs` | `crates/dokime-operations/src/host/files/transfer.rs` |
| `crates/dokime-operations-next/src/client/files_uploads_append.rs` | `crates/dokime-operations/src/host/files/uploads/append.rs` |
| `crates/dokime-operations-next/src/client/files_uploads_create.rs` | `crates/dokime-operations/src/host/files/uploads/create.rs` |
| `crates/dokime-operations-next/src/client/files_uploads_delete.rs` | `crates/dokime-operations/src/host/files/uploads/delete.rs` |
| `crates/dokime-operations-next/src/client/files_uploads_get.rs` | `crates/dokime-operations/src/host/files/uploads/get.rs` |
| `crates/dokime-operations-next/src/client/files_uploads_publish.rs` | `crates/dokime-operations/src/host/files/uploads/publish.rs` |
| `crates/dokime-operations-next/src/client/files_uploads_resume.rs` | `crates/dokime-operations/src/host/files/uploads/resume.rs` |
| `crates/dokime-operations-next/src/client/files_watch.rs` | `crates/dokime-operations/src/host/files/watch.rs` |
| `crates/dokime-operations-next/src/client/mod.rs` | `crates/dokime-operations/src/host/mod.rs` |
| `crates/dokime-operations-next/src/client/paging.rs` | `crates/dokime-operations/src/host/paging.rs` |
| `crates/dokime-operations-next/src/client/planning.rs` | `crates/dokime-operations/src/host/planning.rs` |
| `crates/dokime-operations-next/src/client/plot_analysis.rs` | `crates/dokime-operations/src/host/plots/analysis.rs` |
| `crates/dokime-operations-next/src/client/plots_settings_get.rs` | `crates/dokime-operations/src/host/plots/settings/get.rs` |
| `crates/dokime-operations-next/src/client/plots_settings_set.rs` | `crates/dokime-operations/src/host/plots/settings/set.rs` |
| `crates/dokime-operations-next/src/client/plots_snapshots_list.rs` | `crates/dokime-operations/src/host/plots/snapshots/list.rs` |
| `crates/dokime-operations-next/src/client/plots_snapshots_watch.rs` | `crates/dokime-operations/src/host/plots/snapshots/watch.rs` |
| `crates/dokime-operations-next/src/client/projection.rs` | `crates/dokime-operations/src/host/projection.rs` |
| `crates/dokime-operations-next/src/client/catalog_prompt_definitions_get.rs` | `crates/dokime-operations/src/host/prompt_definitions/get.rs` |
| `crates/dokime-operations-next/src/client/catalog_prompt_definitions_list.rs` | `crates/dokime-operations/src/host/prompt_definitions/list.rs` |
| `crates/dokime-operations-next/src/client/prompts_answers_create.rs` | `crates/dokime-operations/src/host/prompts/answers/create.rs` |
| `crates/dokime-operations-next/src/client/prompt_catalog.rs` | `crates/dokime-operations/src/host/prompts/catalog.rs` |
| `crates/dokime-operations-next/src/prompt.rs` | `crates/dokime-operations/src/host/prompts/create.rs` |
| `crates/dokime-operations-next/src/client/prompts_get.rs` | `crates/dokime-operations/src/host/prompts/get.rs` |
| `crates/dokime-operations-next/src/client/prompts_list.rs` | `crates/dokime-operations/src/host/prompts/list.rs` |
| `crates/dokime-operations-next/src/client/prompt_validation.rs` | `crates/dokime-operations/src/host/prompts/validation.rs` |
| `crates/dokime-operations-next/src/client/prompts_watch.rs` | `crates/dokime-operations/src/host/prompts/watch.rs` |
| `crates/dokime-operations-next/src/client/catalog_record_schemas_get.rs` | `crates/dokime-operations/src/host/record_schemas/get.rs` |
| `crates/dokime-operations-next/src/client/catalog_record_schemas_list.rs` | `crates/dokime-operations/src/host/record_schemas/list.rs` |
| `crates/dokime-operations-next/src/client/reports_create.rs` | `crates/dokime-operations/src/host/reports/create.rs` |
| `crates/dokime-operations-next/src/client/reports_get.rs` | `crates/dokime-operations/src/host/reports/get.rs` |
| `crates/dokime-operations-next/src/client/reports_publish.rs` | `crates/dokime-operations/src/host/reports/publish.rs` |
| `crates/dokime-operations-next/src/client/secrets_delete.rs` | `crates/dokime-operations/src/host/secrets/delete.rs` |
| `crates/dokime-operations-next/src/client/secrets_export.rs` | `crates/dokime-operations/src/host/secrets/export.rs` |
| `crates/dokime-operations-next/src/client/secrets_get.rs` | `crates/dokime-operations/src/host/secrets/get.rs` |
| `crates/dokime-operations-next/src/client/secrets_import.rs` | `crates/dokime-operations/src/host/secrets/import.rs` |
| `crates/dokime-operations-next/src/client/secrets_list.rs` | `crates/dokime-operations/src/host/secrets/list.rs` |
| `crates/dokime-operations-next/src/client/secrets_set.rs` | `crates/dokime-operations/src/host/secrets/set.rs` |
| `crates/dokime-operations-next/src/client/secrets_store.rs` | `crates/dokime-operations/src/host/secrets/store.rs` |
| `crates/dokime-operations-next/src/client/secrets_watch.rs` | `crates/dokime-operations/src/host/secrets/watch.rs` |
| `crates/dokime-operations-next/src/client/settings_get.rs` | `crates/dokime-operations/src/host/settings/get.rs` |
| `crates/dokime-operations-next/src/client/settings_preferences.rs` | `crates/dokime-operations/src/host/settings/preferences.rs` |
| `crates/dokime-operations-next/src/client/settings_schema.rs` | `crates/dokime-operations/src/host/settings/schema.rs` |
| `crates/dokime-operations-next/src/client/settings_set.rs` | `crates/dokime-operations/src/host/settings/set.rs` |
| `crates/dokime-operations-next/src/client/storage_sources_get.rs` | `crates/dokime-operations/src/host/storage/sources/get.rs` |
| `crates/dokime-operations-next/src/client/storage_sources_set.rs` | `crates/dokime-operations/src/host/storage/sources/set.rs` |
| `crates/dokime-operations-next/src/client/catalog_stream_schemas_get.rs` | `crates/dokime-operations/src/host/stream_schemas/get.rs` |
| `crates/dokime-operations-next/src/client/catalog_stream_schemas_list.rs` | `crates/dokime-operations/src/host/stream_schemas/list.rs` |
| `crates/dokime-operations-next/src/client/catalog_suites_get.rs` | `crates/dokime-operations/src/host/suites/get.rs` |
| `crates/dokime-operations-next/src/client/catalog_suites_list.rs` | `crates/dokime-operations/src/host/suites/list.rs` |
| `crates/dokime-operations-next/src/client/views.rs` | `crates/dokime-operations/src/host/views.rs` |
| `crates/dokime-operations-next/src/lib.rs` | `crates/dokime-operations/src/lib.rs` |
| `crates/dokime-operations-next/src/resources.rs` | `crates/dokime-operations/src/resources.rs` |

### Direct Store Consumer Inventory

These files still contain Store imports/access and require item 7 of the operations/
runtime work list. A file may retain product lifecycle logic, but its persistence
must become a library native operation call. The storage backend itself is excluded
because physical I/O belongs there. The inventory is source inspection, not test proof.

`crates/dokime-operations/src/` (91 files):

- `host/backend/appearance.rs`
- `host/backend/environment.rs`
- `host/backend/overview/get.rs`
- `host/backend/overview/watch.rs`
- `host/cases/create.rs`
- `host/cases/delete.rs`
- `host/cases/watch.rs`
- `host/catalog_inventory.rs`
- `host/content/artifacts/create.rs`
- `host/content/artifacts/get.rs`
- `host/content/artifacts/list.rs`
- `host/content/artifacts/watch.rs`
- `host/content/execution_files/get.rs`
- `host/content/execution_files/list.rs`
- `host/content/execution_files/watch.rs`
- `host/cycles/create.rs`
- `host/cycles/get.rs`
- `host/cycles/seal.rs`
- `host/cycles/set.rs`
- `host/cycles/watch.rs`
- `host/documents/expansion.rs`
- `host/documents/get.rs`
- `host/documents/plot.rs`
- `host/documents/watch.rs`
- `host/evidence/archive.rs`
- `host/evidence/health_events/watch.rs`
- `host/evidence/notes/create.rs`
- `host/evidence/notes/get.rs`
- `host/evidence/notes/list.rs`
- `host/evidence/notes/set.rs`
- `host/evidence/notes/watch.rs`
- `host/evidence/records/append.rs`
- `host/evidence/records/list.rs`
- `host/evidence/records/watch.rs`
- `host/evidence/samples/list.rs`
- `host/evidence/samples/watch.rs`
- `host/evidence/statistics.rs`
- `host/evidence/traceability/get.rs`
- `host/evidence/traceability/set.rs`
- `host/evidence/traceability/watch.rs`
- `host/execution_state.rs`
- `host/executions/commands/get.rs`
- `host/executions/create.rs`
- `host/executions/delete.rs`
- `host/executions/get.rs`
- `host/executions/list.rs`
- `host/executions/parameters.rs`
- `host/executions/progress/get.rs`
- `host/executions/progress/set.rs`
- `host/executions/progress/watch.rs`
- `host/executions/schemas/get.rs`
- `host/executions/set.rs`
- `host/executions/steps.rs`
- `host/executions/watch.rs`
- `host/files/delete.rs`
- `host/files/transfer.rs`
- `host/files/uploads/append.rs`
- `host/files/uploads/create.rs`
- `host/files/uploads/delete.rs`
- `host/files/uploads/get.rs`
- `host/files/uploads/publish.rs`
- `host/files/uploads/resume.rs`
- `host/files/watch.rs`
- `host/mod.rs`
- `host/planning.rs`
- `host/plots/settings/get.rs`
- `host/plots/settings/set.rs`
- `host/plots/snapshots/list.rs`
- `host/plots/snapshots/watch.rs`
- `host/prompts/answers/create.rs`
- `host/prompts/catalog.rs`
- `host/prompts/list.rs`
- `host/prompts/watch.rs`
- `host/record_schemas/get.rs`
- `host/reports/create.rs`
- `host/reports/get.rs`
- `host/reports/publish.rs`
- `host/secrets/delete.rs`
- `host/secrets/export.rs`
- `host/secrets/get.rs`
- `host/secrets/import.rs`
- `host/secrets/list.rs`
- `host/secrets/set.rs`
- `host/secrets/store.rs`
- `host/secrets/watch.rs`
- `host/settings/preferences.rs`
- `host/settings/set.rs`
- `host/storage/sources/get.rs`
- `host/storage/sources/set.rs`
- `host/views.rs`
- `resources.rs`

`crates/dokime-runtime/src/` (12 files):

- `capabilities.rs`
- `documents.rs`
- `execution.rs`
- `execution_manager.rs`
- `host.rs`
- `prompts.rs`
- `reports.rs`
- `run_control.rs`
- `services/access_credentials.rs`
- `services/mod.rs`
- `services/state.rs`
- `streams.rs`
