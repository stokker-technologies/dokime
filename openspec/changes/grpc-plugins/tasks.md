# gRPC Plugins Tasks

## Desktop Plot Documents

The sole writer on `perf/desktop-plot-documents` owns this local-only repair.
Inventory: desktop plot selection/visibility and bounded group lifecycle; native
plot request and document adapters; host plot batch planning and trace boundaries;
Arrow preview/tail reads and bounded frequency selection; the plot declaration
and its generated consumers if a declaration is required; native/frontend/storage
plot and document stories; desktop/storage benchmark observations and maintained
architecture, GUI, authoring and tooling guidance. Existing storage and transport
memory bounds, durability, history sizes and timing-free stories remain required.
No specification lifecycle command or push is authorized for this work.

- [x] Measure the current request-to-draw/refusal path and identify its largest costs.
- [x] Select visible plots in bounded desktop groups, retaining independent cancellation and updates; close effect dependency and generation identity proof.
- [x] Share identical plot reads within one host batch and remove redundant full-history work; retain numeric grid-frame keys through reduction and format only returned instants.
- [x] Select bounded frequency input for long executions and preserve explicit author intent.
- [x] Prove documents exceeding one watch and frequency plots over long history deterministically; preserve source instants and categorical coordinates at the native renderer handoff exposed by whole-document traversal.
- [x] Update affected maintained guides and benchmark boundaries; review README impact. Existing README claims and links remain accurate.
- [x] On source head `ae0cfada8`, pass format, Rust/TypeScript/repository checks and the workspace test compiler check through mise; pass 182 frontend unit, 119 frontend integration, two native desktop and 18 storage stories on Windows. Complete 34 drawn desktop cases with eight complete traversals and nine storage document measurements; retain the inclusive stage profile and exact boundaries in tooling guidance. `cmdkey /list` leaves no entry from these runs; the 13 pre-existing access entries are preserved. Hosted Linux proof is not claimed. Commits remain local.

## Storage Benchmark Boundaries

This repair is owned by the sole writer on `fix/storage-bench-boundaries`;
commits stay local. Source inventory: the desktop system host initializer;
runtime and desktop benchmark sources and manifest/lock dependencies; their
benchmark producer/consumer tooling and focused Python stories; batched observer
admission in `crates/dokime-runtime-next/src/streams.rs` and its deterministic
round-trip story; the storage reader/cache only if measurements justify a change; `docs/tooling.md`,
`docs/architecture.md` and this task list. Outcome folding and the four outcome
regression story modules remain with their separate owner.

- [x] Repair the desktop host initializer alone, compile workspace tests and run the desktop plot/document system story.
- [x] Name store boundaries honestly; add full scans and freshly opened query, median and document cases with matching historical tile counts.
- [x] Restore current-host sample publication, execution end/shutdown and durable disk archive measurements, documenting unavailable old boundaries.
- [x] Load desktop history through the real tick-driven Samples path and obtain measurements, including explicit refusals under the unchanged frequency and 32-plot bounds.
- [x] Measure the complete bench-profile inventory and record comparable figures and remaining performance limits in the benchmark guide. Review README impact; its existing claims and links are unaffected.
- [x] Close the source inventory and focused proof for the local candidate; retain the mandatory committed-head gate below.

After committing this repair, the handoff requires `mise run format`, Rust and
repository checks, locked workspace test compilation, storage integration stories,
the desktop plot/document system story, every storage/runtime and desktop bench
case once in the bench profile, and a `cmdkey /list` comparison proving these runs
leave no additional `.dokime.access` entries. This is final proof, not a claim made
by source closure. Keep the two desktop refusals above distinct from successful
drawing and retain any other failing proof as a repair obligation.

## Face Split Story Repairs

This checkout has one writer on `fix/face-split-stories`; commits remain local.
Inventory: runtime plot services and operations plus their focused host stories;
Python host client story and `tests/support/thermal.rs` (final request IDs, statistics receipts and prose-free cross-SDK evidence comparisons); desktop WebView report stories and any demonstrated
connection defect, `documents_get.rs` section-identity frames and their fixed-document host story, and `sdk/rust/examples/thermal.rs` (the shared fixture report document); host fixture producers, C++ package dependency configuration/runtime-DLL staging, focused workspace-check dispatch and Rust
story launcher, its `release/sbom.py` wrapper and the CLI/desktop fixture launchers; README, AGENTS and operation-grammar completion checkboxes.
Preserve the library-owned discovery refusal in `operator_calls_operation`.

- [x] Repair the plot query stack shape and cover the failing query deterministically.
- [x] Read the final Python suite identity and prove the host stories.
- [x] Assert saved report identity in both WebView lanes; repair any independent connection failure.
- [x] Repair Windows fixture paths/compiler resolution and Rust launcher diagnostics/cause.
- [x] Correct starter guidance and merged face split completion inventory.
- [ ] Obtain passing final focused proof; `operator_calls_operation` remains blocked by the pinned library external-service registration API described in `docs/execution-host.md`. Run and report all other requested checks on the committed candidate.

## Focused Test Feedback

Source inventory: `tools/python/src/dokime_tools/{ci/tasks.py,build/{cargo,configuration,rust/tests}.py,test/{focused,archive,run}.py}`,
their tooling integration stories, `mise.toml`, retained packaging integration stories,
`crates/dokime-server/tests/support/artifacts.rs` (the Tests-owned host fixture path), and
`docs/tooling.md`. Product code and suites being replaced by waves 4–5 remain
with their implementation owners.

- [x] Measure Windows focused compilation, harness inventory, run costs and compiler contention; distinguish cold, warm and unmeasured native Linux results.
- [x] Route focused tests through nextest with canonical guards, story and changed-path selection, native compiler timings and explicit plans.
- [x] Consolidate retained test fixtures where measured duplication dominates, preserving all distinct refusal and round-trip branches.
- [x] Run each touched suite, format, Rust/repository checks and the Tests artifact task; retain local commits without pushing.
- [ ] Obtain native Linux proof in the combined delivery; the Windows validation failures below are repaired and the combined Windows checks and Tests producer pass.

Retain the existing harness boundaries: the workspace already groups integration stories
by package. Keep runtime workflow/recovery, embedded SDK and macro-consumer rewrites with
waves 4–5; their measured costs do not justify rebuilding superseded suites here. Focused
selection preserves deadlines, required target features and stamped fixture checks.
Linux timing and isolated per-harness linker timing remain unmeasured. Windows proof
still requires repair of unused protocol dependencies (`tonic`, `tonic-prost`), the
runtime threaded workflow failure under concurrent execution (its standalone rerun
passes), and legacy Python/TypeScript artifact prerequisites.
## PR 411 Desktop System Repair

Source inventory: `crates/dokime-gui/e2e/{native-desktop,system}.mjs`, the TypeScript
system task in `mise.toml`, `.github/workflows/{ci,nightly}.yml`, and `docs/tooling.md`.
Product changes are limited to a demonstrated cause of the reported regression.

- [x] Read Linux job 131065996838 and Windows job 130993783301, including retained Linux RPC and backend evidence.
- [x] Run isolated admission and the failing scenes on current Windows artifacts; repair native numeric entry and any demonstrated admission defect without extending deadlines.
- [x] Measure the Windows system lane before and after; keep the 600-second guard and split hosted work if the legitimate stories exceed it.
- [x] Run format, TypeScript and repository checks, retain local commits and report native Linux proof as pending hosted CI.

The Linux host plugin initialized in 1.051 s; StartExecution succeeded in 166 ms.
The failed numeric edit left the preceding execution at its operator prompt, so
the next execution remained queued behind it. This run does not reproduce the
separate host-plugin initialization deadline failure.
Current Windows artifacts also pass isolated `execution-stop` in 14.387 s with
host-plugin startup taking 3.626 s. The existing 46-test plot integration journey
passes, including digit retention across presentation refreshes. Bulk numeric
entry was introduced by `8e45a8216`; serial system stories by `8f7abf734`.
The original Windows Node lane passed in 572.718 s; the runner including
trace processing took 588.76 s (591.284 s including task setup). The split execution
Node lane passes in 258.855 s (276.75 s with trace processing). With separate native numeric key events,
`cycle`, `execution-stop` and `containers` pass in 43.715, 17.687 and 23.270 s.
The configuration Node selection passes in 455.119 s (474.66 s with trace processing),
so every selected story passes across the unchanged 600-second guards. Formatting,
TypeScript and repository checks pass. One successful
Windows restart's host-plugin start took 45.156 s; its retained trace is separate
from the Linux prompt-blocked admission and does not establish the cause of the
earlier initialization-deadline failure. Native Linux proof remains hosted.

## 1. Contract

- [x] 1.1 Settle the open points in `design.md` with the owner and record each outcome there.
- [x] 1.2 Reconcile the overlapping changes and write the spec deltas that retire superseded requirements.
- [x] 1.3 Lock the source inventory: producers, consumers, tests, fixtures, examples, tasks, README and guidance.
- [x] 1.2.1 Retire Python backend, native-wheel, embedded-callback, generation reload and dependency-start-order requirements; retain remote authoring, typed clients, diagnostics and focused proof requirements in replacement deltas.
- [x] 1.3.1 Protocol inventory: `proto/{dokime,harmos}/`, `crates/dokime-protocol/{src,examples,tests}`, client-service generation in `crates/dokime-server/`, document bindings under `sdk/{python,csharp,cpp}` and protocol export in `crates/dokime-cli/src/commands/proto.rs`; check generated drift and protobuf round trips.
- [x] 1.3.2 Host inventory: `crates/dokime-domain/src/model/`, `crates/dokime-operations/src/`, `crates/dokime-runtime/src/{catalog.rs,contributions.rs,domains,execution,runtime,storage,transactions,transport}`, and `crates/dokime-server/src/{grpc,host}`; migrate producers and consumers together, preserving effects, bounded queries/transfers, authorization and durability.
- [x] 1.3.3 Consumer inventory: `crates/{dokime,dokime-client,dokime-cli,dokime-packaging,cargo-dokime,dokime-gui/src-tauri}`, GUI source/generated clients and report assets, `sdk/`, `plugins/sidecars/{host,obc}` and examples/starters; migrate capabilities, origin/outcome, settings and catalog consumers with their producers.
- [x] 1.3.4 Removal/proof inventory: `crates/dokime-macros`, old execution workflow/provider/replay and embedded-plugin tests/fixtures, Python native bindings/stubs/backend and wheel tooling; retain meaningful effect/recovery/transfer/report/plot journeys under their replacement owners and use focused package targets only.
- [x] 1.3.5 Configuration/guidance inventory: workspace/package manifests and locks, build scripts, `mise.toml`, `tools/{project.toml,python}`, `.github/workflows`, README, `docs/{product-guide,tooling,gui}.md`, topic guides and `.agents/skills/dokime-{architecture,authoring,conventions}`. Update only affected current contracts; no customer or private plugin identifiers.
- [x] 1.3.6 Ownership/order: this worker owns the inventory except GUI benches, new bench files under GUI e2e, `tools/python/src/dokime_tools/test/run.py`, `tools/python/tests/integration/test_desktop_bench.py`, other bench tooling under `tools/` and `plot-refresh-scaling`; existing desktop stories and the report-browser fixture are owned here. Preserve pre-existing shared manifest/lock/tooling edits. Contract/protocol precede Harmos adoption, execution model precedes manager, SDKs precede removals; storage extraction and import admission remain later work. Each source checkbox requires focused native proof; final/platform validation stays separate.

## 2. Protocol

