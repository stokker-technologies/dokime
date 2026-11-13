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
Execution progress follows manager transitions and questions; each execution
settles independently. Cancelling either watch leaves its run active.
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
then prints the typed JSON response. Use `--input-file
request.json` for file input. Stream contracts require the typed streaming client;
refusals name the requested operation. Policies run on the host; the CLI adds no
mutation retries.

`dokime-next version --json` reports the compiled package version, source revision
(with `-dirty` for tracked local edits), publication channel, OS and architecture.
Local builds have a null channel; release and Nightly publishers identify theirs
from the build environment. Source archives without Git or CI revision metadata
report a null revision.

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

The desktop native package is temporarily excluded from Cargo membership while
its independent host switch is completed. Restore workspace/default membership,
native Product builds, generated settings checks, hosted native/system stories,
benchmarks, distribution/installer packaging and signing together using the task
in `openspec/changes/grpc-plugins/tasks.md`. The frontend source remains present.

## Context Protocol Packages

Plugin child calls for records, logs, statistics, prompts, document parts,
file capture and environment use `dokime.context.v1` and its nested packages.
Their wrappers are `<Service><Method>Request/Response`; identities are
`package.Service/Method`. These declarations add no client gRPC entry methods.
Environment and Time.Get now resolve through the active invocation and the
host clock. Document and file-capture declarations still need their remaining
handlers; the grammar task list retains those obligations.

The plugin lifecycle, client resource split, Progress/Traceability/Effects
revision contracts, HostValues and library plugin/transfer composition remain
open. The lifecycle binding-policy dependency has a failing consumer proof in
Library Requests. Existing context groups are replaced without wire aliases.

## Present Operation to Grammar Name

The locked client mapping below is the desktop adapter's rename inventory.
Each automatic `Run` twin maps to the same destination and is removed. A `+`
separates workflow calls. Service groups migrate with their consumers; the
operation-grammar task list records which replacements are implemented.

