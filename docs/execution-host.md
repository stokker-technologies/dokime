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

This work order starts from the definitions at `db4269b54`, immediately before
the boundary amendment of 2026-11-13. Present identities below are the actual
pre-amendment wire names, not the names from earlier grammar proposals. Wire
identities use `package.Service/Method`; plugin lifecycle bindings remain dotted.
`moved` records an amended definition, `done` an unchanged binding or composed
library definition, and `awaits composition` a retained ordinary declaration or
library destination. These states do not claim that consumers compile.

`dokime.v1` is shared vocabulary only. Clients call `dokime.client.v1`; plugins
implement the independent Protected bindings using `dokime.plugin.v1` messages;
plugins call the host through `dokime.plugin.host.v1` messages. The SDK object
is still named `context`. Product plugin and plugin-host files declare no services:
they use the pinned library Instance.Invoke carrier. Private bindings stay local.

The retained Plugins and Content services live in `client/v1/plugins.proto` and
`client/v1/content.proto`. They keep their existing methods and streaming shapes
until direct library composition and are excluded from generated client roots.
Pending settings, secret and artifact-transfer messages live with their plugin-host
resources. The library vendoring, packages and generated references remain pinned.

| Present Identity | Final Wire Identity or Lifecycle Binding | Request | Response | Boundary | State |
| --- | --- | --- | --- | --- | --- |
| `dokime.v1.Aggregates/Get` | `dokime.client.v1.Aggregates/Get` | `dokime.client.v1.AggregatesGetRequest` | `dokime.client.v1.AggregatesGetResponse` | Client | moved |
| `dokime.v1.Backend/Appearance` | `dokime.client.v1.Backend/Appearance` | `dokime.client.v1.BackendAppearanceRequest` | `dokime.client.v1.BackendAppearanceResponse` | Client | moved |
| `dokime.v1.backend.Overview/Get` | `dokime.client.v1.backend.Overview/Get` | `dokime.client.v1.backend.OverviewGetRequest` | `dokime.client.v1.backend.OverviewGetResponse` | Client | moved |
| `dokime.v1.backend.Overview/Watch` | `dokime.client.v1.backend.Overview/Watch` | `dokime.client.v1.backend.OverviewWatchRequest` | `dokime.client.v1.backend.OverviewWatchResponse` | Client | moved |
| `dokime.v1.Cases/Create` | `dokime.client.v1.Cases/Create` | `dokime.client.v1.CasesCreateRequest` | `dokime.client.v1.CasesCreateResponse` | Client | moved |
| `dokime.v1.Cases/Get` | `dokime.client.v1.Cases/Get` | `dokime.client.v1.CasesGetRequest` | `dokime.client.v1.CasesGetResponse` | Client | moved |
| `dokime.v1.Cases/List` | `dokime.client.v1.Cases/List` | `dokime.client.v1.CasesListRequest` | `dokime.client.v1.CasesListResponse` | Client | moved |
| `dokime.v1.Cases/Watch` | `dokime.client.v1.Cases/Watch` | `dokime.client.v1.CasesWatchRequest` | `dokime.client.v1.CasesWatchResponse` | Client | moved |
| `dokime.v1.Cases/Delete` | `dokime.client.v1.Cases/Delete` | `dokime.client.v1.CasesDeleteRequest` | `dokime.client.v1.CasesDeleteResponse` | Client | moved |
| `dokime.v1.Cases/Parameters` | `dokime.client.v1.Cases/Parameters` | `dokime.client.v1.CasesParametersRequest` | `dokime.client.v1.CasesParametersResponse` | Client | moved |
| `dokime.v1.Catalog/Analyses` | `dokime.client.v1.Analyses/List` | `dokime.client.v1.AnalysesListRequest` | `dokime.client.v1.AnalysesListResponse` | Client | moved |
| `dokime.v1.catalog.PromptDefinitions/Get` | `dokime.client.v1.PromptDefinitions/Get` | `dokime.client.v1.PromptDefinitionsGetRequest` | `dokime.client.v1.PromptDefinitionsGetResponse` | Client | moved |
| `dokime.v1.catalog.PromptDefinitions/List` | `dokime.client.v1.PromptDefinitions/List` | `dokime.client.v1.PromptDefinitionsListRequest` | `dokime.client.v1.PromptDefinitionsListResponse` | Client | moved |
| `dokime.v1.catalog.RecordSchemas/Get` | `dokime.client.v1.RecordSchemas/Get` | `dokime.client.v1.RecordSchemasGetRequest` | `dokime.client.v1.RecordSchemasGetResponse` | Client | moved |
| `dokime.v1.catalog.RecordSchemas/List` | `dokime.client.v1.RecordSchemas/List` | `dokime.client.v1.RecordSchemasListRequest` | `dokime.client.v1.RecordSchemasListResponse` | Client | moved |
| `dokime.v1.catalog.StreamSchemas/Get` | `dokime.client.v1.StreamSchemas/Get` | `dokime.client.v1.StreamSchemasGetRequest` | `dokime.client.v1.StreamSchemasGetResponse` | Client | moved |
| `dokime.v1.catalog.StreamSchemas/List` | `dokime.client.v1.StreamSchemas/List` | `dokime.client.v1.StreamSchemasListRequest` | `dokime.client.v1.StreamSchemasListResponse` | Client | moved |
| `dokime.v1.catalog.Suites/Get` | `dokime.client.v1.Suites/Get` | `dokime.client.v1.SuitesGetRequest` | `dokime.client.v1.SuitesGetResponse` | Client | moved |
| `dokime.v1.catalog.Suites/List` | `dokime.client.v1.Suites/List` | `dokime.client.v1.SuitesListRequest` | `dokime.client.v1.SuitesListResponse` | Client | moved |
| `dokime.v1.Content/DownloadAppearanceAsset` | `dokime.client.v1.Content/DownloadAppearanceAsset` | `dokime.client.v1.ContentDownloadAppearanceAssetRequest` | `dokime.client.v1.ContentDownloadAppearanceAssetOutput` | Client | awaits composition |
| `dokime.v1.Plugins/GetInstallation` | `dokime.client.v1.Plugins/GetInstallation` | `dokime.client.v1.PluginsGetInstallationRequest` | `dokime.client.v1.PluginInstallation` | Client | awaits composition |
| `dokime.v1.Plugins/Install` | `dokime.client.v1.Plugins/Install` | `dokime.client.v1.PluginsInstallInput` | `dokime.client.v1.PluginInstallation` | Client | awaits composition |
| `dokime.v1.Plugins/ListAdmittedPlugins` | `dokime.client.v1.Plugins/ListAdmittedPlugins` | `dokime.client.v1.PluginsListAdmittedPluginsRequest` | `dokime.client.v1.AdmittedPluginPage` | Client | awaits composition |
| `dokime.v1.Plugins/ListInstallations` | `dokime.client.v1.Plugins/ListInstallations` | `dokime.client.v1.PluginsListInstallationsRequest` | `dokime.client.v1.InstallationPage` | Client | awaits composition |
| `dokime.v1.Plugins/Reload` | `dokime.client.v1.Plugins/Reload` | `dokime.client.v1.PluginsReloadRequest` | `dokime.v1.ClientEmpty` | Client | awaits composition |
| `dokime.v1.Plugins/Remove` | `dokime.client.v1.Plugins/Remove` | `dokime.client.v1.RemovePlugin` | `dokime.client.v1.RemovePluginResponse` | Client | awaits composition |
| `dokime.v1.Plugins/SetEnabled` | `dokime.client.v1.Plugins/SetEnabled` | `dokime.client.v1.PluginsSetEnabledRequest` | `dokime.client.v1.PluginInstallation` | Client | awaits composition |
| `dokime.v1.Plugins/List` | `dokime.client.v1.Plugins/List` | `dokime.client.v1.ListPlugins` | `dokime.client.v1.PluginEntries` | Client | awaits composition |
| `dokime.v1.Plugins/Add` | `dokime.client.v1.Plugins/Add` | `dokime.client.v1.AddRemotePlugin` | `dokime.client.v1.Plugin` | Client | awaits composition |
| `dokime.v1.content.Artifacts/Get` | `dokime.client.v1.content.Artifacts/Get` | `dokime.client.v1.content.ArtifactsGetRequest` | `dokime.client.v1.content.ArtifactsGetResponse` | Client | moved |
| `dokime.v1.content.Artifacts/Create` | `dokime.client.v1.content.Artifacts/Create` | `dokime.client.v1.content.ArtifactsCreateRequest` | `dokime.client.v1.content.ArtifactsCreateResponse` | Client | moved |
| `dokime.v1.content.Artifacts/List` | `dokime.client.v1.content.Artifacts/List` | `dokime.client.v1.content.ArtifactsListRequest` | `dokime.client.v1.content.ArtifactsListResponse` | Client | moved |
| `dokime.v1.content.Artifacts/Watch` | `dokime.client.v1.content.Artifacts/Watch` | `dokime.client.v1.content.ArtifactsWatchRequest` | `dokime.client.v1.content.ArtifactsWatchResponse` | Client | moved |
| `dokime.v1.content.ExecutionFiles/Get` | `dokime.client.v1.content.ExecutionFiles/Get` | `dokime.client.v1.content.ExecutionFilesGetRequest` | `dokime.client.v1.content.ExecutionFilesGetResponse` | Client | moved |
| `dokime.v1.content.ExecutionFiles/List` | `dokime.client.v1.content.ExecutionFiles/List` | `dokime.client.v1.content.ExecutionFilesListRequest` | `dokime.client.v1.content.ExecutionFilesListResponse` | Client | moved |
| `dokime.v1.content.ExecutionFiles/Watch` | `dokime.client.v1.content.ExecutionFiles/Watch` | `dokime.client.v1.content.ExecutionFilesWatchRequest` | `dokime.client.v1.content.ExecutionFilesWatchResponse` | Client | moved |
| `dokime.v1.Cycles/Get` | `dokime.client.v1.Cycles/Get` | `dokime.client.v1.CyclesGetRequest` | `dokime.client.v1.CyclesGetResponse` | Client | moved |
| `dokime.v1.Cycles/Set` | `dokime.client.v1.Cycles/Set` | `dokime.client.v1.CyclesSetRequest` | `dokime.client.v1.CyclesSetResponse` | Client | moved |
| `dokime.v1.Cycles/List` | `dokime.client.v1.Cycles/List` | `dokime.client.v1.CyclesListRequest` | `dokime.client.v1.CyclesListResponse` | Client | moved |
| `dokime.v1.Cycles/Watch` | `dokime.client.v1.Cycles/Watch` | `dokime.client.v1.CyclesWatchRequest` | `dokime.client.v1.CyclesWatchResponse` | Client | moved |
| `dokime.v1.Cycles/Create` | `dokime.client.v1.Cycles/Create` | `dokime.client.v1.CyclesCreateRequest` | `dokime.client.v1.CyclesCreateResponse` | Client | moved |
| `dokime.v1.Cycles/Seal` | `dokime.client.v1.Cycles/Seal` | `dokime.client.v1.CyclesSealRequest` | `dokime.client.v1.CyclesSealResponse` | Client | moved |
| `dokime.v1.Documents/Get` | `dokime.client.v1.Documents/Get` | `dokime.client.v1.DocumentsGetRequest` | `dokime.client.v1.DocumentsGetResponse` | Client | moved |
| `dokime.v1.Documents/Watch` | `dokime.client.v1.Documents/Watch` | `dokime.client.v1.DocumentsWatchRequest` | `dokime.client.v1.DocumentsWatchResponse` | Client | moved |
| `dokime.v1.Evidence/Statistics` | `dokime.client.v1.Evidence/Statistics` | `dokime.client.v1.EvidenceStatisticsRequest` | `dokime.client.v1.EvidenceStatisticsResponse` | Client | moved |
| `dokime.v1.Evidence/Archive` | `dokime.client.v1.Evidence/Archive` | `dokime.client.v1.EvidenceArchiveRequest` | `dokime.client.v1.EvidenceArchiveResponse` | Client | moved |
| `dokime.v1.evidence.HealthEvents/List` | `dokime.client.v1.evidence.HealthEvents/List` | `dokime.client.v1.evidence.HealthEventsListRequest` | `dokime.client.v1.evidence.HealthEventsListResponse` | Client | moved |
| `dokime.v1.evidence.HealthEvents/Watch` | `dokime.client.v1.evidence.HealthEvents/Watch` | `dokime.client.v1.evidence.HealthEventsWatchRequest` | `dokime.client.v1.evidence.HealthEventsWatchResponse` | Client | moved |
| `dokime.v1.evidence.Notes/Get` | `dokime.client.v1.evidence.Notes/Get` | `dokime.client.v1.evidence.NotesGetRequest` | `dokime.client.v1.evidence.NotesGetResponse` | Client | moved |
| `dokime.v1.evidence.Notes/Set` | `dokime.client.v1.evidence.Notes/Set` | `dokime.client.v1.evidence.NotesSetRequest` | `dokime.client.v1.evidence.NotesSetResponse` | Client | moved |
| `dokime.v1.evidence.Notes/List` | `dokime.client.v1.evidence.Notes/List` | `dokime.client.v1.evidence.NotesListRequest` | `dokime.client.v1.evidence.NotesListResponse` | Client | moved |
| `dokime.v1.evidence.Notes/Watch` | `dokime.client.v1.evidence.Notes/Watch` | `dokime.client.v1.evidence.NotesWatchRequest` | `dokime.client.v1.evidence.NotesWatchResponse` | Client | moved |
| `dokime.v1.evidence.Notes/Create` | `dokime.client.v1.evidence.Notes/Create` | `dokime.client.v1.evidence.NotesCreateRequest` | `dokime.client.v1.evidence.NotesCreateResponse` | Client | moved |
| `dokime.v1.evidence.Records/List` | `dokime.client.v1.evidence.Records/List` | `dokime.client.v1.evidence.RecordsListRequest` | `dokime.client.v1.evidence.RecordsListResponse` | Client | moved |
| `dokime.v1.evidence.Records/Watch` | `dokime.client.v1.evidence.Records/Watch` | `dokime.client.v1.evidence.RecordsWatchRequest` | `dokime.client.v1.evidence.RecordsWatchResponse` | Client | moved |
| `dokime.v1.evidence.Samples/List` | `dokime.client.v1.evidence.Samples/List` | `dokime.client.v1.evidence.SamplesListRequest` | `dokime.client.v1.evidence.SamplesListResponse` | Client | moved |
| `dokime.v1.evidence.Samples/Watch` | `dokime.client.v1.evidence.Samples/Watch` | `dokime.client.v1.evidence.SamplesWatchRequest` | `dokime.client.v1.evidence.SamplesWatchResponse` | Client | moved |
| `dokime.v1.evidence.Traceability/Get` | `dokime.client.v1.evidence.Traceability/Get` | `dokime.client.v1.evidence.TraceabilityGetRequest` | `dokime.client.v1.evidence.TraceabilityGetResponse` | Client | moved |
| `dokime.v1.evidence.Traceability/Watch` | `dokime.client.v1.evidence.Traceability/Watch` | `dokime.client.v1.evidence.TraceabilityWatchRequest` | `dokime.client.v1.evidence.TraceabilityWatchResponse` | Client | moved |
| `dokime.v1.Executions/Get` | `dokime.client.v1.Executions/Get` | `dokime.client.v1.ExecutionsGetRequest` | `dokime.client.v1.ExecutionsGetResponse` | Client | moved |
| Embedded `Executions.FrozenSchemas.catalog` | `dokime.client.v1.executions.Schemas/Get` | `dokime.client.v1.executions.SchemasGetRequest` | `dokime.client.v1.executions.SchemasGetResponse` | Client | replaced by digest and paged descriptors |
| `dokime.v1.Executions/Set` | `dokime.client.v1.Executions/Set` | `dokime.client.v1.ExecutionsSetRequest` | `dokime.client.v1.ExecutionsSetResponse` | Client | moved |
| `dokime.v1.Executions/List` | `dokime.client.v1.Executions/List` | `dokime.client.v1.ExecutionsListRequest` | `dokime.client.v1.ExecutionsListResponse` | Client | moved |
| `dokime.v1.Executions/Watch` | `dokime.client.v1.Executions/Watch` | `dokime.client.v1.ExecutionsWatchRequest` | `dokime.client.v1.ExecutionsWatchResponse` | Client | moved |
| `dokime.v1.Executions/Create` | `dokime.client.v1.Executions/Create` | `dokime.client.v1.ExecutionsCreateRequest` | `dokime.client.v1.ExecutionsCreateResponse` | Client | moved |
| `dokime.v1.Executions/Start` | `dokime.client.v1.Executions/Start` | `dokime.client.v1.ExecutionsStartRequest` | `dokime.client.v1.ExecutionsStartResponse` | Client | moved |
| `dokime.v1.Executions/Stop` | `dokime.client.v1.Executions/Stop` | `dokime.client.v1.ExecutionsStopRequest` | `dokime.client.v1.ExecutionsStopResponse` | Client | moved |
| `dokime.v1.Executions/Cancel` | `dokime.client.v1.Executions/Cancel` | `dokime.client.v1.ExecutionsCancelRequest` | `dokime.client.v1.ExecutionsCancelResponse` | Client | moved |
| `dokime.v1.Executions/Delete` | `dokime.client.v1.Executions/Delete` | `dokime.client.v1.ExecutionsDeleteRequest` | `dokime.client.v1.ExecutionsDeleteResponse` | Client | moved |
| `dokime.v1.Executions/Parameters` | `dokime.client.v1.Executions/Parameters` | `dokime.client.v1.ExecutionsParametersRequest` | `dokime.client.v1.ExecutionsParametersResponse` | Client | moved |
| `dokime.v1.Executions/Steps` | `dokime.client.v1.Executions/Steps` | `dokime.client.v1.ExecutionsStepsRequest` | `dokime.client.v1.ExecutionsStepsResponse` | Client | moved |
| `dokime.v1.executions.Commands/Get` | `dokime.client.v1.executions.Commands/Get` | `dokime.client.v1.executions.CommandsGetRequest` | `dokime.client.v1.executions.CommandsGetResponse` | Client | moved |
| `dokime.v1.executions.Commands/Create` | `dokime.client.v1.executions.Commands/Create` | `dokime.client.v1.executions.CommandsCreateRequest` | `dokime.client.v1.executions.CommandsCreateResponse` | Client | moved |
| `dokime.v1.executions.Progress/Get` | `dokime.client.v1.executions.Progress/Get` | `dokime.client.v1.executions.ProgressGetRequest` | `dokime.client.v1.executions.ProgressGetResponse` | Client | moved |
| `dokime.v1.executions.Progress/Watch` | `dokime.client.v1.executions.Progress/Watch` | `dokime.client.v1.executions.ProgressWatchRequest` | `dokime.client.v1.executions.ProgressWatchResponse` | Client | moved |
| `dokime.v1.media.Captures/Create` | `dokime.client.v1.media.Captures/Create` | `dokime.client.v1.media.CapturesCreateRequest` | `dokime.client.v1.media.CapturesCreateResponse` | Client | moved |
| `dokime.v1.media.Captures/Get` | `dokime.client.v1.media.Captures/Get` | `dokime.client.v1.media.CapturesGetRequest` | `dokime.client.v1.media.CapturesGetResponse` | Client | moved |
| `dokime.v1.media.Captures/Watch` | `dokime.client.v1.media.Captures/Watch` | `dokime.client.v1.media.CapturesWatchRequest` | `dokime.client.v1.media.CapturesWatchResponse` | Client | moved |
| `dokime.v1.media.Captures/Stop` | `dokime.client.v1.media.Captures/Stop` | `dokime.client.v1.media.CapturesStopRequest` | `dokime.client.v1.media.CapturesStopResponse` | Client | moved |
| `dokime.v1.plots.Settings/Get` | `dokime.client.v1.plots.Settings/Get` | `dokime.client.v1.plots.SettingsGetRequest` | `dokime.client.v1.plots.SettingsGetResponse` | Client | moved |
| `dokime.v1.plots.Settings/Set` | `dokime.client.v1.plots.Settings/Set` | `dokime.client.v1.plots.SettingsSetRequest` | `dokime.client.v1.plots.SettingsSetResponse` | Client | moved |
| `dokime.v1.plots.Snapshots/List` | `dokime.client.v1.plots.Snapshots/List` | `dokime.client.v1.plots.SnapshotsListRequest` | `dokime.client.v1.plots.SnapshotsListResponse` | Client | moved |
| `dokime.v1.plots.Snapshots/Watch` | `dokime.client.v1.plots.Snapshots/Watch` | `dokime.client.v1.plots.SnapshotsWatchRequest` | `dokime.client.v1.plots.SnapshotsWatchResponse` | Client | moved |
| `dokime.v1.plugins.Settings/Get` | `dokime.client.v1.plugins.Settings/Get` | `dokime.client.v1.plugins.SettingsGetRequest` | `dokime.client.v1.plugins.SettingsGetResponse` | Client | moved |
| `dokime.v1.plugins.Settings/Set` | `dokime.client.v1.plugins.Settings/Set` | `dokime.client.v1.plugins.SettingsSetRequest` | `dokime.client.v1.plugins.SettingsSetResponse` | Client | moved |
| `dokime.v1.Prompts/Get` | `dokime.client.v1.Prompts/Get` | `dokime.client.v1.PromptsGetRequest` | `dokime.client.v1.PromptsGetResponse` | Client | moved |
| `dokime.v1.Prompts/List` | `dokime.client.v1.Prompts/List` | `dokime.client.v1.PromptsListRequest` | `dokime.client.v1.PromptsListResponse` | Client | moved |
| `dokime.v1.Prompts/Watch` | `dokime.client.v1.Prompts/Watch` | `dokime.client.v1.PromptsWatchRequest` | `dokime.client.v1.PromptsWatchResponse` | Client | moved |
| `dokime.v1.prompts.Answers/Create` | `dokime.client.v1.prompts.Answers/Create` | `dokime.client.v1.prompts.AnswersCreateRequest` | `dokime.client.v1.prompts.AnswersCreateResponse` | Client | moved |
| `dokime.v1.Reports/Get` | `dokime.client.v1.Reports/Get` | `dokime.client.v1.ReportsGetRequest` | `dokime.client.v1.ReportsGetResponse` | Client | moved |
| `dokime.v1.Reports/Create` | `dokime.client.v1.Reports/Create` | `dokime.client.v1.ReportsCreateRequest` | `dokime.client.v1.ReportsCreateResponse` | Client | moved |
| `dokime.v1.Reports/Publish` | `dokime.client.v1.Reports/Publish` | `dokime.client.v1.ReportsPublishRequest` | `dokime.client.v1.ReportsPublishResponse` | Client | moved |
| `dokime.v1.Secrets/Get` | `dokime.client.v1.Secrets/Get` | `dokime.client.v1.SecretsGetRequest` | `dokime.client.v1.SecretsGetResponse` | Client | moved |
| `dokime.v1.Secrets/Set` | `dokime.client.v1.Secrets/Set` | `dokime.client.v1.SecretsSetRequest` | `dokime.client.v1.SecretsSetResponse` | Client | moved |
| `dokime.v1.Secrets/List` | `dokime.client.v1.Secrets/List` | `dokime.client.v1.SecretsListRequest` | `dokime.client.v1.SecretsListResponse` | Client | moved |
| `dokime.v1.Secrets/Watch` | `dokime.client.v1.Secrets/Watch` | `dokime.client.v1.SecretsWatchRequest` | `dokime.client.v1.SecretsWatchResponse` | Client | moved |
| `dokime.v1.Secrets/Delete` | `dokime.client.v1.Secrets/Delete` | `dokime.client.v1.SecretsDeleteRequest` | `dokime.client.v1.SecretsDeleteResponse` | Client | moved |
| `dokime.v1.Secrets/Export` | `dokime.client.v1.Secrets/Export` | `dokime.client.v1.SecretsExportRequest` | `dokime.client.v1.SecretsExportResponse` | Client | moved |
| `dokime.v1.Secrets/Import` | `dokime.client.v1.Secrets/Import` | `dokime.client.v1.SecretsImportRequest` | `dokime.client.v1.SecretsImportResponse` | Client | moved |
| `dokime.v1.Settings/Get` | `dokime.client.v1.Settings/Get` | `dokime.client.v1.SettingsGetRequest` | `dokime.client.v1.SettingsGetResponse` | Client | moved |
| `dokime.v1.Settings/Set` | `dokime.client.v1.Settings/Set` | `dokime.client.v1.SettingsSetRequest` | `dokime.client.v1.SettingsSetResponse` | Client | moved |
| `dokime.v1.Settings/Schema` | `dokime.client.v1.Settings/Schema` | `dokime.client.v1.SettingsSchemaRequest` | `dokime.client.v1.SettingsSchemaResponse` | Client | moved |
| `dokime.v1.storage.Sources/Get` | `dokime.client.v1.storage.Sources/Get` | `dokime.client.v1.storage.SourcesGetRequest` | `dokime.client.v1.storage.SourcesGetResponse` | Client | moved |
| `dokime.v1.storage.Sources/Set` | `dokime.client.v1.storage.Sources/Set` | `dokime.client.v1.storage.SourcesSetRequest` | `dokime.client.v1.storage.SourcesSetResponse` | Client | moved |
| `dokime.context.v1.Backend/Environment` | `dokime.plugin.host.v1.Backend/Environment` | `dokime.plugin.host.v1.BackendEnvironmentRequest` | `dokime.plugin.host.v1.BackendEnvironmentResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.content.Artifacts/Create` | `dokime.plugin.host.v1.content.Artifacts/Create` | `dokime.plugin.host.v1.content.ArtifactsCreateRequest` | `dokime.plugin.host.v1.content.ArtifactsCreateResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.Documents/AppendParts` | `dokime.plugin.host.v1.Documents/AppendParts` | `dokime.plugin.host.v1.DocumentsAppendPartsRequest` | `dokime.plugin.host.v1.DocumentsAppendPartsResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.Effects/Create` | `dokime.plugin.host.v1.Effects/Create` | `dokime.plugin.host.v1.EffectsCreateRequest` | `dokime.plugin.host.v1.EffectsCreateResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.Effects/Get` | `dokime.plugin.host.v1.Effects/Get` | `dokime.plugin.host.v1.EffectsGetRequest` | `dokime.plugin.host.v1.EffectsGetResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.Effects/Set` | `dokime.plugin.host.v1.Effects/Set` | `dokime.plugin.host.v1.EffectsSetRequest` | `dokime.plugin.host.v1.EffectsSetResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.Evidence/Statistics` | `dokime.plugin.host.v1.Evidence/Statistics` | `dokime.plugin.host.v1.EvidenceStatisticsRequest` | `dokime.plugin.host.v1.EvidenceStatisticsResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.evidence.Logs/Append` | `dokime.plugin.host.v1.evidence.Logs/Append` | `dokime.plugin.host.v1.evidence.LogsAppendRequest` | `dokime.plugin.host.v1.evidence.LogsAppendResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.evidence.Records/Append` | `dokime.plugin.host.v1.evidence.Records/Append` | `dokime.plugin.host.v1.evidence.RecordsAppendRequest` | `dokime.plugin.host.v1.evidence.RecordsAppendResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.evidence.Records/List` | `dokime.plugin.host.v1.evidence.Records/List` | `dokime.plugin.host.v1.evidence.RecordsListRequest` | `dokime.plugin.host.v1.evidence.RecordsListResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.evidence.Traceability/Get` | `dokime.plugin.host.v1.evidence.Traceability/Get` | `dokime.plugin.host.v1.evidence.TraceabilityGetRequest` | `dokime.plugin.host.v1.evidence.TraceabilityGetResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.evidence.Traceability/Set` | `dokime.plugin.host.v1.evidence.Traceability/Set` | `dokime.plugin.host.v1.evidence.TraceabilitySetRequest` | `dokime.plugin.host.v1.evidence.TraceabilitySetResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.executions.Progress/Get` | `dokime.plugin.host.v1.executions.Progress/Get` | `dokime.plugin.host.v1.executions.ProgressGetRequest` | `dokime.plugin.host.v1.executions.ProgressGetResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.executions.Progress/Set` | `dokime.plugin.host.v1.executions.Progress/Set` | `dokime.plugin.host.v1.executions.ProgressSetRequest` | `dokime.plugin.host.v1.executions.ProgressSetResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.Files/Capture` | `dokime.plugin.host.v1.Files/Capture` | `dokime.plugin.host.v1.FilesCaptureRequest` | `dokime.plugin.host.v1.FilesCaptureResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.files.Captures/Start` | `dokime.plugin.host.v1.files.Captures/Start` | `dokime.plugin.host.v1.files.CapturesStartRequest` | `dokime.plugin.host.v1.files.CapturesStartResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.files.Captures/Stop` | `dokime.plugin.host.v1.files.Captures/Stop` | `dokime.plugin.host.v1.files.CapturesStopRequest` | `dokime.plugin.host.v1.files.CapturesStopResponse` | Plugin Host (Public) | moved |
| `dokime.context.v1.Prompts/Create` | `dokime.plugin.host.v1.Prompts/Create` | `dokime.plugin.host.v1.PromptsCreateRequest` | `dokime.plugin.host.v1.PromptsCreateResponse` | Plugin Host (Public) | moved |
| `<template>.materialize` | `<template>.materialize` | `dokime.plugin.v1.TestCasesMaterializeRequest` | `dokime.plugin.v1.TestCasesMaterializeResponse` | Plugin (Protected) | done |
| `<template>.initialize` | `<template>.initialize` | `dokime.plugin.v1.ExecutionsInitializeRequest` | `dokime.plugin.v1.ExecutionsInitializeResponse` | Plugin (Protected) | done |
| `Step.operation (plugin-authored dotted identity)` | `Step.operation (plugin-authored dotted identity)` | `dokime.plugin.v1.StepsExecuteRequest` | `dokime.plugin.v1.StepsExecuteResponse` | Plugin (Protected) | done |
| `Criterion.operation (plugin-authored dotted identity)` | `Criterion.operation (plugin-authored dotted identity)` | `dokime.plugin.v1.CriteriaEvaluateRequest` | `dokime.plugin.v1.CriteriaEvaluateResponse` | Plugin (Protected) | done |
| `<template>.cleanup` | `<template>.cleanup` | `dokime.plugin.v1.ExecutionsCleanupRequest` | `dokime.plugin.v1.ExecutionsCleanupResponse` | Plugin (Protected) | done |
| `dokime.settings.read` | `harmos.context.v1.HostValues/Get` | `dokime.plugin.host.v1.ReadSettings` | `dokime.plugin.host.v1.SavedSettings` | Plugin Host | awaits composition |
| `dokime.secrets.read` | `harmos.context.v1.HostValues/Get` | `dokime.plugin.host.v1.ReadSecret` | `dokime.plugin.host.v1.Secret` | Plugin Host | awaits composition |
| `dokime.execution.artifact (byte transfer)` | `harmos.context.v1.Storage/AppendFileBytes` | `dokime.plugin.host.v1.content.PublishArtifact` | `dokime.plugin.host.v1.content.ArtifactReceipt` | Plugin Host | awaits composition |
| `harmos.v1.status.Diagnostics/Get` | `harmos.v1.status.Diagnostics/Get` | `harmos.v1.status.DiagnosticsGetRequest` | `harmos.v1.status.DiagnosticsGetResponse` | Client | done |
| `harmos.v1.Status/Get` | `harmos.v1.Status/Get` | `harmos.v1.StatusGetRequest` | `harmos.v1.StatusGetResponse` | Client | done |
| `harmos.v1.status.Logs/Watch` | `harmos.v1.status.Logs/Watch` | `harmos.v1.status.LogsWatchRequest` | `harmos.v1.status.LogsWatchResponse` | Client | done |
| `harmos.v1.status.TraceEvents/Watch` | `harmos.v1.status.TraceEvents/Watch` | `harmos.v1.status.TraceEventsWatchRequest` | `harmos.v1.status.TraceEventsWatchResponse` | Client | done |
| `harmos.context.v1.Storage/GetFileBytes` | `harmos.context.v1.Storage/GetFileBytes` | `harmos.context.v1.StorageGetFileBytesRequest` | `harmos.context.v1.StorageGetFileBytesResponse` | Client | awaits composition |
| `harmos.context.v1.Storage/WatchFileBytes` | `harmos.context.v1.Storage/WatchFileBytes` | `harmos.context.v1.StorageWatchFileBytesRequest` | `harmos.context.v1.StorageWatchFileBytesResponse` | Client | awaits composition |
| `harmos.context.v1.Storage/CreateFileProduction` | `harmos.context.v1.Storage/CreateFileProduction` | `harmos.context.v1.StorageCreateFileProductionRequest` | `harmos.context.v1.StorageCreateFileProductionResponse` | Client | awaits composition |
| `harmos.context.v1.Storage/GetFileProduction` | `harmos.context.v1.Storage/GetFileProduction` | `harmos.context.v1.StorageGetFileProductionRequest` | `harmos.context.v1.StorageGetFileProductionResponse` | Client | awaits composition |
| `harmos.context.v1.Storage/CreateFileWriter` | `harmos.context.v1.Storage/CreateFileWriter` | `harmos.context.v1.StorageCreateFileWriterRequest` | `harmos.context.v1.StorageCreateFileWriterResponse` | Client | awaits composition |
| `harmos.context.v1.Storage/DeleteFileProduction` | `harmos.context.v1.Storage/DeleteFileProduction` | `harmos.context.v1.StorageDeleteFileProductionRequest` | `harmos.context.v1.StorageDeleteFileProductionResponse` | Client | awaits composition |
| `harmos.context.v1.Storage/PublishFileProduction` | `harmos.context.v1.Storage/PublishFileProduction` | `harmos.context.v1.StoragePublishFileProductionRequest` | `harmos.context.v1.StoragePublishFileProductionResponse` | Client | awaits composition |
| `harmos.v1.Plugins/Get` | `harmos.v1.Plugins/Get` | `harmos.v1.PluginsGetRequest` | `harmos.v1.PluginsGetResponse` | Client | awaits composition |
| `harmos.v1.Plugins/Create` | `harmos.v1.Plugins/Create` | `harmos.v1.PluginsCreateRequest` | `harmos.v1.PluginsCreateResponse` | Client | awaits composition |
| `harmos.v1.Plugins/List` | `harmos.v1.Plugins/List` | `harmos.v1.PluginsListRequest` | `harmos.v1.PluginsListResponse` | Client | awaits composition |
| `harmos.v1.Plugins/Restart` | `harmos.v1.Plugins/Restart` | `harmos.v1.PluginsRestartRequest` | `harmos.v1.PluginsRestartResponse` | Client | awaits composition |
| `harmos.v1.Plugins/Delete` | `harmos.v1.Plugins/Delete` | `harmos.v1.PluginsDeleteRequest` | `harmos.v1.PluginsDeleteResponse` | Client | awaits composition |
| `harmos.v1.Plugins/Set` | `harmos.v1.Plugins/Set` | `harmos.v1.PluginsSetRequest` | `harmos.v1.PluginsSetResponse` | Client | awaits composition |
| `harmos.v1.access.Pairings/Create` | `harmos.v1.access.Pairings/Create` | `harmos.v1.access.PairingsCreateRequest` | `harmos.v1.access.PairingsCreateResponse` | Client | done |
| `harmos.v1.access.Listener/Get` | `harmos.v1.access.Listener/Get` | `harmos.v1.access.ListenerGetRequest` | `harmos.v1.access.ListenerGetResponse` | Client | done |
| `harmos.v1.access.Certificates/Create` | `harmos.v1.access.Certificates/Create` | `harmos.v1.access.CertificatesCreateRequest` | `harmos.v1.access.CertificatesCreateResponse` | Client | done |
| `harmos.v1.access.Listener/Set` | `harmos.v1.access.Listener/Set` | `harmos.v1.access.ListenerSetRequest` | `harmos.v1.access.ListenerSetResponse` | Client | done |
| `harmos.context.v1.Time/Get` | `harmos.context.v1.Time/Get` | `harmos.context.v1.TimeGetRequest` | `harmos.context.v1.TimeGetResponse` | Plugin Host | done |
| `harmos.context.v1.Time/Sleep` | `harmos.context.v1.Time/Sleep` | `harmos.context.v1.TimeSleepRequest` | `harmos.context.v1.TimeSleepResponse` | Plugin Host | done |