- [x] 2.1 Define and check in `dokime.v1` host operations, `TestCase`, `Step`, `Criterion` and `Statistics`; generate the protocol crate.
- [x] 2.1.1 Reproduce the hosted Windows interruption timeout five times by exact name; all five pass. Examined terminal-query servicing, batch acknowledgement, held mutation replies, admission watches and the frozen-origin commit; no cleanup regression or permanently unsent event was established.
- [ ] 2.2 Add the host operations plugins need that exist only as evidence-page fields today: progress, log, traceability, document parts, record reads and statistics.
- [ ] 2.3 Replace JSON catalog readers and producers with `dokime.v1` messages; add running-host, plugin URL and bundle export sources. Embedded host-source export is tracked in 2.6.
- [ ] 2.4 Serve settings as a host operation and delete settings injection at start and the applied-versus-saved comparison.
- [ ] 2.5 Add the per-execution resource store (`ctx.set`, `ctx.get`) for Rust and remove template fields and `Own`.
- [x] 2.6 Export embedded Dokime and Harmos protocol sources through `dokime proto export --out`, preserving package paths without opening storage.
- [x] 2.7 Attach the checked-in protocol sources to releases through the archive owned by `release-layout` task 2.3; the packaged-CLI export/archive path passes 11 focused release tooling tests and the complete CLI export/descriptor test on Windows. Current-source release packaging and hosted publication remain final proof.
- [ ] 2.8 Wave 2: make proto files authoritative, put generated definitions and light helpers in protocol, document cross-language semantics for each SDK, and inventory Harmos native state/recovery/storage semantics retained in `harmos-domain`; move wire mirrors out.
  - [x] Client Services: declare Plugins (9), Catalog (6), Planning (3), Execution (28), Observation (4), Reports (3), Health (1) and Backend (6) in new per-service proto files.
  - [x] Client Services: declare Appearance (1), Content (9), Documents (2), Evidence (3), History (11), Media (5), Notes (3), Plots (3), Prompts (5), Query (6), Remote (4), Runtime (2), Secrets (7), Settings (8) and Telemetry (2).
  - [x] Client Services: express all 129 served client operations plus canonical plugin List/Add, with the 131 matching cancellable Run-method routes (262 RPCs total); no served client operation is unexpressed. Internal provider-run and retired plugin evidence/lease calls have no client gRPC binding and are not client services.
  - [x] Client Services: reuse canonical values/catalog/documents/prompts and frozen origins; expose attributed outcomes, open/closed/sealed execution state, separate live state and managed/remote entries. Add Metadata tags/attributes/times and rename the colliding Prompts catalog payload to PromptCatalog; retain native percentile rank explicitly. Handler/client migration remains with 3.9–3.11 and the SDK workers.
  - [x] Client Services Isolated Proof: regenerate Rust/descriptors/reflection through the unchanged mise task in a protocol-only copy under target; focused build, seven protocol integration tests, Rust format and focused Rust lint pass. The generator proves the complete Python/C#/C++ proto set without writing SDK-owned files. The focused lint emits the existing root Clippy configuration's unreachable tokio process-path warning but exits successfully.
  - [x] Client Services Repository Proof: `mise run check -- --repository` passes; protoc 36.2 generates Python, C# and C++ for the complete Dokime/Harmos proto set under target/proto-sdk-proof. No handler or SDK source is edited.
  - [x] Client Services Worktree Proof: the combined worktree passes regeneration, all eight focused protocol tests (including descriptor drift and managed envelopes), the protocol package Rust check and whole-tree formatting. Whole-workspace Rust proof is tracked separately in branch convergence below.
- [x] 2.8.1 Export an authoritative descriptor set with checked-in generated messages; expose descriptor lookup for host operation declarations and prove generation drift.
- [x] 2.8.2 Document shared value, identifier, timestamp, resource, error, settings and operation semantics for all four SDKs; inventory retained Harmos native semantics against the requested revision.
- [ ] 2.8.3 Replace remaining Dokime wire mirrors and migrate their declarations and consumers; dissolve the domain owner in 5.9 after the host and SDK waves.
- [x] 2.8.3.1 Define canonical appearance/backend diagnostics messages and generate the protocol-owned embedded source inventory; prove drift and complete CLI export/descriptor agreement (five protocol tests and one focused CLI export test, focused CLI build and protocol/CLI lint on Windows). This inventory work does not close wire-mirror or handler adoption.
- [ ] 2.8.3.2 Protocol handoff: `dokime-protocol` has generated `dokime.v1` plugin messages and reflection but no client service declarations or generated client stubs. The SDK binds generated messages to discovered Harmos `Method` contracts; complete service declarations/generation in the protocol owner before promising service-wide client coverage. First declare Backend, Appearance and Plugins with canonical messages and Harmos managed-call envelopes; retain the other service areas and all handler adoption in this task.
- [x] 2.8.3.2.1 Declare Backend, Appearance and Plugins service methods and managed-call envelopes in their authoritative proto files; regenerate Rust client/server interfaces and descriptors. The service-envelope journey, generator drift and protocol package Rust check pass on Windows. This does not close the remaining services or native handler adoption.

- [ ] 2.9 Host-wave prerequisite owned here, before 3.2/3.3: define Origin with PluginRun containing frozen TestCase/plugin/template/digest/process start time and Import containing its definition/manifest; reserve removed TestCase identity fields, regenerate bindings and migrate persistence, materialization and plugin-call envelopes. No separate execution-definition or Run type.
- [x] 2.9.1 Define frozen Origin/PluginRun/Import protobuf messages, reserve TestCase identity fields and add origin to every plugin execution-call envelope; regenerate and prove round trips and origin agreement.
- [ ] 2.9.2 Migrate persisted execution definitions, catalog materialization and every execution-call consumer to the frozen origin; refuse incompatible development roots with explicit backup/reset guidance.
- [ ] 2.9.2.1 Replace `dokime-domain/src/model/execution/snapshot.rs` case/template/plugin snapshots and their storage/finalization consumers with the frozen protocol origin; keep execution identity/cycle separate and preserve existing roots on refusal.
- [ ] 2.9.2.2 Replace `model/definition/{suite,case_materialization}.rs` and runtime planning/catalog producers with parameter expansion plus pure plugin materialization; retain the returned case and provider digest/process identity together.
- [ ] 2.9.2.3 Migrate runtime execution-call producers, SDK handlers and client/report projections to the same origin/case envelopes; remove superseded snapshots/conversions in the same cutover and prove storage round trips and mismatched-origin refusal.

## 3. Host

- [x] 3.0.1 Record the owner's host-first order: new host beside the old loop, shared-suite proof, then a later CLI/desktop switch and complete old-code deletion.
- [x] 3.0.2 Add independent `dokime-storage`, `dokime-operations-next` and `dokime-runtime-next`; own only these crates, additive workspace membership/lock, `docs/execution-host.md` and this change. Keep old crates untouched.
- [x] 3.0.3 Prove the shared SDK thermal story with managed/remote admission, pure materialization and frozen origin, cycle lifecycle, scoped unary calls, host evidence/clock/streams, evaluated outcome, sealing and identical restart reads.
- [x] 3.0.4 Prove cancellation during a step with cleanup, cancelled termination, closed streams and sealed evidence under a controlled clock.
- [x] 3.0.5 Run format, Rust language check and each thermal story three consecutive times; commit locally without pushing. Both thermal stories pass three consecutive runs, and the attribution-counting unit passes. Formatting and the focused new-host Rust check pass; the root Rust check reports the SDK lint follow-up in 4.1.7.
- [ ] 3.0.6 Later cutover owner: switch CLI/desktop, delete the old host and its tests, rename `dokime-operations-next` and `dokime-runtime-next` to their target names before final full validation; the pinned scoped Harmos revision is already adopted by the new host and SDKs.
- [ ] 3.1 Adopt the Harmos `grpc-plugins` revision and add `Plugin::{Managed, Remote}` with the add and remove operations and entry states.
- [x] 3.1.1 Adopt the exact-revision protocol owner, remove the duplicate generated Harmos Rust module and generator output, and prove vendored descriptor parity plus the five protocol journeys on Windows. Native host, SDK and tool consumer adoption remains in 3.1/3.8.2; protocol adoption precedes it to keep the tree buildable.
- [ ] 3.2 After 2.9, move phases and criteria from the template to the test case and call `materialize` per parameter set, with rematerialization on a changed PluginRun-origin digest.
- [ ] 3.3 After 2.9 and 3.12–3.15, build the manager with live queued/initializing/stepping/evaluating/cleaning-up state and finished/cancelled/interrupted termination; invoke initialize, steps, criteria and cleanup with retries and interactive commands, addressing cancel to executions only while driven by a run.
- [x] 3.3.1 Bind the execution scope at new-host invocation and publication call sites, and adopt pinned Harmos scope APIs in every SDK. Pure materialization retains an empty scope; lifecycle envelopes inherit their execution identity.
- [ ] 3.4 Open and close catalog and test case streams, retain source time, receipt time and durable position with declared axes, and ingest live samples in arrival order without claiming the host owns source clocks.
- [ ] 3.5 Compute stream statistics per execution and step range.
- [x] 3.5.1 Execution-feature worker inventory: owned new runtime execution/host/dispatch/streams/plugin/clock and added modules, process-constructor visibility for the killed-host story, storage, required operation handlers, six ordered stories, SDK execution sample and its one-line consumer entrypoint; leave old host and client services untouched. Prove isolation, retries, outcomes, process interruption, render/publication and capabilities with focused tasks, then hand back local commits without pushing.
- [x] 3.5.2 Execution-feature focused handoff proof: format once, run each of the six new stories twice by exact name, rerun both shared thermal stories, check-build all three new crates and run their package Rust checks; keep integration cutover, full lanes and specification retirement with the orchestrator.
- [ ] 3.6 Replace Settings › Plugins with managed and remote entries and their states.
- [ ] 3.7 Adopt unary-only plugin operations, migrate streamed sample operations, apply internal visibility and unset-policy defaults, resolve Duration setting fields and require the placement stream window after owner confirmation; refresh vendored Harmos sources and verify descriptor parity.
- [x] 3.7.1 Verify that the vendored `proto/harmos/v1/plugin.proto` already matches the requested Harmos revision byte for byte; retain it without a local protocol fork.
- [ ] 3.7.2 Migrate plugin/host call sites and operation policies, stream-setting resolution, required stream window and descriptor parity after dependency adoption.
- [ ] 3.8 Wave 2: extend Harmos client-facing operation declarations/codecs to accept generated protobuf types and descriptors before adoption; preserve bounded decoding, cancellation, backpressure, service paths and streaming envelopes, then pin the published revision.
- [x] 3.8.1 Inspect the pre-client Harmos revision: `ProtobufOperation`, `OperationFactory::protobuf` and descriptor-backed admission provide server registration and internal invocation; identify the managed client's `Portable` requirement and request its replacement. The published answer and remaining admission request are tracked below.
- [x] 3.8.1.1 Obtain generated-message managed client declarations/codecs and call handles from the owner-selected published Harmos revision; prove unary, upload, download, duplex, input finish, concurrent input, cancellation settlement, schema/fingerprint refusal and encoded-size admission against its real host on Windows. Descriptor-backed client memory admission remains separately open in 3.8.1.2; no local codec/projection shim.
- [ ] 3.8.1.2 Adopt the admission bound at the revision that supplies it: Harmos owns the open decoded-memory admission follow-up as an additive `ClientSettings` bound with unchanged client call shapes. Adopt the selected generated client now with wire-message and queue bounds from configuration; this follow-up precedes neither 2.8.3 nor any other adoption task.
- [ ] 3.8.2 Pin the requested published revision in the workspace and migrate every removed facade/SDK consumer to its explicit owner without path overrides or compatibility shims.
- [ ] 3.8.2.3 Keep the existing registry Harmos dependency explicit in the four native host/client packages until their atomic owner cutover; the workspace `harmos` pin belongs to the new product SDK and has no host features. This sequencing preserves buildable commits without a new adapter.
- [x] 3.8.2.1 Replace direct native `Described` derive imports in domain/runtime with the selected published field-reading derive owner, preserving native stored shapes; domain/runtime package Rust checks, the saved-record capture journey and transaction journal journey pass on Windows.
- [x] 3.8.2.2 Replace native transaction identity declarations and the local transaction catalog macro with ordinary implementations, retaining journal identities, version refusal, replay and record collection. Schema-free v1 identities remain ordinary inherent definitions because Harmos `Identified` requires `Described`; native catalog values follow 3.8.2. The focused transaction journal journey and package Rust check pass on Windows.
- [ ] 3.9 Wave 3: move every host operation declaration and handler together, one file per operation in `dokime-operations`, with protocol requests/replies and Rust callee-owned deadlines/retries; define narrow resources without runtime dependencies, register directly, and delete `hosted!` and `Host`/`Serve` forwarding area by area.
- [ ] 3.10 Wave 3: move server logs/trace replay, support bundles, transfers/upload transactions, paging/frontier closure, watches, plot deltas/reduction, diagnostics and redaction into ordinary operations with their meaningful behavior tests.
- [ ] 3.11 Wave 3: put startup `main.rs`, listener configuration and the one client authorization hook in runtime; preserve pairing/key persistence, rotation, settings and shutdown; retain execution manager, effects, plugin entries, transactions, state and cycle transitions under the runtime host owner.

