# New Execution Host

The execution host serves the command-line tool and SDK clients.
`dokime-runtime-next` composes plugin entries, scoped unary lifecycle dispatch,
controlled orchestration time and cycle execution. `dokime-operations-next` owns
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
`crates/dokime-runtime-next/tests/system/manager_lifecycles.rs`.
Existing restart, retry, report, evidence and client stories retain their contracts.

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
reads their records through the client. The system target covers every host story,
including publication, interrupted recovery and every language-equivalence story.
Every story uses `SYSTEM_HANG_GUARD` from `tests/support/settings.rs`; operation
policies retain their own deadlines.

## Command-Line Candidate

`crates/dokime-cli-next` builds the current `dokime-next` tool.
Hosting embeds the new runtime listener; remote commands use `sdk/rust`'s client
half. Build with `mise run build -- --package dokime-cli-next`.

`dokime-next --endpoint URL operation list --json` returns the complete bounded
admitted operation catalog, including owner, generation and contract. It has no
paging flags. `operation show ID --json` inspects the same contract, including
visibility, policy, protobuf descriptors and readable request/response field shapes.
`operation call ID --json '{}'` validates a unary Public request using its admitted
descriptors and calls its admitted gRPC mapping through the SDK generic transport,
then prints the typed JSON response. Use `--input-file request.json` for file input. Stream contracts require the typed streaming client;
refusals name the requested operation. Policies run on the host; the CLI adds no
mutation retries.

`dokime-next version --json` reports the compiled package version, source revision
(with `-dirty` for tracked local edits), publication channel, OS and architecture.
Local builds have a null channel; release and Nightly publishers identify theirs
from the build environment. Source archives without Git or CI revision metadata
report a null revision.

`dokime-next doctor storage --json` probes write, sync, rename, readback and removal
in a disposable scratch directory, then opens and closes a scratch host.
`--storage PATH` (or configured `storage`) additionally diagnoses an existing root
from a bounded copy of its SQLite files under the root lease. Stop the host first.
The original root is never opened for recovery or changed; missing roots are not
created. `retained_bytes` bounds the database copy. Incompatible roots produce a
failed report and backup/fresh-root or explicit-reset guidance.

Export access with `dokime-next --endpoint URL access pairing export --label Operator
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
dokime-next new first-plugin --lang rust
dokime-next new remote-plugin --lang python
dokime-next new managed-plugin --lang csharp
dokime-next new native-plugin --lang cpp
dokime-next plugin bundle --executable first-plugin/target/debug/dokime-starter --out first.dokime
dokime-next plugin inspect --bundle first.dokime --json
dokime-next --endpoint 127.0.0.1:7000 server --storage ./evidence
dokime-next --endpoint http://127.0.0.1:7000 plugin install first.dokime
dokime-next --endpoint http://127.0.0.1:7000 plugin add http://127.0.0.1:7001
dokime-next --endpoint http://127.0.0.1:7000 catalog list --json
dokime-next --endpoint http://127.0.0.1:7000 plan materialize --suite SUITE --template example.capture --out cases.pb
dokime-next --endpoint http://127.0.0.1:7000 cycle create --cases cases.pb --json
dokime-next --endpoint http://127.0.0.1:7000 execution prompt next EXECUTION --json
dokime-next --endpoint http://127.0.0.1:7000 execution prompt answer EXECUTION --action yes --accepted
dokime-next --endpoint http://127.0.0.1:7000 execution watch EXECUTION
dokime-next --endpoint http://127.0.0.1:7000 execution report EXECUTION --out report.html
dokime-next proto export --out proto
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
plugin. Hosting uses the same EOF/interrupt close contract as `dokime-host-next`.

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
mise run test -- --package dokime-cli-next --target system
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
mise run test -- --package dokime-cli-next --target system
```

The fixture producer uses `dokime-next new`, builds that exact generated Rust
project through its mise task against the local SDK candidate, and exports its
sources/executable with Tests. Stories compare generated source bytes and never
compile plugins. Tests Build includes this producer.
Installed binary renaming and desktop integration remain on the combined restoration checklist. Installer packaging is not selected in this candidate.

## Canonical Selection Requests

`dokime-next plan preview --input-file preview.json --json` accepts a
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
Python `examples/thermal/suite.py`, C# `tests/SystemPlugin` and C++ `tests/system`.
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
runtime. C++ builds Harmos's pinned dependencies; `HARMOS_SOURCE` and
`DOKIME_HARMOS_CACHE` can select existing workstation inputs. Tests never compile
fixtures. Local Tests Build includes every available compiled SDK fixture and the
full selection runs every story. Pull requests build Rust and Python and
run every story except the C# and C++ equivalence stories, which run in Nightly. Nightly
builds each language independently on Linux and Windows, caching the C++ SDK's
pinned native dependencies and installing .NET through mise, then runs every
story against exported artifacts. Suite failures are reported alongside the
publication result and do not block publication.

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
mise run build -- --cargo build --package dokime-runtime-next --example content_fixture
mise run test -- --package dokime-runtime-next --target system --exact client_transfers_execution_content::client_transfers_execution_content
mise run test -- --package dokime-runtime-next --target system --exact client_keeps_execution_notes::client_keeps_execution_notes
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

## Context Protocol Packages

Plugin child calls for records, logs, statistics, prompts, document parts,
file capture and environment use `dokime.context.v1` and its nested packages.
Their wrappers are `<Service><Method>Request/Response`; identities are
`package.Service/Method`. These declarations add no client gRPC entry methods.
Environment and Time.Get now resolve through the active invocation and the
host clock. Document and file-capture declarations still need their remaining
handlers; the grammar task list retains those obligations.

Progress uses distinct context `dokime.context.v1.executions.Progress.Get/Set`
wrappers. Get returns the complete writable `ExecutionProgressValue` and revision;
Set replaces completed, total and label together, with an expected revision and
caller-retained request ID. Reusing an unchanged request returns its original
durable receipt, while stale revisions and changed requests conflict. SDK progress
conveniences perform Get then Set and require both declarations. Clients have no Set.
Returned progress positions survive restart until execution deletion; foreign or
unissued positions are refused. `client_observes_revisioned_progress` exercises
the plugin and client journey through the descriptor-derived client hierarchy.

Traceability uses context `dokime.context.v1.evidence.Traceability.Get/Set`.
Each entry name owns its complete metadata, scalar value, source and unit, with an
independent expected revision and semantic request ID. Set replaces the whole
entry; an unchanged resend returns the original durable receipt. Clients use
`client.evidence.traceability.get(request)` for ordered snapshots, bounded by
both page rows and bytes, and `.watch(request)` for an opening followed by durable
changes. Pages retain their scope and returned `ReadCut`; continuation tokens
bind both. A read cut contains one position per selected execution; scopes whose
compound cut cannot fit the configured reply byte bound are refused with narrowing guidance.
Step, criterion, template, suite, case and origin filters retain frozen attribution.
Issued positions remain readable across restart until execution deletion. An
observer's cancellation leaves its execution running. The plugin journey is
`client_observes_revisioned_traceability`.

Lifecycle identities remain independent Protected dotted bindings on Instance.Invoke.
The plugin face now owns their five shared request/response pairs and declares no
second dispatch service. Context Artifacts.Create attaches published content;
Effects.Create/Get/Set declares durable inverse preparation and whole-value
confirmation under revision and request identity. Records.Append, Logs.Append
and Documents.AppendParts carry semantic request identities and durable receipts;
Records.List and Statistics carry returned durable positions. The definition
pass does not implement their remaining host handlers. Existing settings/secret
and artifact-transfer payloads remain until HostValues/plugin/storage composition.
The work order below names every final face contract and pending composition.