| Present Operation | Grammar Name |
| --- | --- |
| `Appearance.GetAppearance` | `Backend.Appearance` |
| `Backend.CollectDiagnostics` | `Status.Diagnostics.Get` |
| `Backend.GetAvailability` | `Status.Get` |
| `Backend.GetHealth` | `Status.Get` |
| `Backend.GetIdentity` | `Status.Get` |
| `Backend.WatchLogs` | `Status.Logs.Watch` |
| `Backend.WatchTrace` | `Status.TraceEvents.Watch` |
| `Catalog.GetRecordSchema` | `Catalog.RecordSchemas.Get` |
| `Catalog.GetSuite` | `Catalog.Suites.Get` |
| `Catalog.GetTelemetrySchema` | `Catalog.StreamSchemas.Get` |
| `Catalog.ListRecordSchemas` | `Catalog.RecordSchemas.List` |
| `Catalog.ListSuites` | `Catalog.Suites.List` |
| `Catalog.ListTelemetrySchemas` | `Catalog.StreamSchemas.List` |
| `Content.DownloadAppearanceAsset` | `Storage.GetFileBytes` |
| `Content.DownloadArtifact` | `Content.Artifacts.Get` + `Storage.GetFileBytes` |
| `Content.DownloadEvidence` | `Evidence.Archive` + `Storage.GetFileBytes` |
| `Content.DownloadExecutionFile` | `Content.ExecutionFiles.Get` + `Storage.GetFileBytes` |
| `Content.FollowExecutionFile` | `Content.ExecutionFiles.Get` + `Storage.WatchFileBytes` |
| `Content.ListArtifacts` | `Content.Artifacts.List` |
| `Content.ListExecutionFiles` | `Content.ExecutionFiles.List` |
| `Content.ReadExecutionFile` | `Content.ExecutionFiles.Get` |
| `Content.UploadArtifact` | `Storage.CreateFileProduction` + `Storage.GetFileProduction` + `Storage.CreateFileWriter` + `Storage.AppendFileBytes` + `Storage.DeleteFileProduction` + `Storage.PublishFileProduction` + `Content.Artifacts.Create` |
| `Documents.CaptureCurrentDocument` | `Documents.Get` |
| `Documents.UpdatePlotSettings` | `Plots.Settings.Set` |
| `Evidence.GetCapturedRecordSchema` | `Catalog.RecordSchemas.Get` |
| `Evidence.ListRecords` | `Evidence.Records.List` |
| `Evidence.WatchRecords` | `Evidence.Records.Watch` |
| `Execution.AppendExecutions` | `Executions.Create` |
| `Execution.CancelExecution` | `Executions.Cancel` |
| `Execution.CreateCycle` | `Cycles.Create` |
| `Execution.DeleteCase` | `Cases.Delete` |
| `Execution.GetCase` | `Cases.Get` |
| `Execution.GetCycle` | `Cycles.Get` |
| `Execution.GetExecution` | `Executions.Get` |
| `Execution.GetExecutionCommand` | `Executions.Commands.Get` |
| `Execution.ListCaseParameters` | `Cases.Parameters` |
| `Execution.ListCases` | `Cases.List` |
| `Execution.ListCycles` | `Cycles.List` |
| `Execution.ListExecutionParameters` | `Executions.Parameters` |
| `Execution.ListExecutionSteps` | `Executions.Steps` |
| `Execution.ListExecutions` | `Executions.List` |
| `Execution.PauseCycle` | `Cycles.Set` |
| `Execution.PauseExecution` | `Executions.Set` |
| `Execution.RenameCycle` | `Cycles.Set` |
| `Execution.RenameExecution` | `Executions.Set` |
| `Execution.RerunCycle` | `Cycles.Create` |
| `Execution.ResumeCycle` | `Cycles.Set` |
| `Execution.ResumeExecution` | `Executions.Set` |
| `Execution.SealCycle` | `Cycles.Seal` |
| `Execution.SetCycleArchived` | `Cycles.Set` |
| `Execution.SetExecutionArchived` | `Executions.Set` |
| `Execution.StartExecution` | `Executions.Start` |
| `Execution.StopExecution` | `Executions.Stop` |
| `Execution.SubmitExecutionCommand` | `Executions.Commands.Create` |
| `Execution.UpdateCyclePolicy` | `Cycles.Set` |
| `Health.ListHealth` | `Evidence.HealthEvents.List` |
| `History.BrowseCycles` | `Cycles.List` |
| `History.BrowseExecutions` | `Executions.List` |
| `History.DeleteExecution` | `Executions.Delete` |
| `History.DownloadCheckpointArtifact` | `Content.Artifacts.Get` + `Storage.GetFileBytes` |
| `History.GetCheckpointContext` | `Executions.Get` + `Cycles.Get` + `Documents.Get` + `Prompts.List` + `Evidence.HealthEvents.List` + `Evidence.Notes.List` + `Evidence.Traceability.Get` + `Backend.Appearance` |
| `History.GetCheckpointDocument` | `Documents.Get` |
| `History.GetCheckpointEvidence` | `Content.Artifacts.List` + `Content.ExecutionFiles.List` + `Catalog.RecordSchemas.List` + `Catalog.StreamSchemas.List` |
| `History.GetExecutionCheckpoint` | `Executions.Get` |
| `History.ListCheckpointRecords` | `Evidence.Records.List` |
| `History.ListCheckpointTelemetry` | `Evidence.Samples.List` |
| `History.ReadCheckpointOutput` | `Content.ExecutionFiles.Get` |
| `Media.AudioRecording` | `Media.Captures.Create` + `Media.Captures.Watch` + `Media.Captures.Stop` |
| `Media.CameraRecording` | `Media.Captures.Create` + `Media.Captures.Watch` + `Media.Captures.Stop` |
| `Media.Photo` | `Media.Captures.Create` |
| `Media.ScreenRecording` | `Media.Captures.Create` + `Media.Captures.Watch` + `Media.Captures.Stop` |
| `Media.Screenshot` | `Media.Captures.Create` |
| `Notes.CreateNote` | `Evidence.Notes.Create` |
| `Notes.EditNote` | `Evidence.Notes.Set` |
| `Notes.ListNotes` | `Evidence.Notes.List` |
| `Observation.GetRuntimeShell` | `Backend.Overview.Get` |
| `Observation.GetTraceability` | `Evidence.Traceability.Get` |
| `Observation.WatchExecutionProgress` | `Executions.Progress.Watch` |
| `Observation.WatchRuntime` | `Backend.Overview.Watch` |
| `Planning.MaterializeTemplate` | `Cases.Create` |
| `Planning.PreviewSelection` | `Cases.Create` |
| `Planning.RegisterSelection` | `Cases.Create` |
| `Plots.ListAnalyses` | `Catalog.Analyses` |
| `Plots.QuerySnapshots` | `Plots.Snapshots.List` |
| `Plots.Watch` | `Plots.Snapshots.Watch` |
| `Plugins.GetInstallation` | `Plugins.Get` |
| `Plugins.Install` | `Storage.CreateFileProduction` + `Storage.GetFileProduction` + `Storage.CreateFileWriter` + `Storage.AppendFileBytes` + `Storage.DeleteFileProduction` + `Storage.PublishFileProduction` + `Plugins.Create` |
| `Plugins.ListAdmittedPlugins` | `Plugins.List` |
| `Plugins.ListInstallations` | `Plugins.List` |
| `Plugins.Reload` | `Plugins.Restart` |
| `Plugins.Remove` | `Plugins.Delete` |
| `Plugins.SetEnabled` | `Plugins.Set` |
| `Plugins.List` | `Plugins.List` |
| `Plugins.Add` | `Plugins.Create` |
| `Prompts.GetDefinition` | `Catalog.PromptDefinitions.Get` |
| `Prompts.GetPrompt` | `Prompts.Get` |
| `Prompts.ListDefinitions` | `Catalog.PromptDefinitions.List` |
| `Prompts.ListPending` | `Prompts.List` |
| `Prompts.Respond` | `Prompts.Answers.Create` |
| `Query.Aggregate` | `Aggregates.Get` |
| `Query.AggregateSamples` | `Evidence.Statistics` |
| `Query.ListCycles` | `Cycles.List` |
| `Query.ListExecutions` | `Executions.List` |
| `Query.ListRecords` | `Evidence.Records.List` |
| `Query.ListSamples` | `Evidence.Samples.List` |
| `Remote.CreatePairing` | `Access.Pairings.Create` |
| `Remote.ReadListener` | `Access.Listener.Get` |
| `Remote.RegenerateCertificate` | `Access.Certificates.Create` |
| `Remote.UpdateListener` | `Access.Listener.Set` |
| `Reports.PublishExecutionReport` | `Reports.Publish` |
| `Reports.DownloadCycleReport` | `Reports.Get` + `Storage.GetFileBytes` + `Reports.Create` |
| `Reports.DownloadExecutionReport` | `Reports.Get` + `Storage.GetFileBytes` + `Reports.Create` |
| `Reports.MaterializeReport` | `Reports.Create` |
| `Runtime.GetHealth` | `Status.Get` |
| `Runtime.GetIdentity` | `Status.Get` |
| `Secrets.Export` | `Secrets.Export` |
| `Secrets.Import` | `Secrets.Import` |
| `Secrets.List` | `Secrets.List` |
| `Secrets.Lookup` | `Secrets.Get` |
| `Secrets.Preview` | `Secrets.Import` |
| `Secrets.Put` | `Secrets.Set` |
| `Secrets.Remove` | `Secrets.Delete` |
| `Settings.GetPluginSettings` | `Plugins.Settings.Get` |
| `Settings.GetSettings` | `Settings.Schema` |
| `Settings.ListStorageSources` | `Storage.Sources.Get` |
| `Settings.ReadApplicationSettings` | `Settings.Get` |
| `Settings.ReorderStorageSources` | `Storage.Sources.Set` |
| `Settings.SetStorageSourceEnabled` | `Storage.Sources.Set` |
| `Settings.UpdateApplicationSettings` | `Settings.Set` |
| `Settings.UpdatePluginSettings` | `Plugins.Settings.Set` |
| `Telemetry.ListSamples` | `Evidence.Samples.List` |
| `Telemetry.WatchSamples` | `Evidence.Samples.Watch` |