Decision: schema retrieval returns descriptors only; display identities stay in the detail.
Descriptor files are the page unit; each encoded response obeys the smaller of
`PageRequest.max_bytes` (when set) and the existing host unary message limit.

Each lifecycle binding keeps its own Protected visibility, policy and handler;
only its shared request/response pair is reused. There is no lifecycle dispatch
service. Documents.Get retains its ordered bounded frames, counters and fixed cut.
No field number, scalar type, cardinality, oneof, reserved tag or streaming shape
changes in this amendment. ReadSettings/SavedSettings, ReadSecret/Secret and
PublishArtifact/ArtifactReceipt above are the retained payloads, not the future
library request/response shapes; their rows deliberately await composition.

Product RecordSchemas and StreamSchemas remain distinct from library registries:
`crates/dokime-operations-next/src/client/catalog_inventory.rs` projects admitted
and frozen record contracts/stream rows; `catalog_record_schemas_list.rs` and
`catalog_stream_schemas_list.rs` select those projections by parent. At the pinned
library revision, `crates/harmos-operations/src/host/catalog.rs` instead reads
StorageSchema and StoredIdentity for its schema/stream kinds. Keep product
RecordSchemas/StreamSchemas; do not merge them with library Schemas/Streams.

The former Catalog.Analyses becomes Analyses.List, preserving its single method
and payload fields; Suites, RecordSchemas, StreamSchemas and PromptDefinitions
lose their catalog parent. SettingsCatalog and PromptCatalog become the shared
SettingsDeclarations and PromptDefinitions message names. `FrozenSchemas` carries
only a SHA-256 descriptor digest plus record/stream identities. Its former embedded
`harmos.v1.Catalog` is removed. `Executions.Schemas.Get` returns the complete frozen
`FileDescriptorSet` in file order through bounded pages, independently of plugin
availability. SDK caches retain complete sets by digest, including across executions.
No product Catalog declaration remains.