- [ ] 3.12 Host wave, before 3.3: implement Execution as identity/cycle/origin/frozen schemas/evidence/outcome/seal with open/closed/sealed only; freeze TestCase from plugin catalog or Import from script-local catalog without plugin registration/global overwrite. Migrate consumers and recovery so interrupted runs close their executions but import drafts never enter run recovery; remove superseded step-owning execution state.
- [ ] 3.13 Host wave, before 3.3: implement not-evaluated/evaluated/importer-declared outcomes and evaluated-only cycle verdict folding; count declared/unevaluated separately in summaries/reports, reject import criteria and prevent Passed for cycles with no evaluated result. Prove empty, declared-only, unevaluated-only and mixed-cycle cases deterministically.
- [ ] 3.14 Host wave, before 3.3: expose backend execution capabilities for start/rerun/cancel/inspect/export and make desktop show only supported actions; reject cancel outside an active run and import start/rerun. Prove origin/state combinations and migrate affected consumers.
- [ ] 3.15 Host wave, before 3.3: separate fixed-snapshot document rendering into temporary returned output from durable report publication/sealing for runs and imports; migrate report callers and prove render failure publishes nothing. Update affected README/guides and focused lifecycle/schema/recovery proof without implementing later import admission here.

## 4. SDKs

- [ ] 4.1 Wave 4, Rust: make `sdk/rust` the `dokime` product crate with `use dokime::prelude::*;`, plugin authoring and the absorbed client half, generated step/criterion operations and derived `materialize`; product dependencies stop at protocol and compile no host. Align Harmos's `sdk/rust` entry point as `harmos`.
  - [x] 4.1.1 Complete the SDK-owned Rust source and focused proof under `sdk/rust`: composed suite/template/step/criterion/operation/observer/prompt/record/stream values, data-only derives, generated private operations, pure materialization, execution resources, typed host calls and the Harmos managed client layer. Product dependencies stop at protocol/SDK/derive support, proven through the SDK Cargo-tree task. The focused Windows build, lint and every focused journey pass: pinned Harmos reference-host admission/materialization, scoped protocol execution, generated client calls against an in-process server, data construction and execution-resource lifecycle. The required root format check found concurrent changes; the root Rust check reached Clippy but was interrupted when disk fell below the authorized floor. SDK formatting and focused lint are repaired and green; combined root/native and host-system proof remain with integration.
- [x] 4.1.2 Interim package ownership: rename the old facade to `dokime-facade` with library `dokime`; 4.8 migrates its consumers and 5.11 deletes it. The CLI package remains `dokime-cli` and its installed binary remains `dokime`. Include macro crate discovery and the old starter dependency in this rename; focused formatting/checking accepts a package to preserve concurrent ownership. Cargo resolution and the renamed-facade operation journey pass on Windows. Native sidecar packaging and project probing select the interim facade package as well; the Windows Tests producer, both plugin fixture bundles and its 16 compiler-contract tests pass.
  - [ ] 4.1.3 Complete generated service-specific client coverage after protocol task 2.8.3.2: the protocol now declares Backend, Appearance and Plugins services with generated interfaces; remaining client service contracts are still open. Bind available generated messages to discovered `harmos::client::Method` contracts in the meantime; do not claim coverage for absent service contracts.
  - [ ] 4.1.4 Adopt the ready SDK through the owning consumer tasks: CLI remote commands need generated-message calls plus configured pairing/TLS/authentication; desktop needs typed calls/managed watches and explicit runtime embedding; plugins/samples need builder values, data derives, resource-store state and host requirements. Preserve existing client pairing/Target, cycle/execution handles, paging/follow and transfer behavior during adoption, then delete `dokime-client` in 5.7. The migration table is in `docs/rust-sdk.md`.
  - [x] 4.1.7 SDK owner: repair `clippy::let_and_return` in `sdk/rust/src/stream.rs`; the integration merge retains the repair, and the Rust SDK lint and all three whole-workspace Rust selections pass on Windows.
  - [x] 4.1.6 Host-story unblock: seven one-line protocol-drift fixes in `sdk/rust/src/{plugin,data,prompt,record,suite,template}.rs`: use PromptCatalog and default the added Metadata fields; no authoring behavior changed.
  - [x] 4.1.5 Supply the shared cross-language Rust suite in `sdk/rust/examples/thermal.rs` and its expected evidence/host requirements in `docs/rust-sdk.md`. Full Nightly host execution in 4.7 still needs execution-scoped Harmos calls, virtual clock ticks/waits, acknowledged evidence reads/writes, lifecycle cleanup and report sealing; the new host now proves these semantics, and the pinned Harmos runtime supports caller-bound Scope/with_scope.

- [ ] 4.2 Move the plugin, installer and proto commands into `dokime`, add `dokime new --lang`, and delete `crates/cargo-dokime`.

  Source inventory: root Cargo workspace/lock; SDK git pins, scope adapters and authoritative starters; new CLI crate; new-runtime plugin inspection and C# assertion; fixture producer/mise wiring; affected SDK/host/product guides and this design/tasks list.
  - [x] Add the independent `dokime-cli-next`/`dokime-next` command tree: new for all four languages; executable bundle/dev/inspect; proto export; remote plugin Add/List/Install, catalog List/Get, materialization, cycle create/append/seal/rerun, execution list/show/start/stop/watch, prompt next/show/answer and report download.
  - [x] Prove the currently served Rust/Python remote first hour in `remote_first_hour_runs_rust_and_python`; both executions seal Passed and their reports verify. The four-language scaffold/proto story also passes.
  - [x] Prove the managed Rust then remote Python first hour against the combined host. The combined host now serves plugin install/remove, cycle append/seal/rerun and execution start/stop; the extended CLI journey owns their end-to-end proof.
  - [ ] Cutover renames the binary/package, replaces old CLI and cargo-dokime, and moves `dokime installer package`; installer packaging stays with the old tool until then.
- [ ] 4.3 Python: flat public APIs with value composition and explicit close, no required context managers; pure package with the plugin half and the generated client half; rewrite the starter and `dokime new --lang python`.
  - [x] 4.3.1 Replace `sdk/python` PyO3 sources, native stubs, tests, benches and build entries with one pure `dokime` wheel; retain plot/document builders and generate Python messages, stubs and descriptors from `proto/` during builds. SDK-owned replacement and producer/test entries are implemented. Combined integration removes the native Cargo member, refreshes Cargo/uv locks, retires native-only environment dependencies and managed binding declarations, and points versioning at the pure SDK manifest.
  - [x] 4.3.2 Implement remote Harmos serving, suite/template composition, generated unary execution operations, execution-owned instances, streams/observers and typed host-call conveniences against the pinned Harmos Python SDK.
  - [x] 4.3.3 Generate every available `dokime.v1` client service, check descriptor/stub drift, add explicit-close client conveniences and prove them with an in-process fake server; report absent authoritative service definitions. Build-time generation and strict source/runtime descriptor checks cover all 23 services and 262 RPC routes. The in-process generated-service story proves plugin Add/List/Remove, canonical Backend log item/completion envelopes, refusal propagation and explicit close. SDK type checking and fresh-wheel starter/generated-surface proof pass on Windows; no Python-local protocol is introduced.
  - [x] 4.3.4 Rewrite the single Python starter and provide the shared system-story suite; document the exact CLI embedding and host prerequisites. CLI owner: embed `sdk/python/examples/starter/{.gitattributes,.gitignore,main.py,suite.py,pyproject.toml,mise.toml,tests/host.py,tests/test_capture.py}` byte-for-byte, substituting only the project name in `pyproject.toml`; the starter serves a remote endpoint and never starts Dokime. The starter retains plugin `example`, template `example.capture` and its own in-process proof; the shared real-host suite now lives in `sdk/python/examples/thermal/suite.py`, matching Rust's `thermal.cycle` field for field. The host must supply the public context operations, controlled ticks/clock, matching PluginRun origin/scope, expansion, cycle execution, report publication/sealing and reopen.
  - [ ] 4.3.5 Run focused SDK stories, Harmos reference-host conformance, format, Python check, wheel build and fresh-environment starter proof; hand back local commits without pushing. Windows Python checking, SDK type checking, three in-process SDK stories, the starter story, fresh-wheel installation, source-distribution rebuild and all 19 pinned Harmos reference-host rules pass. The native Cargo member and declarations are retired; combined Windows formatting, Python checking (112 tooling tests), generated-surface checking and fresh-wheel starter proof pass. The named Python thermal and friendly-client stories now provide Windows real-host execution and sealed restart proof; native Linux remains integration proof. Nightly conformance requires `HARMOS_CONFORMANCE_HOST` pointing to the exact-revision reference executable; no host binary is bundled in the wheel.
- [ ] 4.4 C#: `sdk/csharp` with both halves, a starter and `dokime new --lang csharp`. Generate message code from the proto files during the project build (`Grpc.Tools`) and delete the committed generated sources; the proto files are the only committed source.
  - [x] C# source: `Dokime.csproj` builds the `Dokime` NuGet preview with all checked-in Dokime protos through Grpc.Tools, composed unary plugin authoring on the unmodified Harmos C# SDK at the owner-selected revision recorded in `design.md`, typed host calls, async resource ownership, plot/document builders and a channel-owning client facade. No generated C# or bin/obj outputs remain tracked.
  - [x] CLI owner: embed exactly `sdk/csharp/starter/{Starter.csproj,Program.cs,mise.toml,.gitignore,packages.lock.json}` verbatim for `dokime new --lang csharp`; no second template copy, repository project reference or bin/obj content. The resulting project builds an executable serving harmos.v1 at HARMOS_ADDRESS. Supply Dokime from a configured feed; it includes the unmodified exact-revision Harmos C# source and the repository package task stages it under sdk/csharp/obj/packages. Build output has adjacent managed assemblies and requires .NET/ASP.NET 10; publish a target-specific self-contained single-file executable before an executable-only bundle.
  - [x] Protocol integration: build every authoritative Dokime proto with Grpc.Tools, including all 23 generated clients; pass SDK check and the in-process generated Backend/Plugins story for authentication, finite logs, plugin Add/List and server errors. Retire stale protocol-owned C# outputs and disambiguate the Plots builder from the generated service.
  - [x] Harmos handoff: de07fc54 exposes descriptor-only Message<T>(); Record<T> adopts it and csharp_frozen_record_schema_decodes_door passes in managed and remote placements.
  - [x] Host owner: the package-consuming C# suite passes managed/remote admission, frozen origins, scoped evidence/prompt/statistics, controlled ticks, cleanup and sealed reopen through the shared host stories; retained Door schema decoding now passes as well. Native Linux and hosted proof remain separate.
  - [x] Focused Windows proof: native MSBuild clean removed bin/obj compiler and generated protocol outputs; SDK/test builds, NuGet package, package-consuming starter and system suite builds pass. Root `mise run format`, SDK formatting and helper checks pass. Three SDK journeys (including the in-process fake backend), two builder cases, all 19 Harmos conformance rules in both placements, and the managed starter package story pass. Combined integration now passes SDK `check` with all 23 clients and the generated-client story; shared-host system and native Linux proof remain open.