## Present Operation to Grammar Name

This is the consumer work order for the definition pass. Wire identities are
`package.Service/Method`; wrappers are qualified below. All automatic `Run`
twins and the superseded mixed client files are removed. `done` identifies the
earlier Progress and Traceability groups; `moved` identifies this definition
pass, including composed Status and Access replacements; `awaits composition`
identifies library-owned operations whose ordinary product declaration remains.
These statuses describe definitions, not migrated or building consumers.

Library services are consumed from their pinned owner, never redeclared at their
final identities. The host already composes Status and Access. Plugins, content
transfers and invocation-caller HostValues admission still need composition.
Their remaining ordinary declarations are in `client_pending_plugins.proto`,
`client_pending_content.proto`, `execution.proto` and `host.proto`. The two pending
files contain only uncomposed library destinations; the generator excludes them
from composed SDK roots to avoid duplicate library resource paths. No Run twins
are retained there. Time is already library-owned and has no product declaration.

| Present Operation | Final Wire Identity or Lifecycle Binding | Request | Response | Face | State |
| --- | --- | --- | --- | --- | --- |
| `Appearance.GetAppearance` | `dokime.v1.Backend/Appearance` | `dokime.v1.BackendAppearanceRequest` | `dokime.v1.BackendAppearanceResponse` | Client | moved |
| `Backend.CollectDiagnostics` | `harmos.v1.status.Diagnostics/Get` | `harmos.v1.status.DiagnosticsGetRequest` | `harmos.v1.status.DiagnosticsGetResponse` | Client | moved |
| `Backend.GetAvailability` | `harmos.v1.Status/Get` | `harmos.v1.StatusGetRequest` | `harmos.v1.StatusGetResponse` | Client | moved |
| `Backend.GetHealth` | `harmos.v1.Status/Get` | `harmos.v1.StatusGetRequest` | `harmos.v1.StatusGetResponse` | Client | moved |
| `Backend.GetIdentity` | `harmos.v1.Status/Get` | `harmos.v1.StatusGetRequest` | `harmos.v1.StatusGetResponse` | Client | moved |
| `Backend.WatchLogs` | `harmos.v1.status.Logs/Watch` | `harmos.v1.status.LogsWatchRequest` | `harmos.v1.status.LogsWatchResponse` | Client | moved |
| `Backend.WatchTrace` | `harmos.v1.status.TraceEvents/Watch` | `harmos.v1.status.TraceEventsWatchRequest` | `harmos.v1.status.TraceEventsWatchResponse` | Client | moved |
| `Catalog.GetRecordSchema` | `dokime.v1.catalog.RecordSchemas/Get` | `dokime.v1.catalog.RecordSchemasGetRequest` | `dokime.v1.catalog.RecordSchemasGetResponse` | Client | moved |
| `Catalog.GetSuite` | `dokime.v1.catalog.Suites/Get` | `dokime.v1.catalog.SuitesGetRequest` | `dokime.v1.catalog.SuitesGetResponse` | Client | moved |
| `Catalog.GetTelemetrySchema` | `dokime.v1.catalog.StreamSchemas/Get` | `dokime.v1.catalog.StreamSchemasGetRequest` | `dokime.v1.catalog.StreamSchemasGetResponse` | Client | moved |
| `Catalog.ListRecordSchemas` | `dokime.v1.catalog.RecordSchemas/List` | `dokime.v1.catalog.RecordSchemasListRequest` | `dokime.v1.catalog.RecordSchemasListResponse` | Client | moved |
| `Catalog.ListSuites` | `dokime.v1.catalog.Suites/List` | `dokime.v1.catalog.SuitesListRequest` | `dokime.v1.catalog.SuitesListResponse` | Client | moved |
| `Catalog.ListTelemetrySchemas` | `dokime.v1.catalog.StreamSchemas/List` | `dokime.v1.catalog.StreamSchemasListRequest` | `dokime.v1.catalog.StreamSchemasListResponse` | Client | moved |
| `Content.DownloadAppearanceAsset` | `harmos.context.v1.Storage/GetFileBytes` | `harmos.context.v1.StorageGetFileBytesRequest` | `harmos.context.v1.StorageGetFileBytesResponse` | Client | awaits composition |
| `Content.DownloadArtifact` | `dokime.v1.content.Artifacts/Get` | `dokime.v1.content.ArtifactsGetRequest` | `dokime.v1.content.ArtifactsGetResponse` | Client | moved |
| `Content.DownloadArtifact` | `harmos.context.v1.Storage/GetFileBytes` | `harmos.context.v1.StorageGetFileBytesRequest` | `harmos.context.v1.StorageGetFileBytesResponse` | Client | awaits composition |
| `Content.DownloadEvidence` | `dokime.v1.Evidence/Archive` | `dokime.v1.EvidenceArchiveRequest` | `dokime.v1.EvidenceArchiveResponse` | Client | moved |
| `Content.DownloadEvidence` | `harmos.context.v1.Storage/GetFileBytes` | `harmos.context.v1.StorageGetFileBytesRequest` | `harmos.context.v1.StorageGetFileBytesResponse` | Client | awaits composition |
| `Content.DownloadExecutionFile` | `dokime.v1.content.ExecutionFiles/Get` | `dokime.v1.content.ExecutionFilesGetRequest` | `dokime.v1.content.ExecutionFilesGetResponse` | Client | moved |
| `Content.DownloadExecutionFile` | `harmos.context.v1.Storage/GetFileBytes` | `harmos.context.v1.StorageGetFileBytesRequest` | `harmos.context.v1.StorageGetFileBytesResponse` | Client | awaits composition |
| `Content.FollowExecutionFile` | `dokime.v1.content.ExecutionFiles/Get` | `dokime.v1.content.ExecutionFilesGetRequest` | `dokime.v1.content.ExecutionFilesGetResponse` | Client | moved |
| `Content.FollowExecutionFile` | `harmos.context.v1.Storage/WatchFileBytes` | `harmos.context.v1.StorageWatchFileBytesRequest` | `harmos.context.v1.StorageWatchFileBytesResponse` | Client | awaits composition |
| `Content.ListArtifacts` | `dokime.v1.content.Artifacts/List` | `dokime.v1.content.ArtifactsListRequest` | `dokime.v1.content.ArtifactsListResponse` | Client | moved |
| `Content.ListExecutionFiles` | `dokime.v1.content.ExecutionFiles/List` | `dokime.v1.content.ExecutionFilesListRequest` | `dokime.v1.content.ExecutionFilesListResponse` | Client | moved |
| `Content.ReadExecutionFile` | `dokime.v1.content.ExecutionFiles/Get` | `dokime.v1.content.ExecutionFilesGetRequest` | `dokime.v1.content.ExecutionFilesGetResponse` | Client | moved |
| `Content.UploadArtifact` | `harmos.context.v1.Storage/CreateFileProduction` | `harmos.context.v1.StorageCreateFileProductionRequest` | `harmos.context.v1.StorageCreateFileProductionResponse` | Client | awaits composition |
| `Content.UploadArtifact` | `harmos.context.v1.Storage/GetFileProduction` | `harmos.context.v1.StorageGetFileProductionRequest` | `harmos.context.v1.StorageGetFileProductionResponse` | Client | awaits composition |
| `Content.UploadArtifact` | `harmos.context.v1.Storage/CreateFileWriter` | `harmos.context.v1.StorageCreateFileWriterRequest` | `harmos.context.v1.StorageCreateFileWriterResponse` | Client | awaits composition |
| `Content.UploadArtifact` | `harmos.context.v1.Storage/AppendFileBytes` | `harmos.context.v1.StorageAppendFileBytesRequest` | `harmos.context.v1.StorageAppendFileBytesResponse` | Client | awaits composition |
| `Content.UploadArtifact` | `harmos.context.v1.Storage/DeleteFileProduction` | `harmos.context.v1.StorageDeleteFileProductionRequest` | `harmos.context.v1.StorageDeleteFileProductionResponse` | Client | awaits composition |
| `Content.UploadArtifact` | `harmos.context.v1.Storage/PublishFileProduction` | `harmos.context.v1.StoragePublishFileProductionRequest` | `harmos.context.v1.StoragePublishFileProductionResponse` | Client | awaits composition |
| `Content.UploadArtifact` | `dokime.v1.content.Artifacts/Create` | `dokime.v1.content.ArtifactsCreateRequest` | `dokime.v1.content.ArtifactsCreateResponse` | Client | moved |
| `Documents.CaptureCurrentDocument` | `dokime.v1.Documents/Get` | `dokime.v1.DocumentsGetRequest` | `dokime.v1.DocumentsGetResponse` | Client | moved |
| `Documents.UpdatePlotSettings` | `dokime.v1.plots.Settings/Set` | `dokime.v1.plots.SettingsSetRequest` | `dokime.v1.plots.SettingsSetResponse` | Client | moved |
| `Evidence.GetCapturedRecordSchema` | `dokime.v1.catalog.RecordSchemas/Get` | `dokime.v1.catalog.RecordSchemasGetRequest` | `dokime.v1.catalog.RecordSchemasGetResponse` | Client | moved |
| `Evidence.ListRecords` | `dokime.v1.evidence.Records/List` | `dokime.v1.evidence.RecordsListRequest` | `dokime.v1.evidence.RecordsListResponse` | Client | moved |
| `Evidence.WatchRecords` | `dokime.v1.evidence.Records/Watch` | `dokime.v1.evidence.RecordsWatchRequest` | `dokime.v1.evidence.RecordsWatchResponse` | Client | moved |
| `Execution.AppendExecutions` | `dokime.v1.Executions/Create` | `dokime.v1.ExecutionsCreateRequest` | `dokime.v1.ExecutionsCreateResponse` | Client | moved |
| `Execution.CancelExecution` | `dokime.v1.Executions/Cancel` | `dokime.v1.ExecutionsCancelRequest` | `dokime.v1.ExecutionsCancelResponse` | Client | moved |
| `Execution.CreateCycle` | `dokime.v1.Cycles/Create` | `dokime.v1.CyclesCreateRequest` | `dokime.v1.CyclesCreateResponse` | Client | moved |
| `Execution.DeleteCase` | `dokime.v1.Cases/Delete` | `dokime.v1.CasesDeleteRequest` | `dokime.v1.CasesDeleteResponse` | Client | moved |
| `Execution.GetCase` | `dokime.v1.Cases/Get` | `dokime.v1.CasesGetRequest` | `dokime.v1.CasesGetResponse` | Client | moved |
| `Execution.GetCycle` | `dokime.v1.Cycles/Get` | `dokime.v1.CyclesGetRequest` | `dokime.v1.CyclesGetResponse` | Client | moved |
| `Execution.GetExecution` | `dokime.v1.Executions/Get` | `dokime.v1.ExecutionsGetRequest` | `dokime.v1.ExecutionsGetResponse` | Client | moved |
| `Execution.GetExecutionCommand` | `dokime.v1.executions.Commands/Get` | `dokime.v1.executions.CommandsGetRequest` | `dokime.v1.executions.CommandsGetResponse` | Client | moved |
| `Execution.ListCaseParameters` | `dokime.v1.Cases/Parameters` | `dokime.v1.CasesParametersRequest` | `dokime.v1.CasesParametersResponse` | Client | moved |
| `Execution.ListCases` | `dokime.v1.Cases/List` | `dokime.v1.CasesListRequest` | `dokime.v1.CasesListResponse` | Client | moved |
| `Execution.ListCycles` | `dokime.v1.Cycles/List` | `dokime.v1.CyclesListRequest` | `dokime.v1.CyclesListResponse` | Client | moved |
| `Execution.ListExecutionParameters` | `dokime.v1.Executions/Parameters` | `dokime.v1.ExecutionsParametersRequest` | `dokime.v1.ExecutionsParametersResponse` | Client | moved |
| `Execution.ListExecutionSteps` | `dokime.v1.Executions/Steps` | `dokime.v1.ExecutionsStepsRequest` | `dokime.v1.ExecutionsStepsResponse` | Client | moved |
| `Execution.ListExecutions` | `dokime.v1.Executions/List` | `dokime.v1.ExecutionsListRequest` | `dokime.v1.ExecutionsListResponse` | Client | moved |
| `Execution.PauseCycle` | `dokime.v1.Cycles/Set` | `dokime.v1.CyclesSetRequest` | `dokime.v1.CyclesSetResponse` | Client | moved |
| `Execution.PauseExecution` | `dokime.v1.Executions/Set` | `dokime.v1.ExecutionsSetRequest` | `dokime.v1.ExecutionsSetResponse` | Client | moved |
| `Execution.RenameCycle` | `dokime.v1.Cycles/Set` | `dokime.v1.CyclesSetRequest` | `dokime.v1.CyclesSetResponse` | Client | moved |
| `Execution.RenameExecution` | `dokime.v1.Executions/Set` | `dokime.v1.ExecutionsSetRequest` | `dokime.v1.ExecutionsSetResponse` | Client | moved |
| `Execution.RerunCycle` | `dokime.v1.Cycles/Create` | `dokime.v1.CyclesCreateRequest` | `dokime.v1.CyclesCreateResponse` | Client | moved |
| `Execution.ResumeCycle` | `dokime.v1.Cycles/Set` | `dokime.v1.CyclesSetRequest` | `dokime.v1.CyclesSetResponse` | Client | moved |
| `Execution.ResumeExecution` | `dokime.v1.Executions/Set` | `dokime.v1.ExecutionsSetRequest` | `dokime.v1.ExecutionsSetResponse` | Client | moved |
| `Execution.SealCycle` | `dokime.v1.Cycles/Seal` | `dokime.v1.CyclesSealRequest` | `dokime.v1.CyclesSealResponse` | Client | moved |
| `Execution.SetCycleArchived` | `dokime.v1.Cycles/Set` | `dokime.v1.CyclesSetRequest` | `dokime.v1.CyclesSetResponse` | Client | moved |
| `Execution.SetExecutionArchived` | `dokime.v1.Executions/Set` | `dokime.v1.ExecutionsSetRequest` | `dokime.v1.ExecutionsSetResponse` | Client | moved |
| `Execution.StartExecution` | `dokime.v1.Executions/Start` | `dokime.v1.ExecutionsStartRequest` | `dokime.v1.ExecutionsStartResponse` | Client | moved |
| `Execution.StopExecution` | `dokime.v1.Executions/Stop` | `dokime.v1.ExecutionsStopRequest` | `dokime.v1.ExecutionsStopResponse` | Client | moved |
| `Execution.SubmitExecutionCommand` | `dokime.v1.executions.Commands/Create` | `dokime.v1.executions.CommandsCreateRequest` | `dokime.v1.executions.CommandsCreateResponse` | Client | moved |
| `Execution.UpdateCyclePolicy` | `dokime.v1.Cycles/Set` | `dokime.v1.CyclesSetRequest` | `dokime.v1.CyclesSetResponse` | Client | moved |
| `Health.ListHealth` | `dokime.v1.evidence.HealthEvents/List` | `dokime.v1.evidence.HealthEventsListRequest` | `dokime.v1.evidence.HealthEventsListResponse` | Client | moved |
| `History.BrowseCycles` | `dokime.v1.Cycles/List` | `dokime.v1.CyclesListRequest` | `dokime.v1.CyclesListResponse` | Client | moved |
| `History.BrowseExecutions` | `dokime.v1.Executions/List` | `dokime.v1.ExecutionsListRequest` | `dokime.v1.ExecutionsListResponse` | Client | moved |
| `History.DeleteExecution` | `dokime.v1.Executions/Delete` | `dokime.v1.ExecutionsDeleteRequest` | `dokime.v1.ExecutionsDeleteResponse` | Client | moved |
| `History.DownloadCheckpointArtifact` | `dokime.v1.content.Artifacts/Get` | `dokime.v1.content.ArtifactsGetRequest` | `dokime.v1.content.ArtifactsGetResponse` | Client | moved |
| `History.DownloadCheckpointArtifact` | `harmos.context.v1.Storage/GetFileBytes` | `harmos.context.v1.StorageGetFileBytesRequest` | `harmos.context.v1.StorageGetFileBytesResponse` | Client | awaits composition |
| `History.GetCheckpointContext` | `dokime.v1.Executions/Get` | `dokime.v1.ExecutionsGetRequest` | `dokime.v1.ExecutionsGetResponse` | Client | moved |
| `History.GetCheckpointContext` | `dokime.v1.Cycles/Get` | `dokime.v1.CyclesGetRequest` | `dokime.v1.CyclesGetResponse` | Client | moved |
| `History.GetCheckpointContext` | `dokime.v1.Documents/Get` | `dokime.v1.DocumentsGetRequest` | `dokime.v1.DocumentsGetResponse` | Client | moved |
| `History.GetCheckpointContext` | `dokime.v1.Prompts/List` | `dokime.v1.PromptsListRequest` | `dokime.v1.PromptsListResponse` | Client | moved |
| `History.GetCheckpointContext` | `dokime.v1.evidence.HealthEvents/List` | `dokime.v1.evidence.HealthEventsListRequest` | `dokime.v1.evidence.HealthEventsListResponse` | Client | moved |
| `History.GetCheckpointContext` | `dokime.v1.evidence.Notes/List` | `dokime.v1.evidence.NotesListRequest` | `dokime.v1.evidence.NotesListResponse` | Client | moved |
| `History.GetCheckpointContext` | `dokime.v1.evidence.Traceability/Get` | `dokime.v1.evidence.TraceabilityGetRequest` | `dokime.v1.evidence.TraceabilityGetResponse` | Client | done |
| `History.GetCheckpointContext` | `dokime.v1.Backend/Appearance` | `dokime.v1.BackendAppearanceRequest` | `dokime.v1.BackendAppearanceResponse` | Client | moved |
| `History.GetCheckpointDocument` | `dokime.v1.Documents/Get` | `dokime.v1.DocumentsGetRequest` | `dokime.v1.DocumentsGetResponse` | Client | moved |
| `History.GetCheckpointEvidence` | `dokime.v1.content.Artifacts/List` | `dokime.v1.content.ArtifactsListRequest` | `dokime.v1.content.ArtifactsListResponse` | Client | moved |
| `History.GetCheckpointEvidence` | `dokime.v1.content.ExecutionFiles/List` | `dokime.v1.content.ExecutionFilesListRequest` | `dokime.v1.content.ExecutionFilesListResponse` | Client | moved |
| `History.GetCheckpointEvidence` | `dokime.v1.catalog.RecordSchemas/List` | `dokime.v1.catalog.RecordSchemasListRequest` | `dokime.v1.catalog.RecordSchemasListResponse` | Client | moved |
| `History.GetCheckpointEvidence` | `dokime.v1.catalog.StreamSchemas/List` | `dokime.v1.catalog.StreamSchemasListRequest` | `dokime.v1.catalog.StreamSchemasListResponse` | Client | moved |
| `History.GetExecutionCheckpoint` | `dokime.v1.Executions/Get` | `dokime.v1.ExecutionsGetRequest` | `dokime.v1.ExecutionsGetResponse` | Client | moved |
| `History.ListCheckpointRecords` | `dokime.v1.evidence.Records/List` | `dokime.v1.evidence.RecordsListRequest` | `dokime.v1.evidence.RecordsListResponse` | Client | moved |
| `History.ListCheckpointTelemetry` | `dokime.v1.evidence.Samples/List` | `dokime.v1.evidence.SamplesListRequest` | `dokime.v1.evidence.SamplesListResponse` | Client | moved |
| `History.ReadCheckpointOutput` | `dokime.v1.content.ExecutionFiles/Get` | `dokime.v1.content.ExecutionFilesGetRequest` | `dokime.v1.content.ExecutionFilesGetResponse` | Client | moved |
| `Media.AudioRecording` | `dokime.v1.media.Captures/Create` | `dokime.v1.media.CapturesCreateRequest` | `dokime.v1.media.CapturesCreateResponse` | Client | moved |
| `Media.AudioRecording` | `dokime.v1.media.Captures/Watch` | `dokime.v1.media.CapturesWatchRequest` | `dokime.v1.media.CapturesWatchResponse` | Client | moved |
| `Media.AudioRecording` | `dokime.v1.media.Captures/Stop` | `dokime.v1.media.CapturesStopRequest` | `dokime.v1.media.CapturesStopResponse` | Client | moved |
| `Media.CameraRecording` | `dokime.v1.media.Captures/Create` | `dokime.v1.media.CapturesCreateRequest` | `dokime.v1.media.CapturesCreateResponse` | Client | moved |
| `Media.CameraRecording` | `dokime.v1.media.Captures/Watch` | `dokime.v1.media.CapturesWatchRequest` | `dokime.v1.media.CapturesWatchResponse` | Client | moved |
| `Media.CameraRecording` | `dokime.v1.media.Captures/Stop` | `dokime.v1.media.CapturesStopRequest` | `dokime.v1.media.CapturesStopResponse` | Client | moved |
| `Media.Photo` | `dokime.v1.media.Captures/Create` | `dokime.v1.media.CapturesCreateRequest` | `dokime.v1.media.CapturesCreateResponse` | Client | moved |
| `Media.ScreenRecording` | `dokime.v1.media.Captures/Create` | `dokime.v1.media.CapturesCreateRequest` | `dokime.v1.media.CapturesCreateResponse` | Client | moved |
| `Media.ScreenRecording` | `dokime.v1.media.Captures/Watch` | `dokime.v1.media.CapturesWatchRequest` | `dokime.v1.media.CapturesWatchResponse` | Client | moved |
| `Media.ScreenRecording` | `dokime.v1.media.Captures/Stop` | `dokime.v1.media.CapturesStopRequest` | `dokime.v1.media.CapturesStopResponse` | Client | moved |
| `Media.Screenshot` | `dokime.v1.media.Captures/Create` | `dokime.v1.media.CapturesCreateRequest` | `dokime.v1.media.CapturesCreateResponse` | Client | moved |
| `Notes.CreateNote` | `dokime.v1.evidence.Notes/Create` | `dokime.v1.evidence.NotesCreateRequest` | `dokime.v1.evidence.NotesCreateResponse` | Client | moved |
| `Notes.EditNote` | `dokime.v1.evidence.Notes/Set` | `dokime.v1.evidence.NotesSetRequest` | `dokime.v1.evidence.NotesSetResponse` | Client | moved |
| `Notes.ListNotes` | `dokime.v1.evidence.Notes/List` | `dokime.v1.evidence.NotesListRequest` | `dokime.v1.evidence.NotesListResponse` | Client | moved |
| `Observation.GetRuntimeShell` | `dokime.v1.backend.Overview/Get` | `dokime.v1.backend.OverviewGetRequest` | `dokime.v1.backend.OverviewGetResponse` | Client | moved |
| `Observation.GetTraceability` | `dokime.v1.evidence.Traceability/Get` | `dokime.v1.evidence.TraceabilityGetRequest` | `dokime.v1.evidence.TraceabilityGetResponse` | Client | done |
| `Observation.WatchExecutionProgress` | `dokime.v1.executions.Progress/Watch` | `dokime.v1.executions.ProgressWatchRequest` | `dokime.v1.executions.ProgressWatchResponse` | Client | done |
| `Observation.WatchRuntime` | `dokime.v1.backend.Overview/Watch` | `dokime.v1.backend.OverviewWatchRequest` | `dokime.v1.backend.OverviewWatchResponse` | Client | moved |
| `Planning.MaterializeTemplate` | `dokime.v1.Cases/Create` | `dokime.v1.CasesCreateRequest` | `dokime.v1.CasesCreateResponse` | Client | moved |
| `Planning.PreviewSelection` | `dokime.v1.Cases/Create` | `dokime.v1.CasesCreateRequest` | `dokime.v1.CasesCreateResponse` | Client | moved |
| `Planning.RegisterSelection` | `dokime.v1.Cases/Create` | `dokime.v1.CasesCreateRequest` | `dokime.v1.CasesCreateResponse` | Client | moved |
| `Plots.ListAnalyses` | `dokime.v1.Catalog/Analyses` | `dokime.v1.CatalogAnalysesRequest` | `dokime.v1.CatalogAnalysesResponse` | Client | moved |
| `Plots.QuerySnapshots` | `dokime.v1.plots.Snapshots/List` | `dokime.v1.plots.SnapshotsListRequest` | `dokime.v1.plots.SnapshotsListResponse` | Client | moved |
| `Plots.Watch` | `dokime.v1.plots.Snapshots/Watch` | `dokime.v1.plots.SnapshotsWatchRequest` | `dokime.v1.plots.SnapshotsWatchResponse` | Client | moved |
| `Plugins.GetInstallation` | `harmos.v1.Plugins/Get` | `harmos.v1.PluginsGetRequest` | `harmos.v1.PluginsGetResponse` | Client | awaits composition |
| `Plugins.Install` | `harmos.context.v1.Storage/CreateFileProduction` | `harmos.context.v1.StorageCreateFileProductionRequest` | `harmos.context.v1.StorageCreateFileProductionResponse` | Client | awaits composition |
| `Plugins.Install` | `harmos.context.v1.Storage/GetFileProduction` | `harmos.context.v1.StorageGetFileProductionRequest` | `harmos.context.v1.StorageGetFileProductionResponse` | Client | awaits composition |
| `Plugins.Install` | `harmos.context.v1.Storage/CreateFileWriter` | `harmos.context.v1.StorageCreateFileWriterRequest` | `harmos.context.v1.StorageCreateFileWriterResponse` | Client | awaits composition |
| `Plugins.Install` | `harmos.context.v1.Storage/AppendFileBytes` | `harmos.context.v1.StorageAppendFileBytesRequest` | `harmos.context.v1.StorageAppendFileBytesResponse` | Client | awaits composition |
| `Plugins.Install` | `harmos.context.v1.Storage/DeleteFileProduction` | `harmos.context.v1.StorageDeleteFileProductionRequest` | `harmos.context.v1.StorageDeleteFileProductionResponse` | Client | awaits composition |
| `Plugins.Install` | `harmos.context.v1.Storage/PublishFileProduction` | `harmos.context.v1.StoragePublishFileProductionRequest` | `harmos.context.v1.StoragePublishFileProductionResponse` | Client | awaits composition |
| `Plugins.Install` | `harmos.v1.Plugins/Create` | `harmos.v1.PluginsCreateRequest` | `harmos.v1.PluginsCreateResponse` | Client | awaits composition |
| `Plugins.ListAdmittedPlugins` | `harmos.v1.Plugins/List` | `harmos.v1.PluginsListRequest` | `harmos.v1.PluginsListResponse` | Client | awaits composition |
| `Plugins.ListInstallations` | `harmos.v1.Plugins/List` | `harmos.v1.PluginsListRequest` | `harmos.v1.PluginsListResponse` | Client | awaits composition |
| `Plugins.Reload` | `harmos.v1.Plugins/Restart` | `harmos.v1.PluginsRestartRequest` | `harmos.v1.PluginsRestartResponse` | Client | awaits composition |
| `Plugins.Remove` | `harmos.v1.Plugins/Delete` | `harmos.v1.PluginsDeleteRequest` | `harmos.v1.PluginsDeleteResponse` | Client | awaits composition |
| `Plugins.SetEnabled` | `harmos.v1.Plugins/Set` | `harmos.v1.PluginsSetRequest` | `harmos.v1.PluginsSetResponse` | Client | awaits composition |
| `Plugins.List` | `harmos.v1.Plugins/List` | `harmos.v1.PluginsListRequest` | `harmos.v1.PluginsListResponse` | Client | awaits composition |
| `Plugins.Add` | `harmos.v1.Plugins/Create` | `harmos.v1.PluginsCreateRequest` | `harmos.v1.PluginsCreateResponse` | Client | awaits composition |
| `Prompts.GetDefinition` | `dokime.v1.catalog.PromptDefinitions/Get` | `dokime.v1.catalog.PromptDefinitionsGetRequest` | `dokime.v1.catalog.PromptDefinitionsGetResponse` | Client | moved |
| `Prompts.GetPrompt` | `dokime.v1.Prompts/Get` | `dokime.v1.PromptsGetRequest` | `dokime.v1.PromptsGetResponse` | Client | moved |
| `Prompts.ListDefinitions` | `dokime.v1.catalog.PromptDefinitions/List` | `dokime.v1.catalog.PromptDefinitionsListRequest` | `dokime.v1.catalog.PromptDefinitionsListResponse` | Client | moved |
| `Prompts.ListPending` | `dokime.v1.Prompts/List` | `dokime.v1.PromptsListRequest` | `dokime.v1.PromptsListResponse` | Client | moved |
| `Prompts.Respond` | `dokime.v1.prompts.Answers/Create` | `dokime.v1.prompts.AnswersCreateRequest` | `dokime.v1.prompts.AnswersCreateResponse` | Client | moved |
| `Query.Aggregate` | `dokime.v1.Aggregates/Get` | `dokime.v1.AggregatesGetRequest` | `dokime.v1.AggregatesGetResponse` | Client | moved |
| `Query.AggregateSamples` | `dokime.v1.Evidence/Statistics` | `dokime.v1.EvidenceStatisticsRequest` | `dokime.v1.EvidenceStatisticsResponse` | Client | moved |
| `Query.ListCycles` | `dokime.v1.Cycles/List` | `dokime.v1.CyclesListRequest` | `dokime.v1.CyclesListResponse` | Client | moved |
| `Query.ListExecutions` | `dokime.v1.Executions/List` | `dokime.v1.ExecutionsListRequest` | `dokime.v1.ExecutionsListResponse` | Client | moved |
| `Query.ListRecords` | `dokime.v1.evidence.Records/List` | `dokime.v1.evidence.RecordsListRequest` | `dokime.v1.evidence.RecordsListResponse` | Client | moved |
| `Query.ListSamples` | `dokime.v1.evidence.Samples/List` | `dokime.v1.evidence.SamplesListRequest` | `dokime.v1.evidence.SamplesListResponse` | Client | moved |
| `Remote.CreatePairing` | `harmos.v1.access.Pairings/Create` | `harmos.v1.access.PairingsCreateRequest` | `harmos.v1.access.PairingsCreateResponse` | Client | moved |
| `Remote.ReadListener` | `harmos.v1.access.Listener/Get` | `harmos.v1.access.ListenerGetRequest` | `harmos.v1.access.ListenerGetResponse` | Client | moved |
| `Remote.RegenerateCertificate` | `harmos.v1.access.Certificates/Create` | `harmos.v1.access.CertificatesCreateRequest` | `harmos.v1.access.CertificatesCreateResponse` | Client | moved |
| `Remote.UpdateListener` | `harmos.v1.access.Listener/Set` | `harmos.v1.access.ListenerSetRequest` | `harmos.v1.access.ListenerSetResponse` | Client | moved |
| `Reports.PublishExecutionReport` | `dokime.v1.Reports/Publish` | `dokime.v1.ReportsPublishRequest` | `dokime.v1.ReportsPublishResponse` | Client | moved |
| `Reports.DownloadCycleReport` | `dokime.v1.Reports/Get` | `dokime.v1.ReportsGetRequest` | `dokime.v1.ReportsGetResponse` | Client | moved |
| `Reports.DownloadCycleReport` | `harmos.context.v1.Storage/GetFileBytes` | `harmos.context.v1.StorageGetFileBytesRequest` | `harmos.context.v1.StorageGetFileBytesResponse` | Client | awaits composition |
| `Reports.DownloadCycleReport` | `dokime.v1.Reports/Create` | `dokime.v1.ReportsCreateRequest` | `dokime.v1.ReportsCreateResponse` | Client | moved |
| `Reports.DownloadExecutionReport` | `dokime.v1.Reports/Get` | `dokime.v1.ReportsGetRequest` | `dokime.v1.ReportsGetResponse` | Client | moved |
| `Reports.DownloadExecutionReport` | `harmos.context.v1.Storage/GetFileBytes` | `harmos.context.v1.StorageGetFileBytesRequest` | `harmos.context.v1.StorageGetFileBytesResponse` | Client | awaits composition |
| `Reports.DownloadExecutionReport` | `dokime.v1.Reports/Create` | `dokime.v1.ReportsCreateRequest` | `dokime.v1.ReportsCreateResponse` | Client | moved |
| `Reports.MaterializeReport` | `dokime.v1.Reports/Create` | `dokime.v1.ReportsCreateRequest` | `dokime.v1.ReportsCreateResponse` | Client | moved |
| `Runtime.GetHealth` | `harmos.v1.Status/Get` | `harmos.v1.StatusGetRequest` | `harmos.v1.StatusGetResponse` | Client | moved |
| `Runtime.GetIdentity` | `harmos.v1.Status/Get` | `harmos.v1.StatusGetRequest` | `harmos.v1.StatusGetResponse` | Client | moved |
| `Secrets.Export` | `dokime.v1.Secrets/Export` | `dokime.v1.SecretsExportRequest` | `dokime.v1.SecretsExportResponse` | Client | moved |
| `Secrets.Import` | `dokime.v1.Secrets/Import` | `dokime.v1.SecretsImportRequest` | `dokime.v1.SecretsImportResponse` | Client | moved |
| `Secrets.List` | `dokime.v1.Secrets/List` | `dokime.v1.SecretsListRequest` | `dokime.v1.SecretsListResponse` | Client | moved |
| `Secrets.Lookup` | `dokime.v1.Secrets/Get` | `dokime.v1.SecretsGetRequest` | `dokime.v1.SecretsGetResponse` | Client | moved |
| `Secrets.Preview` | `dokime.v1.Secrets/Import` | `dokime.v1.SecretsImportRequest` | `dokime.v1.SecretsImportResponse` | Client | moved |
| `Secrets.Put` | `dokime.v1.Secrets/Set` | `dokime.v1.SecretsSetRequest` | `dokime.v1.SecretsSetResponse` | Client | moved |
| `Secrets.Remove` | `dokime.v1.Secrets/Delete` | `dokime.v1.SecretsDeleteRequest` | `dokime.v1.SecretsDeleteResponse` | Client | moved |
| `Settings.GetPluginSettings` | `dokime.v1.plugins.Settings/Get` | `dokime.v1.plugins.SettingsGetRequest` | `dokime.v1.plugins.SettingsGetResponse` | Client | moved |
| `Settings.GetSettings` | `dokime.v1.Settings/Schema` | `dokime.v1.SettingsSchemaRequest` | `dokime.v1.SettingsSchemaResponse` | Client | moved |
| `Settings.ListStorageSources` | `dokime.v1.storage.Sources/Get` | `dokime.v1.storage.SourcesGetRequest` | `dokime.v1.storage.SourcesGetResponse` | Client | moved |
| `Settings.ReadApplicationSettings` | `dokime.v1.Settings/Get` | `dokime.v1.SettingsGetRequest` | `dokime.v1.SettingsGetResponse` | Client | moved |
| `Settings.ReorderStorageSources` | `dokime.v1.storage.Sources/Set` | `dokime.v1.storage.SourcesSetRequest` | `dokime.v1.storage.SourcesSetResponse` | Client | moved |
| `Settings.SetStorageSourceEnabled` | `dokime.v1.storage.Sources/Set` | `dokime.v1.storage.SourcesSetRequest` | `dokime.v1.storage.SourcesSetResponse` | Client | moved |
| `Settings.UpdateApplicationSettings` | `dokime.v1.Settings/Set` | `dokime.v1.SettingsSetRequest` | `dokime.v1.SettingsSetResponse` | Client | moved |
| `Settings.UpdatePluginSettings` | `dokime.v1.plugins.Settings/Set` | `dokime.v1.plugins.SettingsSetRequest` | `dokime.v1.plugins.SettingsSetResponse` | Client | moved |
| `Telemetry.ListSamples` | `dokime.v1.evidence.Samples/List` | `dokime.v1.evidence.SamplesListRequest` | `dokime.v1.evidence.SamplesListResponse` | Client | moved |
| `Telemetry.WatchSamples` | `dokime.v1.evidence.Samples/Watch` | `dokime.v1.evidence.SamplesWatchRequest` | `dokime.v1.evidence.SamplesWatchResponse` | Client | moved |
| Companion `Cases.Watch` | `dokime.v1.Cases/Watch` | `dokime.v1.CasesWatchRequest` | `dokime.v1.CasesWatchResponse` | Client | moved |
| Companion `Content.Artifacts.Watch` | `dokime.v1.content.Artifacts/Watch` | `dokime.v1.content.ArtifactsWatchRequest` | `dokime.v1.content.ArtifactsWatchResponse` | Client | moved |
| Companion `Content.ExecutionFiles.Watch` | `dokime.v1.content.ExecutionFiles/Watch` | `dokime.v1.content.ExecutionFilesWatchRequest` | `dokime.v1.content.ExecutionFilesWatchResponse` | Client | moved |
| Companion `Cycles.Watch` | `dokime.v1.Cycles/Watch` | `dokime.v1.CyclesWatchRequest` | `dokime.v1.CyclesWatchResponse` | Client | moved |
| Companion `Documents.Watch` | `dokime.v1.Documents/Watch` | `dokime.v1.DocumentsWatchRequest` | `dokime.v1.DocumentsWatchResponse` | Client | moved |
| Companion `Evidence.HealthEvents.Watch` | `dokime.v1.evidence.HealthEvents/Watch` | `dokime.v1.evidence.HealthEventsWatchRequest` | `dokime.v1.evidence.HealthEventsWatchResponse` | Client | moved |
| Companion `Evidence.Notes.Get` | `dokime.v1.evidence.Notes/Get` | `dokime.v1.evidence.NotesGetRequest` | `dokime.v1.evidence.NotesGetResponse` | Client | moved |
| Companion `Evidence.Notes.Watch` | `dokime.v1.evidence.Notes/Watch` | `dokime.v1.evidence.NotesWatchRequest` | `dokime.v1.evidence.NotesWatchResponse` | Client | moved |
| Companion `Evidence.Traceability.Watch` | `dokime.v1.evidence.Traceability/Watch` | `dokime.v1.evidence.TraceabilityWatchRequest` | `dokime.v1.evidence.TraceabilityWatchResponse` | Client | done |
| Companion `Executions.Progress.Get` | `dokime.v1.executions.Progress/Get` | `dokime.v1.executions.ProgressGetRequest` | `dokime.v1.executions.ProgressGetResponse` | Client | done |
| Companion `Executions.Watch` | `dokime.v1.Executions/Watch` | `dokime.v1.ExecutionsWatchRequest` | `dokime.v1.ExecutionsWatchResponse` | Client | moved |
| Companion `Media.Captures.Get` | `dokime.v1.media.Captures/Get` | `dokime.v1.media.CapturesGetRequest` | `dokime.v1.media.CapturesGetResponse` | Client | moved |
| Companion `Plots.Settings.Get` | `dokime.v1.plots.Settings/Get` | `dokime.v1.plots.SettingsGetRequest` | `dokime.v1.plots.SettingsGetResponse` | Client | moved |
| Companion `Prompts.Watch` | `dokime.v1.Prompts/Watch` | `dokime.v1.PromptsWatchRequest` | `dokime.v1.PromptsWatchResponse` | Client | moved |
| Companion `Secrets.Watch` | `dokime.v1.Secrets/Watch` | `dokime.v1.SecretsWatchRequest` | `dokime.v1.SecretsWatchResponse` | Client | moved |
| `<template>.materialize` | `<template>.materialize` | `dokime.plugin.v1.TestCasesMaterializeRequest` | `dokime.plugin.v1.TestCasesMaterializeResponse` | Plugin (Protected) | moved |
| `<template>.initialize` | `<template>.initialize` | `dokime.plugin.v1.ExecutionsInitializeRequest` | `dokime.plugin.v1.ExecutionsInitializeResponse` | Plugin (Protected) | moved |
| `Step.operation` | `plugin-authored dotted step identity` | `dokime.plugin.v1.StepsExecuteRequest` | `dokime.plugin.v1.StepsExecuteResponse` | Plugin (Protected) | moved |
| `Criterion.operation` | `plugin-authored dotted criterion identity` | `dokime.plugin.v1.CriteriaEvaluateRequest` | `dokime.plugin.v1.CriteriaEvaluateResponse` | Plugin (Protected) | moved |
| `<template>.cleanup` | `<template>.cleanup` | `dokime.plugin.v1.ExecutionsCleanupRequest` | `dokime.plugin.v1.ExecutionsCleanupResponse` | Plugin (Protected) | moved |
| `dokime.execution.prompt` | `dokime.context.v1.Prompts/Create` | `dokime.context.v1.PromptsCreateRequest` | `dokime.context.v1.PromptsCreateResponse` | Context (Public) | moved |
| `dokime.execution.record` | `dokime.context.v1.evidence.Records/Append` | `dokime.context.v1.evidence.RecordsAppendRequest` | `dokime.context.v1.evidence.RecordsAppendResponse` | Context (Public) | moved |
| `dokime.execution.records` | `dokime.context.v1.evidence.Records/List` | `dokime.context.v1.evidence.RecordsListRequest` | `dokime.context.v1.evidence.RecordsListResponse` | Context (Public) | moved |
| `dokime.execution.statistics` | `dokime.context.v1.Evidence/Statistics` | `dokime.context.v1.EvidenceStatisticsRequest` | `dokime.context.v1.EvidenceStatisticsResponse` | Context (Public) | moved |
| `dokime.execution.artifact` | `dokime.context.v1.content.Artifacts/Create` | `dokime.context.v1.content.ArtifactsCreateRequest` | `dokime.context.v1.content.ArtifactsCreateResponse` | Context (Public) | moved |
| `Progress.Get` | `dokime.context.v1.executions.Progress/Get` | `dokime.context.v1.executions.ProgressGetRequest` | `dokime.context.v1.executions.ProgressGetResponse` | Context (Public) | done |
| `dokime.execution.progress` | `dokime.context.v1.executions.Progress/Set` | `dokime.context.v1.executions.ProgressSetRequest` | `dokime.context.v1.executions.ProgressSetResponse` | Context (Public) | done |
| `dokime.execution.log` | `dokime.context.v1.evidence.Logs/Append` | `dokime.context.v1.evidence.LogsAppendRequest` | `dokime.context.v1.evidence.LogsAppendResponse` | Context (Public) | moved |
| `Traceability.Get` | `dokime.context.v1.evidence.Traceability/Get` | `dokime.context.v1.evidence.TraceabilityGetRequest` | `dokime.context.v1.evidence.TraceabilityGetResponse` | Context (Public) | done |
| `dokime.execution.traceability` | `dokime.context.v1.evidence.Traceability/Set` | `dokime.context.v1.evidence.TraceabilitySetRequest` | `dokime.context.v1.evidence.TraceabilitySetResponse` | Context (Public) | done |
| `dokime.execution.document` | `dokime.context.v1.Documents/AppendParts` | `dokime.context.v1.DocumentsAppendPartsRequest` | `dokime.context.v1.DocumentsAppendPartsResponse` | Context (Public) | moved |
| `dokime.files.copy_files` | `dokime.context.v1.Files/Capture` | `dokime.context.v1.FilesCaptureRequest` | `dokime.context.v1.FilesCaptureResponse` | Context (Public) | moved |
| `dokime.files.start_capture` | `dokime.context.v1.files.Captures/Start` | `dokime.context.v1.files.CapturesStartRequest` | `dokime.context.v1.files.CapturesStartResponse` | Context (Public) | moved |
| `dokime.files.stop_capture` | `dokime.context.v1.files.Captures/Stop` | `dokime.context.v1.files.CapturesStopRequest` | `dokime.context.v1.files.CapturesStopResponse` | Context (Public) | moved |
| `dokime.execution.environment` | `dokime.context.v1.Backend/Environment` | `dokime.context.v1.BackendEnvironmentRequest` | `dokime.context.v1.BackendEnvironmentResponse` | Context (Public) | moved |
| `dokime.execution.effect.prepare` | `dokime.context.v1.Effects/Create` | `dokime.context.v1.EffectsCreateRequest` | `dokime.context.v1.EffectsCreateResponse` | Context (Public) | moved |
| `Effects.Get` | `dokime.context.v1.Effects/Get` | `dokime.context.v1.EffectsGetRequest` | `dokime.context.v1.EffectsGetResponse` | Context (Public) | moved |
| `dokime.execution.effect.applied` | `dokime.context.v1.Effects/Set` | `dokime.context.v1.EffectsSetRequest` | `dokime.context.v1.EffectsSetResponse` | Context (Public) | moved |
| `dokime.settings.read` | `harmos.context.v1.HostValues/Get` | `harmos.context.v1.HostValuesGetRequest` | `harmos.context.v1.HostValuesGetResponse` | Context | awaits composition |
| `dokime.secrets.read` | `harmos.context.v1.HostValues/Get` | `harmos.context.v1.HostValuesGetRequest` | `harmos.context.v1.HostValuesGetResponse` | Context | awaits composition |
| `harmos.clock.now` | `harmos.context.v1.Time/Get` | `harmos.context.v1.TimeGetRequest` | `harmos.context.v1.TimeGetResponse` | Context | moved |
| `harmos.clock.sleep` | `harmos.context.v1.Time/Sleep` | `harmos.context.v1.TimeSleepRequest` | `harmos.context.v1.TimeSleepResponse` | Context | moved |
| `dokime.execution.artifact` transfer | `harmos.context.v1.Storage/CreateFileProduction` | `harmos.context.v1.StorageCreateFileProductionRequest` | `harmos.context.v1.StorageCreateFileProductionResponse` | Context | awaits composition |
| `dokime.execution.artifact` transfer | `harmos.context.v1.Storage/GetFileProduction` | `harmos.context.v1.StorageGetFileProductionRequest` | `harmos.context.v1.StorageGetFileProductionResponse` | Context | awaits composition |
| `dokime.execution.artifact` transfer | `harmos.context.v1.Storage/CreateFileWriter` | `harmos.context.v1.StorageCreateFileWriterRequest` | `harmos.context.v1.StorageCreateFileWriterResponse` | Context | awaits composition |
| `dokime.execution.artifact` transfer | `harmos.context.v1.Storage/AppendFileBytes` | `harmos.context.v1.StorageAppendFileBytesRequest` | `harmos.context.v1.StorageAppendFileBytesResponse` | Context | awaits composition |
| `dokime.execution.artifact` transfer | `harmos.context.v1.Storage/DeleteFileProduction` | `harmos.context.v1.StorageDeleteFileProductionRequest` | `harmos.context.v1.StorageDeleteFileProductionResponse` | Context | awaits composition |
| `dokime.execution.artifact` transfer | `harmos.context.v1.Storage/PublishFileProduction` | `harmos.context.v1.StoragePublishFileProductionRequest` | `harmos.context.v1.StoragePublishFileProductionResponse` | Context | awaits composition |
| `dokime.execution.artifact` transfer | `harmos.context.v1.Storage/GetFileBytes` | `harmos.context.v1.StorageGetFileBytesRequest` | `harmos.context.v1.StorageGetFileBytesResponse` | Context | awaits composition |
| `dokime.execution.artifact` transfer | `harmos.context.v1.Storage/WatchFileBytes` | `harmos.context.v1.StorageWatchFileBytesRequest` | `harmos.context.v1.StorageWatchFileBytesResponse` | Context | awaits composition |