### Protocol File Inventory

All paths below are relative to `proto/dokime/`; braces enumerate files.

- `v1/{analyses,appearance,cycles,documents,empty,evidence,execution,expressions,files,formatting,metadata,notes,parameters,plots,prompts,record_contracts,scalar,schemas,settings,suites,traceability}.proto`
- `client/v1/{aggregates,analyses,backend,backend/overview,cases,content,content/artifacts,content/execution_files,cycles,documents,evidence,evidence/health_events,evidence/notes,evidence/records,evidence/samples,evidence/traceability,executions,executions/commands,executions/progress,executions/schemas,media/captures,plots/settings,plots/snapshots,plugins,plugins/settings,prompt_definitions,prompts,prompts/answers,record_schemas,reports,secrets,settings,storage/sources,stream_schemas,suites}.proto`
- `plugin/v1/{criteria,executions,steps,test_cases}.proto`
- `plugin/host/v1/{backend,content/artifacts,documents,effects,evidence,evidence/logs,evidence/records,evidence/traceability,executions/progress,files,files/captures,prompts,secrets,settings}.proto`

Shared files contain messages and enums only:

| Shared File | Vocabulary |
| --- | --- |
| `v1/analyses.proto` | Immutable analysis declarations, selections and plugin operation payloads |
| `v1/appearance.proto` | Asset references and metadata shared by appearance and content |
| `v1/cycles.proto` | Cycle summaries, phases, states and archive selection |
| `v1/documents.proto` | Document authoring, cards, summaries and selection scope |
| `v1/empty.proto` | Empty response shared by retained ordinary services |
| `v1/evidence.proto` | Evidence scope, origins, conditions, time windows, severity and artifact metadata |
| `v1/execution.proto` | Frozen cases/origins, steps, criteria, outcomes and progress values |
| `v1/expressions.proto` | Data, aggregate, transform and filter expression trees |
| `v1/files.proto` | File selection and captured-file/gap vocabulary |
| `v1/formatting.proto` | Scalar presentation, units and time formats |
| `v1/metadata.proto` | Named metadata and typed attributes |
| `v1/notes.proto` | Annotation ownership and selection |
| `v1/parameters.proto` | Parameter definitions, expansion and expression trees |
| `v1/plots.proto` | Plot declarations, query options, shared grids and rendering enums |
| `v1/prompts.proto` | Prompt definitions, validation, actions, values and responses |
| `v1/record_contracts.proto` | Plugin record declarations and listener bindings |
| `v1/scalar.proto` | Typed scalar/list values, null and maps |
| `v1/schemas.proto` | Record/stream schemas and fields |
| `v1/settings.proto` | Plugin settings declarations and launch operation payload |
| `v1/suites.proto` | Suite and template declarations |
| `v1/traceability.proto` | Named traceability entries |