- [ ] 4.5 C++: `sdk/cpp` with both halves, a starter and `dokime new --lang cpp`. Generate message code from the proto files in CMake against the protobuf the build uses and delete the committed generated headers and sources. The generated Rust in `dokime-protocol` stays committed; Python generates when its package is built.
  - [x] 4.5.1 C++ SDK implementation and Windows proof (clean configure/build, five focused stories, 19 Harmos conformance cases, checks and source-archive starter): `sdk/cpp/{CMakeLists.txt,cmake,include,src,tests,tools,mise.toml,pyproject.toml,starter}`, `docs/sdk-cpp.md` and the C++ README claims. Plugin composition uses exact Harmos revision recorded in `design.md`; generation consumes every authoritative `proto/dokime/v1/*.proto` with the build's protobuf compiler and gRPC plugin. No generated C++ source remains committed.
  - [x] 4.5.2 CLI handoff: embed only `sdk/cpp/starter/{CMakeLists.txt,CMakePresets.json,main.cpp,mise.toml,vcpkg.json,.gitignore}` as the single C++ scaffold source. Populate `configurePresets[0].cacheVariables` in the generated `CMakePresets.json` with `DOKIME_SDK_URL` for the published `dokime-cpp-<version>.tar.gz` and `DOKIME_SDK_SHA256` from its `.tar.gz.sha256` sidecar. CMake uses `FetchContent` with that URL/hash and `SOURCE_SUBDIR sdk/cpp`; the vcpkg manifest supplies gRPC, protobuf and OpenSSL. The resulting `dokime-plugin` executable uses `HARMOS_ADDRESS` for managed placement or `--address HOST:PORT` for remote placement. This worker does not edit the CLI.
  - [x] 4.5.3 Protocol integration: CMake generates all 23 client services from authoritative protos; the in-process story uses generated Backend and Plugins services for identity/authentication, canonical log completion, plugin Add/List and server refusals. Retire the test-only service and generation target, adopt PromptCatalog, and enable MSVC big objects for the complete generated Execution service. The build, five focused stories and SDK check pass on Windows using exact pinned Harmos source and read-only native dependency imports.
  - [x] 4.5.4 Protocol generator handoff outside C++ ownership: retire C++ output in `crates/dokime-protocol/{examples/generate.rs,tests/support/protocol.rs,tests/integration/generated_code_matches_proto.rs}`. Combined integration removes the C++ and C# generator outputs; the example and drift story retain Rust output only. SDK builds now own message and service generation.
  - [ ] 4.5.5 Host handoff: `sdk/cpp/tests/system/main.cpp` is the C++ shared-system suite. `docs/sdk-cpp.md#shared-system-story` specifies the template, frozen origin, prompt answer, acknowledged record, controlled tick, stream statistics, cleanup and expected evidence. The named new-host `cpp_plugin_matches_rust_thermal_evidence` story now runs both placements on Windows and compares exact sealed evidence with Rust. Nightly runner integration, native Linux and vcpkg-provider validation remain integration proof.
- [ ] 4.6 Migrate the host telemetry and OBC plugins and the samples.
- [ ] 4.7 Add `dokime plugin bundle` for any executable and build each SDK's sample plugin in parallel in CI, running one shared story against all of them.
- [ ] 4.8 Wave 4: redirect CLI hosting and desktop embedding from the server/facade to the runtime library; keep remote commands on the SDK client half and migrate host/distribution facade helpers to their owners.
  - [x] The additive CLI hosts the new listener from configuration with an explicit endpoint and uses the Rust SDK for remote commands; desktop embedding, old-consumer deletion and final crate renaming remain with cutover.
- [ ] 4.9 Wave 4: replace server/desktop build-script facade imports with checked-in protocol output and definitions-only settings; remove build-time host compilation.

## 5. Removal

- [ ] 5.1 Delete `<template>.run` and `.signal`, the signal and evidence-page protocol, the plugin walker and their transport.
- [ ] 5.2 Delete the execution hand-off layer and rename what remains as the execution manager.
- [ ] 5.3 Delete embedded plugins, the PyO3 SDK, the Python backend, stubs and wheel tasks.
  - [x] Remove the Python variant of `plugin::remote_scripts`; The designated new-host replacement is `python_client_runs_thermal_suite_over_grpc` (present in the new-host system target). CLI variants remain.
  - [x] Remove `plugin::interruptions_python`; The designated new-host replacement is `process_interruption` (present in the new-host system target).
  - [x] Remove the Python variants of `plugin::disconnections::disconnections`; client disconnection during a run has no new-host replacement story yet. CLI variants remain.
  - [x] Remove `matrix::python_runs_matrix`; The designated new-host replacement is `python_client_runs_thermal_suite_over_grpc` (present in the new-host system target). The desktop matrix remains.
  - [x] Integrate the designated new-host replacement stories `python_client_runs_thermal_suite_over_grpc` and `process_interruption`; both pass in each complete Windows host run.
  - [ ] Add a new-host client-disconnection-during-run story proving retained admission and independent client survival.
  - [ ] Add a new-host partial-capture interruption story proving saved partial evidence and no completed drain claim; the deleted Python-only helper has no equivalent replacement story yet.
- [ ] 5.4 Delete leases, sample and record delivery, replay and restart recovery during an execution.
- [ ] 5.5 Remove or rework the tests tied to the old loop; update the README, guides and authoring guidance.
- [ ] 5.6 Delete the template, step, criterion, suite, catalog and plugin macros and their tests; keep derives for messages, rows, records, parameters and settings.
- [ ] 5.7 Wave 5: delete the separate `dokime-client` crate after its consumers use the SDK client half; remove host-enabling SDK features and prove both plugin/client dependency trees stop at protocol within the product graph.
- [ ] 5.8 Wave 5: delete `dokime-server`, per-service adapters, superseded generation/build dependencies and dead wire conversions after all behavior and consumers move.
- [ ] 5.9 Wave 5: dissolve `dokime-domain`; migrate definitions to protocol messages, admission/expansion/expression rules to their operations and persisted shapes beside storage, migrating all consumers and deleting duplicate models and superseded guidance.
- [ ] 5.10 Wave 5: remove the operations contract-only surface and `runtime` feature once no consumer needs them; preserve Harmos built-in declaration/handler co-location and enforce protocol → storage → operations → runtime → CLI/desktop dependencies without reverse edges.
- [x] 5.11 Owner amendment (2026-11-13): delete `dokime-tracing` with the retired loop; current host stories own their tracing support. Reconcile manifests, examples and guides with the remaining crates.

## 6. Final Validation

- [ ] 6.1 Run focused operations, runtime and desktop tests for plugin entries, materialization, execution order, retries, sealing, streams and criteria; verify protobuf handler routing, retained server behavior and host-free SDK boundaries.
- [ ] 6.2 Run the system stories with a managed plugin and with a Python script that serves and drives its own suite.
- [ ] 6.3 Run the combined-tree Rust and settings/plot checks after all source handoffs; repair failures and retain native platform limitations.
- [x] 6.3.1 Rerun Python checking after the benchmark owner repairs import ordering in `tools/python/tests/integration/test_desktop_bench.py`; the Windows check passes with 110 tooling tests. That path remains outside this worker's ownership.
- [x] 6.3.2 Clear the earlier workstation interruption with Windows whole-tree Rust checking and the Tests artifact build after restoring free space above the cutoff; three Rust selections, cargo-machete and the archive's 16 compile-contract tests pass. The first archive attempt's allocation failures are repaired by using one Cargo job. Final feature validation in 6.3 and native Linux proof remain open.
- [x] 6.3.3 Rerun workspace Rust checking after the SDK stream repair: all three selections and cargo-machete pass on Windows. The rebuilt Tests producer archives 53 binaries, exports all language fixtures and passes all 16 compile contracts. Native Linux and hosted proof remain separate.

## Workflow Follow-Up

- The new-host source-policy audit has no findings in its new crates or SDK fixes; root manifest skeletons, generated protocol output, other changes' task policy, stale generated guidance and the missing CONTRIBUTING.md remain integration-owner repairs outside this worktree's additive ownership.

- Execution worker owns frozen execution definitions/persistence, outcome folding, capability projections and reports; envelope contents stay here and sending stays with the plugin worker. Sequence the native materialization result and connected Description digest/process identity before replacing snapshot consumers. SDK, benchmark and plugin-entry paths are excluded.


- Tell private plugin owners the migration once a published Nightly carries the change.
- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
- SDK transport conformance runs in Nightly; the shared execution-host system target selects thirteen Rust/Python stories on pull requests and all fifteen in Nightly on Linux and Windows.
- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- This change owns the execution model before manager construction; imported-executions depends on it and follows the SDK wave and storage-interface for admission, draft publication and SDK/report integration.
- Start `storage-interface` only after wave 5: benchmark before/after with multi-day data, extend Harmos storage first, then extract `dokime-storage`; no storage extension or extraction in the gRPC waves.
- Run only focused tasks until combined source closure; final native proof remains separate.

## Combined Tree Integration

Source inventory: specification merge and validation under `openspec/`, Python
SDK export lists in `sdk/python/python/dokime/{plot.py,v1/__init__.py}`, shared
`.github/workflows/{publish,verify}.yml`, `docs/tooling.md`, and the existing
`integration-plots` benchmark and its notes in `../plot-pipeline/tasks.md`.

- [x] Merge `feat/gp-organization-pack`, retaining both sides' specification evidence and completed implementation tasks.
- [x] Repair strict OpenSpec validation and pass all 80 specifications and changes.
- [x] Run whole-tree formatting; repair Python export ordering and pass Python checking (107 tooling tests).
- [x] Pass TypeScript and repository checking, and protocol drift checking (three focused protocol tests).
- [x] Pass focused release tooling (35 tests) and workflow routing (12 tests); honor `build-run` for Windows Rust downloads and refresh Harmos alignment guidance.
- [x] Repeat release staging from clean output with both platforms' existing dev packages (17 upload entries); preserve the earlier output. Current-source packaging and release-profile/hosted proof remain separate.
- [x] Pass the combined Rust check: dev/test, desktop and bench-profile Clippy selections and cargo-machete. The unreachable `tokio::process::Command::new` configuration warning remains nonfatal.
- [x] Complete one bounded time-series profiling pass and record its findings under plot-pipeline benchmark notes without changing budgets or thresholds. All four native comparisons pass; the live increase remains measured, and no plain cause was established.

## Repository Root and Workflow Repairs

- [x] Move explicit tool configuration to `tools/config/`, remove unused root About and Typos configuration, and reconcile managed ownership.
- [x] Move contributor/security guides, environment example and screenshots under `docs/`; update readers, links, snippets and Markdown policy.
- [x] Generate notices under `target/notices/` and preserve distribution, installer and wheel notice contents.
- [x] Repair compiler cache save inputs and preserve job-level Setup cache settings.
- [x] Run repository/Python checks and formatting; exercise notices generation/staging through focused tooling tests.
- [x] Rerun the combined repository/Python entry points after the concurrent organization edits are formatted.
- [x] Complete a local artifact release staging rehearsal: clean-output `--profile dev --target all --stage-only` passed with both platforms' existing packages and 17 upload entries. Native executables identify the rehearsed source revision; full release-profile and current-source packaging proof remain separate.

## Desktop and Repository Integration