Lifecycle rows name each actual dotted catalog binding, not the obsolete shared
`dokime.plugin.v1.<Family>/<Verb>` dispatch identity. Every binding is Protected,
keeps its own operation policy, and uses only the indicated shared messages over
`harmos.plugin.v1.Instance/Invoke`. Product plugin and context files declare no
RPC services. Private operations are local and absent from these faces; Public
client and context operations still require their audience and scoped admission.

Where the design leaves field layout open, new wrappers use the completed
Progress/Traceability pattern: message-presence `at`, explicit `resolved_at`,
Harmos Position for one history and ReadCut for compound selections, Harmos
PageRequest/Receipt/Failure and Storage FileReference, byte revisions, and caller-retained request_id.
Watches return opening snapshots followed by durable changes and an explicit
resync_required failure on gaps. Unavailable returned cuts fail rather than fall forward; retention and restart
proof for new resources remains with their host implementation.
Pending prompts return whole without page controls; retained history is paged.
Documents.Get carries a bounded ordered frame sequence with continuation at one
fixed cut; its message comments define structure, counters, EvidenceCut and results.
Catalog analyses and native analysis payloads remain immutable whole declarations.
No consumer handler behavior is claimed by this definition pass.

The following consumer lists name the changed generated modules. `dokime.v1`
contains the root resource services; its changed nested modules are `backend`,
`catalog`, `content`, `evidence`, `executions`, `media`, `plots`, `plugins`,
`prompts` and `storage`. The plugin module is `dokime.plugin.v1`; context modules
are `dokime.context.v1`, `.content` and `.evidence` (existing `.executions` and
`.files` remain separate). Remove imports of superseded mixed request/response
and checkpoint DTOs while adopting the wrappers in the table.