The former resources.proto, catalog_types.proto, document_types.proto, values.proto,
host.proto, catalog.proto and client_pending_*.proto are dissolved. Message owners
are determined by resource use across both boundaries; public plugin declaration
types remain shared with their client readers. Unreferenced analysis request/results
and prompt-section declarations remain shared public vocabulary. Appearance failures
belong to Backend. There are no aliases, compatibility files or catch-all payload files.

### Generated Module Rename Recipe

The protocol Rust exports are now `v1` (shared), `client`, `plugin` and `context`
(the latter exports `dokime.plugin.host.v1`). Generated modules follow the actual
package tree. Apply the table to operation IDs, descriptor lookups, raw gRPC paths,
frozen type names, fixtures and imports together; do not change library names.

| Old Module or Namespace | New Module or Namespace |
| --- | --- |
| `dokime.v1.<resource wrappers>` | `dokime.client.v1.<resource wrappers>` |
| `dokime.v1.{backend,content,evidence,executions,media,plots,plugins,prompts,storage}` | `dokime.client.v1.<same nested package>` |
| `dokime.v1.catalog.<Message>` | `dokime.client.v1.<Message>` (top level) |
| `dokime.context.v1[.<nested>]` | `dokime.plugin.host.v1[.<same nested>]` |
| `dokime.plugin.v1` | `dokime.plugin.v1` (unchanged) |
| `dokime_protocol::v1::<resource wrapper>` | `dokime_protocol::client::<resource wrapper>` |
| `dokime_protocol::v1::<nested>::<Message>` | `dokime_protocol::client::<nested>::<Message>` |
| `dokime_protocol::v1::catalog::<Message>` | `dokime_protocol::client::<Message>` |
| `dokime_protocol::context[::<nested>]` | Same Rust export; descriptor identity becomes `dokime.plugin.host.v1[.<nested>]` |
| `dokime_protocol::dokime::context::v1` | `dokime_protocol::dokime::plugin::host::v1` |
| `Dokime.Protocol.V1.<resource wrapper>` | `Dokime.Protocol.Client.V1.<resource wrapper>` |
| `Dokime.Protocol.V1.<Name>Resources` | `Dokime.Protocol.Client.V1.<Name>Resources` |
| `Dokime.Protocol.V1.CatalogResources` | `Dokime.Protocol.Client.V1` |
| `Dokime.Protocol.Context.V1[.<Name>]` | `Dokime.Protocol.Plugin.Host.V1[.<Name>Resources]` |
| `dokime::v1::<resource wrapper>` | `dokime::client::v1::<resource wrapper>` |
| `dokime::v1::catalog::<Message>` | `dokime::client::v1::<Message>` |
| `dokime::v1::<nested>` | `dokime::client::v1::<nested>` |
| `dokime::context::v1[::<nested>]` | `dokime::plugin::host::v1[::<nested>]` |