- [x] Reconcile the pending privacy guide formatter noise, merge the organization-pack specification branch and update moved-path references.
- [x] Regenerate the runtime report, bundled brand resources and desktop icons after the default appearance change.
- [x] Publish Product compiler records only after successful compilation, retaining failed output separately; prove replacement and failure behavior through tooling integration tests.
- [x] Pass combined formatting, repository, Python, TypeScript and Rust checks and the Tests artifact build.
- [x] Rerun plot and settings TypeScript unit/integration tests, shell organization tests and the focused runtime captured-head test.

## Hosted Failure Repair and Policy Adoption

- [x] Repair archived protocol compiler inputs and prove both protocol tests with `TEST_PROFILE` selecting transferred fixtures.
- [x] Align remote-operation, plot-watch and terminal journeys with the agreed contracts; pass their focused tests.
- [x] Repair native plugin counter collection and store transitions, and keep plot statistics progressing during redraws; pass the focused live-plot story.
- [x] Replace the organization-generation radius constant with an authored fixture override and pass the focused Python test.
- [x] Update the density system journey to drive the appearance combobox; pass the TypeScript checks.
- [x] Trim company policy while retaining project-owned tooling configuration.
- [x] Pass formatting, repository/Python/Rust checks, Tests Build and the focused repaired tests.
- [ ] Validate affected packaged system stories in hosted CI; no local focused packaged runner is documented. Linux proof remains hosted; no push is authorized.

## PR 411 Python Integration Repair

- [x] Reproduce the hosted JSON decode failure with the workflow environment and transferred Windows wheel; identify the expected `--features` rejection diagnostic.
- [x] Supply Cargo metadata and native compiler fixtures in the archive test; verify both platform selections and transferred files without removing collector coverage.
- [x] Capture and assert the expected unfocused `--features` rejection in the dispatcher test.
- [x] Pass the repaired Windows Python integration lane (94 tooling and 275 SDK tests), formatting, Python check and repository check; retain the repair in a local commit without pushing. Native Linux proof remains hosted.
- [x] Read both failed TypeScript System jobs and their retained traces; repair pairing origins, diagnostics filenames, installed-pack identity, update comboboxes, stale recovery controls and two-desktop startup/navigation. Restore Product-generated clients in the artifact-consuming system task.
- [x] Pass the focused Windows configuration, installed-pack, owned-backend and two-desktop stories, combined formatting and Python/repository/TypeScript checks; retain the repairs locally without pushing. Use the existing Chrome override for the report browser on this workstation. Native Linux and hosted candidate reruns remain unverified.
- [x] Read the completed head run's other failure, `CI / Required`; confirm it reports the failed test matrix and preserve its gate.

## PR 411 Plot Transport Repair

- [x] Fit contour grids by exact whole-cell merging with actual returned resolution; fit complete GRID_CHART frames and TRAJECTORY timestamp columns without changing budgets. Source inventory: runtime plot payload/preview owners and server plot fitting/wire owners, their focused integration journey, GUI contour geometry/live-data notifications and their existing plot journey, `e2e/native-desktop.mjs`, `mise.toml`, tooling/product guides and plot-kinds requirements. Pass backend proof and the existing 46-test GUI journey, including nonuniform-edge cell/band/mask geometry.
- [x] Deduplicate background refresh notifications until the plot recovers; pass the existing GUI plot integration journey including repeated errors and recovery (46 tests).
- [x] Honor the existing GUI integration task's `--select` with `--prepared`, retaining the normal regenerate/check path and proving the focused selection through that task; document the existing cycle journey filter.
- [x] Pass the deterministic focused gRPC integration journey for dense contour means, complete playback frames, trajectory admission, atomic shared watch budgets and actionable irreducible errors; pass the existing telemetry/plot-watch story and the focused mean/min/max/gap reducer unit test.
- [x] Pass the focused desktop execution story containing scene cycle with no new error notifications using the current plot implementation; reopen its three sealed execution reports. Compare semantic error-record identities over the complete history after the Library's deliberate invalid-input checks; the nonuniform-edge renderer is covered.
- [x] Repair the existing owned report-browser fixture when Chrome stays alive after DevTools close: retain closed-tab checks and the existing deadline, terminate only its owned disposable child and await physical exit. Startup and cycle cleanup pass on Windows.
- [x] Pass format, Rust/Python/TypeScript and repository checks and the Tests artifact build; preserve the benchmark worker's paths and report native Linux proof separately. The archive retry uses one Cargo job after allocation failures; its 16 compile-contract tests pass. Windows proof only; Linux and hosted reruns remain unverified.

## Protocol and SDK Branch Convergence

- [x] Merge the protocol, Python, C# and C++ branches in order, with one merge commit each; resolve service declarations in authoritative protos and regenerate Rust output.
- [x] Retire the native Python Cargo member, environment dependencies and binding declarations, and stale C#/C++ protocol generation/drift ownership.
- [x] Generate all authoritative client services in Python, C# and C++; migrate canonical types and PromptCatalog, and prove plugin Add then List in each SDK's in-process client story.
- [x] Merge the current integration branch, regenerate and run final focused proof: format, Rust/Python/repository checks, protocol stories, each SDK build/check/stories and Tests artifact build. The prior regeneration, eight protocol tests and Python/C#/C++ build/check/story proof retain unchanged protocol and SDK inputs. The repaired combined tree passes whole-workspace Rust, Python and repository checks, formatting, the Tests producer with 47 binaries and 16 compile contracts, and five external-consumer Rust SDK journeys on Windows. Native Linux and real Dokime-host SDK system proof remain unverified.


## Client-Driven New Host Serving

### Query and Content Service Completion

Source inventory (data-service worker; continuation adds Content, Notes, Documents/Reports and advanced plot variants, with Query and Media held): owned Query, Evidence, Telemetry, Plots, Documents, Reports, Content, Media and Notes service files; their operation files and client stories; additive storage and Host methods; alphabetical registration lines; required protocol additions and generated bindings; this inventory and implementation decisions. Old crates remain read-only.

- [x] Serve Evidence schema lookup, filtered cursor pages including logs, and event-driven watches with real-process client proof.
- [x] Retire Evidence selector variants for sessions, planned cases and opaque attempt IDs because the new model owns execution/cycle scopes and frozen step/criterion identifiers. Keep shared proto fields while other consumers use them; no RPC is retired.
- [x] Serve bounded Telemetry pages and watches, including Run envelopes, with real-process client proof.
- [x] Serve base paths for the ten landed plot kinds through QuerySnapshots/Watch and their Run envelopes; use storage reduction, explicit resolution and a counted two-million-sample, forty-plot story.
- [x] Add isolated cancellable storage read sessions for blocking plot workers, row filters, scale/offset/detail projections and source-time stream/record joins without loading whole streams.
- [x] Serve grouped time series, histograms and boxes, logarithmic contour bins, XY sigma intervals, grid highlights, bounded record markers and separate summary windows; prove the variants through `client_queries_advanced_plots`.
- [x] Serve bounded Raw/LTTB time-series windows, encoded-budget bucket coarsening and unary plugin analyses with typed defaults, bounded raw input and output validation; prove endpoint retention, batch fitting and malformed-output refusal through the advanced plot story.
- [ ] Complete incremental bucket reads and live deltas, higher-order category aggregates, non-category aggregate plots and derived analysis-parameter defaults. QuerySnapshots and Watch are routed; unsupported variants refuse with corrective actions and Watch still emits snapshots. Retired RPCs: none.
- [x] Route Documents CaptureCurrentDocument/UpdatePlotSettings and all Run envelopes, persist plot preferences, stream structure/table/summary/plot bindings, and bind document plot requests to a captured execution frontier.
- [x] Route all Reports direct/Run methods: fixed-snapshot materialization, explicit execution publication and execution/cycle HTML downloads; prove temporary download, publication independence and bounded plot data through `client_renders_fixed_documents`.
- [x] Expand repeated sections in bounded pages with stable identities, inherit filters in plot and summary queries, serve cycle-scoped default capture, evaluate record/scalar summary expressions and exact percentiles, and render masked contour bands against nonuniform edges.
- [ ] Complete evidence-card filter evaluation, cross-execution/higher-order category and artifact expressions, and complete trajectory/grid report playback. These remaining variants are not covered by the served-route checks.
- [x] Export `content_fixture` through the shared Tests fixture producer; the story consumes its archived path or `DOKIME_CONTENT_PLUGIN`.
- [ ] Add the client-driven live execution-file producer/follow journey when the producer group connects `register_execution_file`, `append_content` and `finish_execution_file`; the present real-process journey proves saved files, windows, identity-bound resume and replacement.
- [x] Serve Content artifact uploads/downloads, evidence archives and execution-file list/window/download/follow with durable chunk storage and client proof; unary plugin artifacts enforce sequences, replay fingerprints, explicit terminal, abort and replacement.
- [x] Serve Notes create/edit/list and Run envelopes, idempotent mutations, ownership and cursor validation, restart durability and sealed-owner refusal.
- [x] Pass Windows continuation proof: format; each of the four new stories twice by name; every host system story without skips; three new-crate check builds; all eight protocol stories including generation drift; repository and all three package language checks. The focused report integrity test also passes, including map persistence and tamper rejection.
- [x] Pass combined integration proof on Windows; native Linux and hosted proof remain with integration. The existing `clippy.toml:14` unreachable-method warning remains; checking exits successfully. No push is authorized.

Source inventory: `crates/dokime-runtime-next/src/{services,bin}/`, its library/manifest and client-driven `tests/system/` plus test support; the single `Host::pending_prompts` accessor in `src/host.rs` and public visibility for the existing console-free `src/process/` constructor; `crates/dokime-operations-next/src/client/`, its library/error/manifest; additive fields in `proto/dokime/v1/{client_execution,client_types}.proto` and regenerated `crates/dokime-protocol/src/{v1.rs,descriptors.bin,sources.rs}`; Cargo lock, the execution-host guide and its README link. Existing runtime execution/dispatch/stream sources remain outside this serving slice; the data-service inventory above owns storage additions.
Execution service worktree inventory (`feat/gp-svc-execution`): owned service files
`execution.rs`, `planning.rs`, `observation.rs`, `prompts.rs` and new `history.rs`;
their operation declarations/handlers and client stories; additive host/storage
methods and alphabetical registration lines. Old crates stay unchanged.

- [x] Complete Prompts catalog, pending paging, retained answers and all Run methods; prove the real-process operator journey.
- [x] Complete Planning preview, registration and materialization with ordered bounded expansion; prove the real-process planning journey.
- [x] Complete Execution lifecycle, controls, listing/detail and case reads; prove the real-process lifecycle journey and matching Query/detail projections for explicit origins.
- [x] Complete Observation runtime/progress watches and traceability reads; prove the real-process observation journey.
- [x] Complete History saved cycles/executions and checkpoint reads; prove the real-process history journey and complete filtered cycle counts.
- [x] Pass format, owned stories twice by exact name, every existing host story, three new-crate check builds, all three scoped Rust checks, eight protocol checks and repository checking on Windows; commit locally without pushing. Every story in the final host target passes with no skips; the parameter-expansion unit proof passes. Native Linux and combined integration proof remain with the orchestrator.

Source inventory: `crates/dokime-runtime-next/src/{services,bin}/`, its library/manifest and client-driven `tests/system/` plus test support; the single `Host::pending_prompts` accessor in `src/host.rs` and public visibility for the existing console-free `src/process/` constructor; `crates/dokime-operations-next/src/client/`, its library/error/manifest; additive fields in `proto/dokime/v1/{client_execution,client_types}.proto` and regenerated `crates/dokime-protocol/src/{v1.rs,descriptors.bin,sources.rs}`; Cargo lock, the execution-host guide and its README link. Existing execution/dispatch/streams/storage source stays with the host-feature owner.