- Host Handlers and Operations: `dokime_protocol::v1` and all nested client modules above; `dokime_protocol::plugin`; `dokime_protocol::context` and its `content`/`evidence` modules. Rebind services and context handlers; preserve lifecycle binding IDs/policies; implement the ordered Documents.Get contract and library compositions.

- Rust SDK: the same protocol face modules; regenerated `sdk/rust/src/client/resources.rs`. Migrate authoring, context helpers, convenience clients, descriptors and fixtures to the new typed wrappers.

- Python SDK: `dokime.v1` root/nested protobuf modules, `dokime.plugin.v1`, `dokime.context.v1` and its `content`/`evidence` modules; regenerated `sdk/python/src/dokime/resources.json`. Regenerate protobuf modules and migrate lifecycle and child-call helpers.

- C# SDK: `Dokime.Protocol.V1` and the nested namespaces above, `Dokime.Protocol.Plugin.V1`, `Dokime.Protocol.Context.V1`/`.Content`/`.Evidence`; regenerated `sdk/csharp/Dokime/Resources.cs` and `ClientResources.cs`. Migrate frozen descriptors, context helpers and typed calls.

- C++ SDK: `dokime::v1` and nested client namespaces above, `dokime::plugin::v1`, `dokime::context::v1`/`content`/`evidence`; regenerated `sdk/cpp/include/dokime/resources.hpp`. Regenerate messages and migrate lifecycle/context/publication calls.

- Command-Line Tool: `dokime_protocol::v1` and its nested client modules above, plus regenerated Rust resource roots. Migrate raw identities, discovery/examples, lifecycle mutations, metadata/content workflows and retained-cut continuation.

- Desktop Adapter: generated `dokime/v1/{backend,catalog,content,evidence,executions,media,plots,plugins,prompts,storage}` modules and root resources in the host/frontend generation. Migrate adapter request/response decoding, revisioned settings, library transfers and ordered document frame validation.