All nested C# client and plugin-host packages use the `<Name>Resources` namespace
suffix, including ContentResources, EvidenceResources, ExecutionsResources and
FilesResources. Root Client.V1 and Plugin.Host.V1 do not gain a suffix. Plugin.V1
and shared Protocol.V1 stay unchanged, except for explicitly relocated types.

The following payload relocation table is the exception list for former shared
`dokime.v1` imports. Types not listed retain `dokime.v1`. Request/response wrappers
already appear in the wire table; all other moved messages/enums are listed here.

| Previous Qualified Package | Final Qualified Package | Messages and Enums |
| --- | --- | --- |
| `dokime.context.v1` | `dokime.plugin.host.v1` | `EffectWritable` |
| `dokime.v1` | `dokime.client.v1` | `AddRemotePlugin`, `AdmittedPlugin`, `AdmittedPluginCursor`, `AdmittedPluginPage`, `AggregateReply`, `AppearanceAssetFrame`, `AppearanceAssetFramePayload`, `AppearanceFailure`, `AppearanceFailureKind`, `AppearanceLogo`, `AppearanceSnapshot`, `ApplicationAppearance`, `ApplicationSettings`, `ArtifactChunk`, `ArtifactCompletion`, `BootstrapPaths`, `CachedRefresh`, `CaseActions`, `CaseDetail`, `CaseParameter`, `CaseStatus`, `CaseSummary`, `ClientPreferences`, `ContentDownloadAppearanceAssetOutput`, `CriterionStatus`, `CycleFilter`, `CycleOrder`, `CycleOrderKey`, `CycleSelection`, `CycleWritable`, `Density`, `Diagnostic`, `DocumentBinding`, `DocumentBindingResult`, `DocumentCard`, `DocumentCardBody`, `DocumentCardBudget`, `DocumentCardFailure`, `DocumentCardKind`, `DocumentCompletion`, `DocumentDensity`, `DocumentEvidenceCut`, `DocumentFrame`, `DocumentFrameFrame`, `DocumentFrontiers`, `DocumentHeader`, `DocumentLayoutNode`, `DocumentLayoutNodeNode`, `DocumentOwnerReference`, `DocumentPage`, `DocumentPlacement`, `DocumentPlotDefinition`, `DocumentRail`, `DocumentRepeat`, `DocumentRepeatInstance`, `DocumentRepeatSelection`, `DocumentSection`, `DocumentSectionIdentity`, `DocumentSpan`, `DocumentSummary`, `DocumentSummaryCondition`, `DocumentSummaryFormat`, `DocumentSummaryGroup`, `DocumentSummaryGroupDefinition`, `DocumentSummaryPresentation`, `DocumentSummaryStat`, `DocumentSummaryState`, `DocumentSummaryTile`, `DocumentSummaryTileDefinition`, `DocumentTable`, `DocumentTableRow`, `DocumentTelemetryCut`, `DocumentWarning`, `EvidenceSubject`, `ExecutionBinding`, `ExecutionControls`, `ExecutionDetail`, `ExecutionFilter`, `ExecutionMode`, `ExecutionOrder`, `ExecutionOrderKey`, `ExecutionOutcomeCounts`, `ExecutionParameterSet`, `ExecutionSelection`, `ExecutionStep`, `ExecutionSummary`, `ExecutionWritable`, `FrozenSchemas`, `HostCardBlock`, `HostCardBlockKind`, `HostCardBody`, `HostCardKeyValue`, `HostCardKeyValueList`, `HostCardMetric`, `HostCardMetrics`, `HostCardStatus`, `HostCardTone`, `InstallPluginChunk`, `InstallPluginFrame`, `InstallPluginFrameFrame`, `InstallPluginHeader`, `InstallationCursor`, `InstallationPage`, `ListPlugins`, `LiveExecutionState`, `LogoVariant`, `ManagedPlugin`, `ManualRefresh`, `MaterializedParameter`, `OnDemandRefresh`, `OrganizationIdentity`, `Output`, `ParameterSource`, `PlannedCase`, `PlotAxisKind`, `PlotPresentation`, `PlotReferenceBand`, `PlotReferenceLine`, `PlotReferenceValue`, `Plugin`, `PluginEntries`, `PluginIdentity`, `PluginInstallation`, `PluginInstallationState`, `PluginMetadata`, `PluginPreferences`, `PluginState`, `PluginsInstallInput`, `PromptInstance`, `PromptKind`, `PromptView`, `QueryAggregate`, `QueryFilter`, `QueryFilterOperator`, `QueryRow`, `QuerySource`, `RegisteredSelection`, `RegisteredSelectionFilesEntry`, `RegisteredSelectionParametersEntry`, `RemoteListenerSettings`, `RemotePlugin`, `RemovePlugin`, `ReportDefaults`, `ReportMetadata`, `ReportOrigin`, `ReportState`, `ReportStatus`, `ReportSubject`, `RerunCaseNotice`, `SecretArchive`, `SecretImportAction`, `SecretImportChoice`, `SecretImportEntry`, `SecretImportPreview`, `SecretSnapshot`, `SecretSummary`, `SecretWritable`, `SelectedFile`, `SelectedFileKind`, `SelectionPlan`, `SelectionValues`, `SettingsCard`, `SettingsCardRefreshPolicy`, `SettingsCardRefreshPolicyPolicy`, `SettingsCardState`, `SettingsFrame`, `SettingsSection`, `SettingsSectionOwner`, `Statistics`, `StepCriterion`, `StepStatus`, `TelemetryPreferences`, `ThemeCornerRadii`, `ThemeFamily`, `ThemeMode`, `ThemeModePolicy`, `ThemeVariant` |
| `dokime.v1` | `dokime.client.v1.backend` | `EvidenceCounts`, `RuntimeShellSnapshot` |
| `dokime.v1` | `dokime.client.v1.content` | `ExecutionFileKind`, `ExecutionFileMetadata`, `ExecutionFileState` |
| `dokime.v1` | `dokime.client.v1.evidence` | `Bucket`, `BucketValue`, `CapturedErrorCause`, `CapturedRecord`, `CapturedRecordError`, `CapturedRecordSelection`, `CycleScope`, `HealthItem`, `OperatorNote`, `RecordScope`, `SampleRow`, `StatisticValue`, `TelemetrySample`, `TraceabilitySnapshot` |
| `dokime.v1` | `dokime.client.v1.executions` | `ExecutionCommandAction`, `ExecutionCommandPhase`, `ExecutionCommandReceipt` |
| `dokime.v1` | `dokime.client.v1.media` | `RecordingAudio`, `RecordingProgress` |
| `dokime.v1` | `dokime.client.v1.plots` | `AnalysisComputation`, `AnalysisEvidence`, `AnalysisSeries`, `ContourGrid`, `GridCell`, `GridFrame`, `GridFrames`, `NullableNumber`, `PlotAnalysis`, `PlotAnalysisError`, `PlotAnalysisResult`, `PlotBoxSummary`, `PlotBoxSummaryAnalysis`, `PlotCartesianMode`, `PlotColumn`, `PlotDomain`, `PlotEnvelope`, `PlotFrame`, `PlotHistogramAnalysis`, `PlotHistogramBin`, `PlotHistogramSeries`, `PlotHistogramStatistics`, `PlotMarker`, `PlotPayload`, `PlotPayloadDiagnostics`, `PlotSeries`, `PlotSeriesSummary`, `PlotSigmaBin`, `PlotSigmaIntervalAnalysis`, `PlotSigmaSeries`, `PlotStreamKind`, `PlotStreamUpdate` |
| `dokime.v1` | `dokime.client.v1.plugins` | `SecretAvailability` |
| `dokime.v1` | `dokime.client.v1.storage` | `StorageSource`, `StorageSources` |
| `dokime.v1` | `dokime.plugin.host.v1` | `ReadSecret`, `ReadSettings`, `SavedSettings`, `Secret` |
| `dokime.v1` | `dokime.plugin.host.v1.content` | `ArtifactReceipt`, `PublishArtifact` |
| `dokime.v1` | `dokime.plugin.host.v1.evidence` | `Record` |
| `dokime.v1` | `dokime.plugin.host.v1.files` | `CaptureMode`, `LineField`, `LineMarker`, `LineMatcher`, `MirrorCapture`, `StreamCapture` |
| `dokime.v1` | `dokime.v1` | `PromptCatalog → PromptDefinitions`, `SettingsCatalog → SettingsDeclarations` |
| `dokime.v1.evidence` | `dokime.client.v1.evidence` | `NoteWritable` |
| `dokime.v1.media` | `dokime.client.v1.media` | `Capture`, `CaptureKind` |
| `dokime.v1.plots` | `dokime.client.v1.plots` | `PlotSettingsValue` |