- [x] Implement the real-process Rust SDK journey `client_runs_thermal_suite_over_grpc`, including URL admission, catalog, materialization, execution, progress, prompt, evidence, samples, report and sealed restart reads.
- [x] Implement thin generated servers for Plugins, Catalog, Planning, Execution, Observation, Prompts, Query, Evidence, Telemetry, Reports, Health and Backend, with one declaration/handler file per served operation and named refusals for all other methods.
- [x] Implement the configured listener with explicit close and `dokime-host-next` process entry point; keep new host dependencies independent of old crates.
- [x] Add only required additive wire fields and regenerate authoritative Rust bindings.
- [x] Run the same real-host journey with the present pure Python SDK's generated clients.
- [x] Pass focused Windows proof: whole-tree format, `client_runs_thermal_suite_over_grpc` three consecutive runs, `python_client_runs_thermal_suite_over_grpc`, both existing host stories, three new-crate check builds, operations package language checking and protocol generation drift. Python evidence decoding loads the frozen descriptor dependency graph before decoding exact payloads.
- [x] Complete `mise run check -- --language rust --package dokime-runtime-next`: the language worker repaired `clippy::let_and_return` in `sdk/rust/src/stream.rs` and both the SDK lint and new-runtime package check pass on Windows. Native Linux and hosted validation remain integration proof.

- [ ] Remaining client refusals and unfinished variants (all services are registered through the shared authenticated route composer):
  - [ ] Query: held for the owner's client API overlap decision; Aggregate, RunAggregate, AggregateSamples, RunAggregateSamples, ListCycles, RunListCycles, RunListExecutions, ListRecords, RunListRecords, ListSamples and RunListSamples remain named refusals. ListExecutions is served with the shared execution case/archive presentation.
  - [ ] Media: held for the owner's API decision; AudioRecording, RunAudioRecording, CameraRecording, RunCameraRecording, Photo, RunPhoto, ScreenRecording, RunScreenRecording, Screenshot and RunScreenshot remain named refusals.
  - [ ] Content: DownloadAppearanceAsset and RunDownloadAppearanceAsset remain named refusals. Every artifact, evidence archive and execution-file method is served through direct and Run transports.
  - [ ] Plots: every RPC is routed; incremental bucket reads/live deltas, higher-order category aggregates, non-category aggregate plots and derived analysis-parameter defaults remain unfinished as listed above.
  - [ ] Documents: every RPC is routed; evidence-card filters, cross-execution/higher-order category and artifact expressions, and complete trajectory/grid report playback remain unfinished as listed above.
  - [x] Evidence, Telemetry, Reports and Notes: all direct and Run methods are served; no named refusals remain.
  - [x] Execution, Planning, Observation, Prompts, History and administration: all direct and Run methods are served; no named refusals remain.

## New Host Branch Convergence

Source inventory: both local host branches; new storage, operations and runtime crates;
additive client proto fields and regenerated Rust protocol output; Rust system stories
and shared test settings; execution-host and tooling guides; the existing SDK execution
fixture. Other SDK language sources remain with their owner.

- [x] Merge the services and feature branches, retaining both module trees, stories and contracts.
- [x] Expose execution capabilities, attributed outcomes and separate counts through detail/list reads; retain plugin state and interrupted evidence after restart.
- [x] Add explicit report publication, read published bytes without publishing during rendering, and prove the lifecycle over the wire.
- [x] Extend the Rust client story with two executions in one cycle and isolated record reads; prove interrupted reads over the same services.
- [x] Give every system story one shared support-owned hang guard with no literal story deadlines.
- [x] Run format, every host story twice, three focused crate check builds and repository checking; verify locked offline metadata on the committed head. Windows passes: every host story in each final run, all three check builds, formatting, repository checking and locked offline metadata. The missing Python wheel prerequisite is repaired through SDK build/install tasks; no other language SDK source is edited. Native Linux and hosted proof remain with integration.

## Cross-Language New Host Proof

Source inventory: `sdk/python/{src/dokime/{__init__,authoring,client}.py,examples/thermal}`, `sdk/csharp/{Dokime/Plugin.cs,tests/SystemPlugin}`, `sdk/cpp/{src/{plugin,context}.cpp,tests/{CMakeLists.txt,system,integration/author_materializes_suite.cpp}}`, `sdk/rust/src/{client*,stream.rs}`, the new-host `tests/{support,system}` stories and SDK/execution-host guidance.

- [x] Write one named thermal evidence-equivalence story per SDK against the Rust reference, with controlled ticks, prompt events, lifecycle completion, sealed evidence comparison and restart reads.
- [x] Align the Python, C# and C++ thermal suites and generated operation naming; preserve source time and exact record/sample payloads.
- [x] Add thin Python and Rust client conveniences and rewrite both real-process client stories around materialized cases and cycle watches.
- [x] Pass each cross-language story twice, every existing host story, whole-tree format and each affected SDK check on Windows. Python remote, C# remote/managed and C++ remote/managed all match Rust's sealed evidence; every selected SDK story passes, as do Python binding/fresh-wheel, C# check, C++ check, Rust SDK lint and new-runtime package checking. Native Linux and hosted proof remain separate. The retained-record-schema assertion below now passes after the Harmos pin update.

- [x] Harmos assertion `csharp_frozen_record_schema_decodes_door` passes in both placements with descriptor-only Message<T>() at de07fc54.

- [x] CLI owner: embed the eight authoritative pure Python starter files listed in 4.3.4, substituting only the project name in pyproject.toml; remove the three retired starter/plugin includes and native-wheel hint. The CLI story proves source-byte parity and existing-destination preservation; whole-workspace Rust checking and the Tests producer pass.
- [x] Rust SDK owner: replace the `v1::Prompts` construction in `sdk/rust/src/plugin.rs` with canonical `v1::PromptCatalog`; default the protocol's added Metadata fields in the six SDK initializers. Whole-workspace Rust checking and every external-consumer SDK journey pass against the complete client protos.
- [x] Preserve native test inputs removed with the Python SDK: relocate the unchanged negotiation fixture to GUI test fixtures, and relocate the independent Polars Arrow verifier to tooling with its pinned workspace test dependency. The Tests archive, its 16 compile contracts, the GUI negotiation story and the runtime Arrow export story pass.
- [x] Integrate both finished speed-tooling branches, retaining story/package/changed-path selection, batched compilation, selective extraction, desktop freshness, bundled development/test LLD, cache overrides and timings. Resolve the selector/linker environment and duplicate timing flags together; 22 build-dispatch, seven selector and six schedule tooling tests pass. Keep usage and conservative concurrency advice in the tooling guide; condensed measurements live with tooling-streamline.
- [x] Merge the completed integration numeric-entry and split desktop-system-lane repair, retaining both tooling changes and both sets of guidance. Preserve the fast-forward handoff to the current integration branch; no product execution-loop semantics are changed by this repair.
- [x] Prove the combined repair and speed-tooling candidate on Windows: format, repository/Rust/Python checks, Tests artifact build, all 125 tooling tests, CLI Python scaffold, relocated GUI/Arrow fixture stories, the admit_bundle_metadata packaging selector and locked offline Cargo metadata pass. The SDK's journeys run through its separate consumer workspace, not the repository Tests archive. Native Linux and real Dokime-host SDK proof remain separate.
- [ ] Host/client owner: migrate the old GUI acceptance matrix's removed Python client API and deleted execution-matrix driver together. This runtime-only dependency cannot be repaired by relocating the script; old execution-loop semantics are outside the bounded build/reference repair. No acceptance lane was run or claimed here.


## Execution Host Integration Handoff

Source inventory: the language and integration merges; new-host system support;
root and SDK build tasks, fixture exports and hosted test matrices; Rust consumer
manifest versions; protocol regeneration; tooling, execution-host and SDK guidance.

- [x] Merge the language branch and current integration branch, preserving every host story and regenerating protocol output through its task.
- [x] Keep all three new crates in the workspace, retain SDK dependency versions, share the story hang guard and expose package/story/changed-path selection.
- [x] Build shared language fixtures through SDK tasks, export portable fixture bytes with Tests and select the new-host system target separately on pull requests and Nightly for Linux and Windows.
- [x] Pass Windows format, repository/Rust/Python checks (125 tooling tests), the complete Tests producer (53 archived binaries, 16 compile contracts), eight protocol drift/routing stories, all four SDK checks and locked offline metadata. Every execution-host story passes twice with no skips, including managed/remote C# and C++ equivalence. No old-host check failures remain in this candidate. Native Linux and hosted runs remain unverified; the C# record-only descriptor assertion is now enabled and passes at de07fc54.

- [x] Refresh the generated C# fixture lock when republishing the locally rebuilt development SDK. Reproduce NU1403 across committed checkpoints and pass the same publisher with native restore graph re-evaluation; preserve package hash validation and keep the lock in the fixture output. The complete producer and both complete host runs pass after this repair.

## PR 411 Pure Python Release Packaging

- [x] Replace native Python release packaging with one pure wheel and source distribution, shared across Release and Nightly, with generated protocol validation and one registry upload.
- [x] Pass focused release-layout (15), plugin packaging (2), CI gates (25) and supply tooling (16) tests, Python build/package, format, Python and repository checks (128 tooling tests in the final Python check). Local dry publication reuses the integration checkout's native packages, stages the shared Python pair once, and stops at the unchanged git-dependency rejection. Keep commits local; Linux execution proof remains hosted.

## Administration Services

Source inventory: assigned Plugins, Catalog, Settings, Secrets, Backend, Appearance, Runtime, Remote and Health service files; matching operation and story files; additive host/storage support and shared registration lists. Old crates and other service files remain untouched. The original Rust client story loses its superseded Reload refusal assertion; the new plugin-entry journey owns successful reload and validation proof.

- [x] Runtime GetIdentity/GetHealth and their Run transports; real-process restart identity journey.
- [x] Appearance GetAppearance and RunGetAppearance: built-in identity and admitted typed appearance contributions; real-process read journey.
- [x] Remote four direct methods and four Run transports: authenticated TLS listener, pairing files, live key rotation, certificate regeneration and persisted restart state; real-process TLS journey.
- [x] Plugins List/Add/Reload/Remove/ListAdmittedPlugins and all matching Run transports; durable URL placement and preconnection identity, retained failures, explicit reconnect and remote detach.
- [x] Plugins managed installation, verified v1 bundle input, replacement/rollback, enable/disable, GetInstallation/ListInstallations and their Run transports; real-process install/replace/restart/remove journey.
- [x] Catalog six direct methods and six Run methods: schema projections from admitted descriptors, epoch/content-bound cursor paging and the real-process catalog journey.
- [x] Settings eight direct methods and eight Run transports: revision-checked preferences, bootstrap paths, native storage-source administration and on-demand plugin settings (including unavailable plugins).
- [x] Secrets seven direct methods and seven Run transports: protected identifier storage, generation checks, bounded v1 archives and explicit import choices.
- [x] Backend diagnostics ZIP, bounded log following and epoch/scope trace reads; all Run transports. Health has content-bound paging and both transports.
- [x] Required Windows focused proof: whole-tree format; every administration story twice and every host story; check builds and Clippy for all three new crates; repository checking. Every story in the final client selection and host target passes, with no skips. The native credential publication failure is repaired with read-back confirmation before SQLite publication.
- [ ] Integration owner: native Linux and hosted proof, with an available OS credential store for protected secret/TLS writes; no push is authorized here.

## Execution, Administration and CLI Convergence

Source inventory: merge conflicts and additive integration repairs in the new runtime,
operations and storage crates; command-line system stories; SDK/git dependency locks;
shared registration lists; execution-host/tooling guides and this change. Data service
implementation files and old-host source remain with their owners.