The resource-local file moves also change Python `_pb2` modules and C++ `.pb.h`
include paths even when a type stays in the same package. Replace imports/includes
from resources, catalog_types, document_types, values and host using the file
inventory above and the generated descriptor owner for that type. Shared packages
do not imply a shared file: e.g. Metadata is in metadata, Value is in scalar,
Document is in documents, Plot is in plots and DataExpression is in expressions.

### Host Handlers and Operations

- Regenerated Rust modules: `dokime.client.v1` and its backend/content/evidence/executions/media/plots/plugins/prompts/storage children; `dokime.plugin.host.v1` and content/evidence/executions/files children; shared `dokime.v1`; unchanged `dokime.plugin.v1`.
- Mechanical recipe: apply the Rust rows above to service trait/client/server imports and operation wrappers; move former `v1::catalog::*` to `client::*`; apply the payload exception table; keep `context::*` imports while rebinding descriptor/operation IDs to plugin.host. Rename CatalogAnalyses to AnalysesList, SettingsCatalog to SettingsDeclarations and PromptCatalog to PromptDefinitions. Compose the retained library destinations in their separate work.

### Rust SDK

- Regenerated output: `sdk/rust/src/client/resources.rs` and the protocol modules listed for the host. The generator resolves product client request/response types through `dokime_protocol::client`.
- Mechanical recipe: `dokime_protocol::v1::<operation wrapper> -> dokime_protocol::client::<operation wrapper>`; `v1::<nested> -> client::<nested>`; `v1::catalog::<Message> -> client::<Message>`; use the payload exception table for former shared data. `context` remains the handler object/module name; replace descriptor strings with plugin.host. Client roots keep their resource hierarchy except the explicitly flattened catalog resources (`client.suites`, `client.record_schemas`, `client.stream_schemas`, `client.prompt_definitions`, `client.analyses.list`).

### Python SDK

- Regenerated root metadata: `sdk/python/src/dokime/resources.json`; protobuf imports become `dokime.client.v1.*_pb2`, `dokime.plugin.host.v1.*_pb2`, shared named `dokime.v1.*_pb2` and unchanged `dokime.plugin.v1.*_pb2`. Generate protobuf messages through the SDK task before adapting helpers.
- Move all resource imports and frozen descriptors by file owner; remove catalog parent imports; use `context` for the handler object. Migrate lifecycle/context helpers, typed wrappers, examples and tests against the wire table.

### C# SDK

- Regenerated outputs: `sdk/csharp/Dokime/Resources.cs` and `ClientResources.cs`. Proto namespaces are `Dokime.Protocol.Client.V1`, its `<Name>Resources` children, `Dokime.Protocol.Plugin.Host.V1`, its `<Name>Resources` children, shared `Dokime.Protocol.V1` and unchanged `Dokime.Protocol.Plugin.V1`.
- Mechanical recipe: `Dokime.Protocol.V1.<resource wrapper> -> Dokime.Protocol.Client.V1.<resource wrapper>`; `Dokime.Protocol.V1.<Name>Resources -> Dokime.Protocol.Client.V1.<Name>Resources`; `CatalogResources.<Message> -> Client.V1.<Message>`; `Dokime.Protocol.Context.V1.<Name> -> Dokime.Protocol.Plugin.Host.V1.<Name>Resources`. Apply the payload exception table, rename the three declaration families, rebuild generated/frozen descriptors and keep the Context object.