- [x] Merge administration and pinned-Harmos/CLI branches locally, preserving every service, story and v1 table.
- [x] Share authenticated local/remote routing for the execution group, including History; combine operation registrations without duplicates and notify runtime observers after durable plugin-entry commits.
- [x] Repair merged enum ownership, preserve precondition versus revision-conflict statuses, match execution admission by described plugin identity and include managed-bundle compatibility metadata.
- [x] Prove the managed starter first hour and command-line append/start/stop/seal/rerun flows against the combined services.
- [x] Pass final format, Python/repository/supply checks and all four SDK checks on Windows. Rebuild and install the pure Python wheel against the pinned Harmos revision before rerunning its fresh-wheel check.
- [x] Pass final Rust language checking and Tests artifact build on Windows. The Tests producer passes after closing the owned starter left by an earlier failed CLI journey; every compile contract passes. Untouched `clippy.toml:14` still emits an unreachable-method warning; the required check returns success.
- [x] Build every host fixture, pass every host story twice and rerun every CLI story against the pinned fixtures on Windows, with no skipped stories. The C# Door assertion passes in both placements; runtime observation wakes after plugin admission.
- [x] Verify locked offline Cargo metadata on the committed source head; retain local commits without pushing. Native Linux and hosted proof remain with the integration owner.

## Data Service Branch Convergence

Source inventory: the local integration merge; shared host routes/state, operation registrations/errors/projections, storage tables and fixtures; authoritative client protos and regenerated protocol output; CLI and remote-access stories; execution-host/tooling guides and this change's design/tasks. Old-host source remains read-only.

- [x] Merge the local integration branch, retain both complete service/story sets and regenerate protocol output through its mise task.
- [x] Register every data service through the shared authenticated route composer, join Query execution presentation, and retain the pinned Harmos generic scope and client settings contracts.
- [x] Extend the CLI journey with a served report read and cover remote data routes under bearer authentication.
- [x] Pass final Windows format, Rust/Python/repository/supply checks, host fixture build, every host story twice without skips, CLI stories, protocol drift, complete Tests artifact build and committed locked/offline metadata. The final complete host runs use `NEXTEST_TEST_THREADS=4`; default parallelism previously exceeded the managed installation operation deadline. Deadlines and project test configuration remain unchanged. The Tests producer also passes its compile contracts. Native Linux and hosted validation remain with integration; no push is authorized.

## Old Loop Deletion and Grammar Continuation

Source inventory: root Cargo manifests/lock and mise tasks; the ten retired loop
crates, packaging, both sidecars; project-owned tooling, workflows and configuration;
maintained guides and SDK examples; product protocol, operations, storage, host,
SDKs and starters for the subsequent grammar slices. Desktop source is excluded.
The owner authorizes deletion before the desktop and face cutovers (2026-11-13).
All work remains local on feat/grpc-plugins under the existing implementing marker.

- [x] Delete the ten old-loop crates and their exclusive consumers, fixtures and configuration.
- [x] Retain current bundle production/inspection without facade dependencies.
- [ ] Re-author the host telemetry plugin on the Rust SDK, including its collectors and controls.
- [ ] Re-author the OBC sidecar on the Rust SDK, including its device operations and suites.
- [ ] Re-author storage/plot benchmarks on the remaining host/storage and restore their Nightly producer and suite selections.
- [ ] Restore the desktop to the workspace when its switch merges.
  - [x] Restore crates/dokime-gui/src-tauri workspace and default membership.
  - [x] Restore native desktop Product build, icons, embedded resources and generated-settings checks, including tests/unit/settings/preferences.test.ts and its real native default exporter. Host startup and Product export pass after the workstation credential cleanup; no fallback defaults are used.
  - [x] Restore frontend production build, route generation and full generated-client type checking with its native preference declarations.
  - [x] Restore every retained frontend unit suite, including console, cycle, document-stream, history, library, navigation, plots and notification history: 181 stories pass with no exclusions. Move the dormant appearance story into the canonical organization workflow; its deleted facade transfer is replaced by the current unavailable-content contract and independent installed-logo selection, while content-source composition remains with its owner.
  - [x] Restore retained frontend integration suites, including document-query, lifecycle, saved-report, responsiveness, plots, registry, status and report-import: 94 stories pass. Remove only shell/contracts, which asserted the deleted facade's generated source inventory; current type checking and native IPC stories own that boundary. Shared appearance context is independent of native clients so standalone report imports stay closed.
  - [x] Restore native Rust acceptance and both TypeScript system story selections on PR and Nightly for Windows and Linux. Restore the SDK-authored desktop benchmark producer, archive and Nightly leaves. Hosted execution remains pending integration delivery.
  - [x] Restore desktop distribution, Windows NSIS and Linux DEB/RPM packaging from verified Product bytes; declare native release assets and signed updater metadata. Restore protected stable Windows signing before updater signatures and publication, consuming the validated review Product without rebuilding. Windows unsigned NSIS packaging passes; Linux and credentialed publication remain hosted proof.
- [ ] Complete command-line parity after host support is available.
  - [x] `operation list`: discover admitted operations; `operator_discovers_operations` compares the whole host catalog.
  - [x] `operation show`: inspect identity, visibility, policy and shapes; `operator_inspects_contract`.
  - [x] `operation call`: invoke typed Public unary operations through the generic client; `operator_calls_operation`.
  - [x] `doctor storage`: probe scratch storage and validate a bounded frozen database copy; `operator_diagnoses_storage` proves original-byte preservation.
  - [ ] `gui`: waits for the desktop to return to the workspace.
  - [x] `version`: report compiled JSON/source/channel metadata; `operator_identifies_build`.
  - [x] Pairing-file connection and access pairing export; `operator_pairs_with_host` proves pinned TLS, backend identity, product reads and revocation.
  - [ ] Installer packaging: waits for desktop packaging restoration.
  - [ ] Plugin development supervision and plugin-specific protocol export: normal startup, admission and entry cleanup pass in `operator_pairs_with_host`; unexpected-exit notification and source export wait for Host Requests.
  - [ ] `execution export`: download a saved evidence archive and files.
  - [ ] `execution run`: drive interactive commands.
  - [x] `plan preview`: validate canonical protobuf JSON selection requests without registration.
  - [x] `plan register`: register canonical protobuf JSON selections and recover identical replies by request key; changed requests conflict.
  - [ ] Planning convenience flags: resolve suite/template shorthand and construct complete selections from CLI flags.
  - [x] `cycle report`: download and verify a cycle report without publication; refuse overwriting an existing file.
- [x] Record promised old-loop journeys without new-host coverage as explicit remaining stories.
  - [ ] Prove earliest inverse retention, owner-qualified confirmations, reverse restore order and failed startup restoration through new-host Effects.
  - [ ] Prove stream/record schema evolution and plugin removal retain decoding on one root.
  - [ ] Prove bounded complete queries, pinned cuts and explicit watch gaps through the final grammar.
  - [ ] Prove typed cross-plugin invocation, cancellation propagation and client authorization through the composed registry.
  - [ ] Restore SDK-authored host telemetry/controls and device simulation journeys with the two sidecar re-authoring tasks.
  - [ ] Restore desktop/installer organization, licensing, signature and update journeys with native packaging.
  - [ ] Retain client-disconnection and partial-capture interruption tasks from removal section 5.3.
- [ ] Complete the owner's focused checks, host/CLI/four SDK stories and Tests build on the committed head.

### Host Requests

- `Plugins.Watch` for the product-owned plugin inventory: the existing library request `{name, at}` and response `{resolved_at, snapshot | change | resync_required}`, ending on host shutdown. `plugin dev` needs unexpected-exit notification; the current local `PluginProcess` exposes start/describe/close but no wait handle, and the library plugin inventory is not composed with product entries.
- `Plugins.Protocol.Get`: request `{name}`; response `{files: [{path, source}]}` for the plugin's authoritative `.proto` source closure. `proto export --plugin ID --out PATH` needs source files; current descriptions expose descriptor sets, not `.proto` sources. Product-wide embedded export already works.
- `Invocations.Invoke`: request `{operation, fingerprint, payload}`; response `{payload}`. The SDK declares this generic service, but this head returns `UNIMPLEMENTED`; Public unary operations with admitted gRPC mappings already work through `operation call`, while Public contracts without a mapping await this host route.

## Remaining Source Policy Proof

- [x] Repair CI after the protocol face split: export declared preferences without
  host startup, preserve startup causes, move Python upload I/O off the event loop,
  include every face in the Python stub catalog, prepare standalone fixture jobs
  without frontend tools, and prove current release-profile desktop bundling.
  Source inventory: the desktop exporter and startup diagnostics; runtime error
  and service composition; Python client and service catalog; setup action,
  build-test workflow and mise preparation; affected generation/tooling tests;
  architecture and tooling guides. Product generation additionally owns canonical
  ICNS chunk ordering in the organization generator and its generated icons and
  integrity inventory, with deterministic tooling coverage. Generated bindings
  and assets change only through mise.
- [x] Run the requested committed-head format, Rust/Python/repository checks,
  generated wheel check, Windows Product and release-profile fixture builds;
  confirm generation leaves a clean tree and no disposable access credentials.

- [x] Remove disposable Windows root credentials through product-owned access/secret removal, wire host/CLI/Python SDK/starter failure teardown, and prove named credentials are unreadable after normal and failed story disposal. Inventory: storage administration, operations secret owner, runtime access/composition/host executable, shared story root support and consumers, Python host support, architecture/execution-host/tooling guides. Normal shutdown retains restart credentials. Each owner journals native slot ownership before writes and retains cleanup intents for staged, active and retired credentials; the disposal journey also removes publication pointers before failed teardown and includes nested and storage-only roots.

- [ ] Resolve project-wide native desktop dependency/source findings when its host switch restores workspace ownership; do not restore deleted loop dependencies to make the dormant manifest parse.
- [ ] Reconcile generated protocol source auditing with generator ownership; keep the protocol drift stories and never hand-format generated bindings.
- [ ] Reconcile the thermal Python suite's dependent schema declarations and the protocol generator's required-error-module finding with source policy; preserve execution order and do not add unused error types.
- [x] Move managed-install cancellation to the cross-platform client_cancels_managed_installation system story, using awaited process events and cancellation settlement. Windows proof passes; native Linux execution remains open.
- [x] Move stored-report byte retention/restart/tamper refusal into the storage integration target without exposing the private database handle.
- [ ] Reconcile the checker-required root contributing entry with the project-owned guide under docs/.
- [ ] Reconcile maintained plot task lists with source policy without losing their benchmark requirements or retained evidence.


## Desktop Host Cutover

Source inventory: `crates/dokime-gui/**`, `docs/gui.md`, desktop sections of
`docs/product-guide.md` and `docs/tooling.md`, and desktop story selection in
tooling. One connection module owns new-host embedding, readiness, shutdown,
remote TLS/pairing and generation cancellation; the SDK root serves native calls.
The frontend data adapter owns shape changes. Existing components retain their
presentation. No old-loop fallback or dual-host switch is part of this cutover.
Additive host operations, if required, are listed here with their handler/story;
existing host services, proto and generated Rust protocol remain with their owner.

- [x] Map every frontend native command/event to the served host operation or a missing operation in `docs/gui.md`.
- [ ] Close Start and Connect with embedded lifecycle, SDK readiness, pinned pairing and joined shutdown/restart proof.
  - [x] Implement the native SDK connection owner and descriptor-derived dispatch; prove embedded readiness, attach/detach ownership, restart identity, incompatible-root preservation, bounded Watch delivery and joined shutdown, and pinned pairing with certificate rotation.
  - [x] Switch the Tauri session and IPC bridge to the SDK owner; remove the old embedded server/client path and prove generation replacement through real Tauri IPC.
  - [ ] Pass the native WebView Start and Connect journey; its system driver still embeds the old CLI/fixtures.
- [ ] Close Catalog and Suites through the new host and preserve parameter/template presentation.
  - [x] Generate canonical ES messages/client calls from the product and exact pinned library descriptors; project suites/templates and typed parameters into the existing Library view models, with precision/list/range proof.
  - [x] Wire the canonical catalog projection into the running shell and adopt bounded host list replies.
  - [ ] Pass the native WebView Library journey with the replacement driver.
- [ ] Close Execution with live progress, observations, prompts and gap resynchronization.
  - [x] Drive the shared thermal fixture through the desktop SDK adapter: admit its catalog, materialize a frozen case, run it through a watched prompt, answer it and observe terminal completion.
  - [x] Switch the WebView planning, execution detail/control, runtime/progress observation and prompt adapters to canonical host calls. Preserve recursive exact values and selection order, register cases with a cycle-scoped idempotency key, project attributed outcomes independently of live state, and gate run actions by host capabilities. Fifteen focused lifecycle stories pass, including binary request/reply round-trips and generation-owned Watch cancellation; six focused operation-status stories pass. Full WebView execution proof remains open with the shared evidence/document adapters and driver.
- [ ] Close live and saved Evidence, Telemetry and Plots.
  - [x] Read captured record payloads and telemetry from the sealed thermal execution through the desktop SDK adapter. Switch the WebView live-record tail and Watch adapter to canonical cursors and per-execution frozen descriptors; prove exact unsigned/double fields, changed schemas of the same record type and generation-owned stream cancellation. Canonical plot requests, analyses and columnar Watch frames now drive the renderer; 18 plot unit stories and 46 plot integration stories pass. The shared thermal story opens a Watch before answering the prompt, drains its terminal response and compares the saved plot with captured samples. Checkpoint consumers and complete WebView proof remain open.
- [ ] Close Documents and Reports with verified native export.
  - [x] Read the thermal execution report through the desktop SDK Watch transport and verify ordered chunks, completion size, digest and terminal response. Native verified delivery now uses the SDK stream and is exercised against thermal report bytes. Canonical incremental document assembly and content/file/upload projections now replace the old frontend calls; four document assembly stories, two document query stories, and canonical artifact/default-document codec stories pass. The thermal story also verifies document completion and the product process epoch. Picker/WebView proof remains open.
- [ ] Close History browsing and saved execution details.
  - [x] Replace the immutable browse/checkpoint adapter, numeric archive/order selections, outcome projection, saved artifact/file messages and frozen-record decoding. Canonical codec stories prove attributed browse results, checkpoint log decoding and note owners/publication. Full WebView proof remains open.
- [ ] Close Settings, Secrets and Appearance with revision conflicts and scope preserved.
  - [x] Replace native application preferences with canonical protobufs and preserve local/backend scope and expected revision; canonical appearance and visible missing-logo reason are wired. Secret catalog/transfer now use canonical metadata, exact uint64 generations and import actions. Plugin settings/cards and storage sources now use canonical catalogs and map values; unserved restart/applied-field facts stay absent. Full WebView proof remains open.
- [ ] Close Plugins install/list/remove and restart availability.
  - [x] Switch inventory, admission and remove/enable/reload requests to canonical plugin entries and installation states; preserve bounded paging and current failure reasons. Seven installation frontend stories pass.
- [ ] Close Remote Access pairing, connection and certificate rotation.
  - [x] Switch remote cards and preference saves to the Access listener resource with its own opaque revision; retain certificate rotation and pinned pairing semantics. TLS/authentication remain host requirements.
- [ ] Close Status, Health and Diagnostics through the library services.
  - [x] Switch status bar identity, traces and diagnostics to library Status resources; verify stable diagnostics ranges/digest and trace List-to-Watch handoff in a native story. Canonical frontend codec stories now cover settings, listener revisions and trace/diagnostics delivery (22 passing). Full WebView proof remains open.
- [ ] Remove all old-loop references from GUI code, manifests, tests, benches and tooling. Integration has deleted the retired crates. The remaining per-journey modules and incompatible request/response shapes are named in `docs/gui.md`, Remaining Journey Adapters. Desktop paths/distribution identity and build facts now have native owners, and tracing uses its maintained independent crate. Desktop update policy and process launching also have current owners. Unsupported plugin release lookup/install refuses before network or file changes; verified bundle installation remains available. Native production support dependencies have been replaced; old native stories/benches and the WebView drivers remain migration work.
  - [x] Replace native test support and its deleted runtime imports with the current embedded host owner, imported content fixtures, generated protobufs and SDK plugin bundles. Native test targets compile with the testing feature. Retire facade-specific negotiation/transcoding tests; retain identity pinning, generation cancellation, adversarial verified delivery, native uploads, installation and completion/refusal/cancellation journeys on the served contracts. Runtime proof and failure-safe credential disposal remain open until the integration cleanup operation is adopted.
  - [x] Replace the desktop benchmark's embedded execution fixture with an SDK stream producer and current-host client operations. Remove the last desktop tooling import of retired runtime report exemptions. The retired crate-name search is empty; all native desktop targets compile with testing. Benchmark execution and WebView journey migration remain open.
  - [x] Add failure-safe native story root ownership and host-operation cleanup for WebView/benchmark roots; select the complete native testing feature in Tests Build. Restore desktop Product frontend/icon generation, embedding and shell exports, plus desktop fixture production and selectable WebView task entry points. Runtime and Product proof remains open; hosted lanes and installers are not yet restored.
  - [x] Replace the retired benchmark runtime with the SDK stream fixture and remove unused facade dependencies. Restore bounded client-input forwarding for plugin bundles and credential cleanup after failed native stories; native integration passes 34 stories. Generate and typecheck the complete frontend from current descriptors. The native WebView drivers now use host/CLI fixtures and Access resources; their runtime proof, retained frontend suites and hosted/installer restoration remain open.
  - [x] Merge integration 7e6854bac, retain its old-loop deletion and restore desktop workspace/default membership. The desktop package builds using direct filesystem dependencies and native appearance/logging owners. Hosted lanes, packaging, old native test/benchmark fixtures and WebView drivers remain open below; the full desktop cutover is not integration-ready.
  - [x] Declare desktop support dependencies directly and own their Cargo.lock entries (owner decision, 2026-11-13). Filesystem publication uses Harmos runtime directly; installed appearance and logging are desktop-owned. Product compilation passes after merging integration 7e6854bac; native test fixtures and desktop tooling still need replacement.
- [ ] Run format, Rust/TypeScript/repository checks, frontend stories, host fixtures, both desktop story lanes, startup proof and clean committed-head locked offline metadata. Record actual failures and remaining proof.
  - [x] Pass Windows format, Rust/TypeScript/Python/repository checks, 37 native unit, 34 native integration, nine native acceptance, 181 frontend unit and 94 frontend integration stories, all available host fixture producers and the complete Tests build. The serial rerun repairs a duplicate C++ producer's PDB lock failure. Native Product and unsigned NSIS packaging pass. Linux, benchmark runtime and credentialed signing/publication remain hosted proof.
  - [ ] Close the Execution WebView's live document card against the repaired host frame contract. Seven other Execution journeys and the complete Connections journey pass. Disposable-root credential removal adds no entries across the final 80 native stories; four entries already present at the verification baseline have no retained root identified and are not deleted without ownership evidence. A globally empty credential-store result is not claimed.
  - [x] Pass twelve focused native host/path stories, seven session acceptance stories, five session/archive integration stories, all 180 frontend unit stories, three canonical secret-transfer stories, frontend/native builds, format and Rust/repository checks on Windows. Manually start the embedded host in a fresh root, connect the shell, navigate Library, read scoped preferences, save Compact density and read it back after reload, and load Secret Storage. Attach to a healthy host while preserving an incompatible local preferences root. Preserve the new store across restart. Correct the status-history/product-epoch readiness mismatch, discover Access/Status methods through the bounded SDK operation catalog, and verify the rebuilt shell connects and renders the listener state with fixed TLS/authentication controls.
  - [x] Move desktop update policy into the shell, remove unreachable facade release-feed installation behind the typed-build Host Request, and use the new host process owner for firewall commands. Three policy/source, two update-trust/selection and three installed-appearance stories pass; the focused GUI Rust check and native build pass.
  - [x] Finish desktop resource generation after old-loop deletion. Real embedded-host preference defaults, protobuf view models, native commands, full frontend type checking, live/report builds and Product generation pass. Product artifacts carry both generated module trees. The owned desktop starts and restarts with preserved data; unsupported preserving recovery stays visibly disabled.
  - [x] Replace both WebView CLI/fixture drivers with the current host, SDK thermal bundle and canonical operations. The Connections story passes on Windows, including two clients, TLS pairing, shared execution/evidence, report reopening, revocation, occupied storage, identity refusal and certificate rotation. Execution passes seven of eight independent journeys; live document rendering remains blocked by the precise Host Request below. No promised journey is removed.

## Host Requests

- [ ] Restore the incremental `Documents.CaptureCurrentDocument` frame contract used by the desktop: declare source cards, summary definitions, bindings (with metadata/identity) and layout before `StructureComplete`; issue the fixed `EvidenceCut` after structure; then settle card results and `CardComplete` with matching counters. The current handler in `crates/dokime-operations-next/src/client/documents_capture_current_document.rs` emits EvidenceCut/Page before structure, emits cards afterward, and omits ordinary literal/summary bindings. The Windows execution WebView reaches Passed and real evidence, then correctly refuses the stream with `evidence cut is misplaced`. Keep the desktop's validation and prove interruption/card-commit behavior against the repaired host.
- [x] Evaluate desktop controls against the available host owners. Media capture and preserving recovery need additional lifecycle/platform ownership; typed connected-host build facts need a public status contract; product discovery needs authoritative RPC bindings. Keep their precise requests below and their controls visibly disabled. Logo/appearance content composition remains with its owner.

- [ ] Serve `Media.Screenshot(MediaScreenshotRequest { scope }) -> ArtifactMetadata`, `Media.Photo(MediaPhotoRequest { scope, input }) -> ArtifactMetadata` and `Media.AudioRecording(MediaAudioRecordingRequest { scope, input, maximum_seconds }) -> Watch<RecordingFrame>` on the new host. `MediaCaptureControls` uses these for Screenshot, Photo and Record Audio in an execution's media card. The current handlers return UNIMPLEMENTED and the host has no capture owner. Keep those controls visibly disabled during the desktop cutover; do not capture on the desktop as a replacement for host capture.
- [ ] Serve appearance content through the library transfer owner's content-source registration, preserving `AppearanceAssetReference { plugin_id, asset_id, sha256 }` and returned metadata, byte ranges and verified completion. `src/lib/rpc/appearance.ts` needs the bytes for connected-host logos; the current `Content.DownloadAppearanceAsset` handler returns UNIMPLEMENTED. The library now supplies content-source registration; its application composition must be completed before enabling provider asset delivery; native installer appearance remains independent.
- [x] Host/storage now owns exclusive storage-root admission in `services::serve`, returning `OccupiedRoot(path)` before database recovery. The desktop maps it to `BackendRunning` and proves refusal, attachment to the existing owner, attached shutdown without stopping that owner, and release/reopen. Destructive recovery remains separately unavailable below.

- [ ] Expose host-owned backup/reset recovery preserving installed bundles, plugin settings, host identity and the secret catalog, with occupied-root refusal and verified backup before destructive action. The connection screen's Back Up and Reset and Diagnostics' Reset Local Storage commands cannot reconstruct this state from the current public SDK. Native recovery refuses before stopping or modifying the root and the control explains the unavailable contract.
- [ ] Expose typed product build facts, including the connected host's artifact target, source revision and channel, through its status/identity owner. Settings > Plugins needs the host target to select a release bundle; local CPU facts cannot identify a remote host. Disable only Check for Updates with a visible reason until this is served; native file installation remains available.

- [ ] Advertise product RPC bindings through `CatalogEntries.List(CatalogEntriesListRequest { kind: OPERATIONS, page }) -> CatalogEntriesListResponse.operations`: product contracts currently omit `DiscoveredOperation.contract.grpc`, so the library cannot report `grpc_available` for the served product routes. Preserve each operation's canonical request/response descriptors, policy and resource binding. Settings > Diagnostics > Storage Source Priority needs discovery for `Settings.ReorderStorageSources` and `Settings.SetStorageSourceEnabled`; those two controls remain disabled with a visible reason. Access/Status services advertise their bindings and stay enabled. Do not invent capability bindings in the desktop.