### C++ SDK

- Regenerated output: `sdk/cpp/include/dokime/resources.hpp`. Message namespaces become `dokime::client::v1[::<nested>]` and `dokime::plugin::host::v1[::<nested>]`; shared `dokime::v1` and plugin `dokime::plugin::v1` remain.
- Mechanical recipe: `dokime::v1::<resource wrapper> -> dokime::client::v1::<resource wrapper>`; `dokime::v1::<nested> -> dokime::client::v1::<nested>`; `dokime::v1::catalog::<Message> -> dokime::client::v1::<Message>`; `dokime::context::v1 -> dokime::plugin::host::v1`. Apply the payload exception table and map `dokime/v1/<resource>.pb.h -> dokime/client/v1/<resource>.pb.h`, `dokime/context/v1/<resource>.pb.h -> dokime/plugin/host/v1/<resource>.pb.h`; shared includes follow the named-file inventory. Keep the context handler object and update publication/frozen-descriptor fixtures.

### Command-Line Tool

- Regenerated modules: Rust client/shared/context exports and nested modules listed above; generated Rust SDK roots. Start from final wire identities, including flattened registry resources.
- Migrate typed/raw calls, discovery, examples, retained paging, revisioned mutations and library content workflows. Apply the payload exception table; do not mechanically move shared vocabulary into the client package.

### Desktop Adapter

- Generated frontend/native message paths become `dokime/client/v1/{backend,content,evidence,executions,media,plots,plugins,prompts,storage}` plus root client resource files, `dokime/plugin/host/v1` child-call messages, shared named `dokime/v1` files and unchanged plugin files.
- Regenerate the desktop adapter from final descriptors after host migration; migrate request/response decoding and typed/raw call paths. Preserve document frame ordering/cuts/counters, settings revisions and direct library workflows. Native and both WebView journeys remain consumer-owned proof.
