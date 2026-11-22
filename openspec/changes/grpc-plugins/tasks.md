## Nightly Regression Repairs

- [x] Reproduce and Retire the Superseded Load-Deadline Failure Story Against the Locked Offline Contract; Its Timeout-Prose Assertion Failed at Line 67 After Offline Projection Removed the Failure. Four Retained Readiness Tests Pass.
- [x] Reproduce and Retire the Superseded Unreachable-Remote Health Story; `crates/dokime-runtime/tests/support/mod.rs:132` Waited for Another Watch Event While Expecting Unavailable After Offline Was Reached Without a Failure. Four Retained Registration, Restart, Removal and Durable-State Tests Pass.
- [x] Keep Embedded Reports Free of Live RPC Dependencies While Preserving Session-Owned Plugin Restart; Complete the Planning Session Mock and Prove Six Focused Desktop Integrations, 41 Neighboring Units and TypeScript Checking.
- [x] Reproduce the C++ SDK Failure Before Configure and Resolve Its Archive Before Entering the SDK Task Directory; All 17 CI Dispatch Tooling Tests Pass.
- [ ] Complete Native C++ Package Consumer Proof With a Compatible MSVC Toolset: Configure Passes, but the Hosted Archive Uses MSVC 19.52/14.52 and This Windows Machine Has 19.44/14.44; Linking Fails on New Standard-Library Symbols Before CTest.
- [x] Complete Format, Repository and Strict OpenSpec Validation.
- [x] Compile Every Workspace Test Target With Desktop Testing Through `mise run test -- --changed Cargo.toml --no-run` (Exit 0).
- [ ] Complete Native Linux Proof for the Nightly Regression Repairs.

## Library State and Ownership

- [x] Refresh Every Library Pin, Native Lock, Vendored Protocol and Tracked Binding to `324ad01b0892b1ca6a3bce4bb9525b22974f606b`.
- [x] Supply Operation Identities to All Four Desktop Benchmark Session Calls and Pass the Focused Benchmark Typecheck Without Building or Running a Benchmark.
- [x] Admit the Generated C++ Plugin-State Resource Through the Existing Private Context Friend List and Compile All Debug Test Targets.
- [x] Prove Registry Restart, Remove and Disable Preserve Externally Owned Listeners and Rust Remote Authoring.
- [x] Compose Durable Plugin State and Prove Restart Readback and the Named Stale-Revision Conflict Through the Product Host.
- [ ] Preserve Entry Identity, Settings and Durable State Across Remote Address Changes; Blocked by Managed-Only Library Replace. Retire Product Delete/Create After the Library Supports Remote Placement Updates.
- [x] Compact Native Recovery Atomically and Prove Bounded Snapshots/Entries and Equivalent Reopened State.
- [x] Expose the Library Collection Directly Through the Product Rust SDK.
- [ ] Replace the Product Scheduler in `sdk/rust/src/stream.rs` With the Library Collection When the Host Plugin Lands; This File Belongs to That Integration Worker.
- [x] Complete the Requested Native Windows Focused Proof and Gates.
- [ ] Complete Native Linux Proof Separately; One Native Platform Does Not Establish the Other.

## Product Contracts

- [x] 2.2 Provide typed host progress, logs, traceability, document parts, record reads and execution/step statistics in `crates/dokime-operations/src/host/`.
- [x] 2.3 Replace JSON catalog sections with protocol messages and embedded protocol-source export.
- [ ] 2.3.1 Add protocol export from a running host, plugin URL and bundle; `commands/proto/export.rs` currently exports embedded sources only. Coordinate bundle inspection and archive closure with plugin-boundary.
- [x] 2.4 Read settings on demand through typed host operations rather than start-time injection.
- [x] 2.5 Implement Rust execution-context typed set/get resources and remove the retired template Own model.
- [x] 2.8 Make protocol messages authoritative and remove the product domain/wire mirrors; generate host service declarations and four-language client roots.
- [x] 2.8.1 Adopt generated frontend settings cards/sections, execution controls, evidence scopes, plot grids/diagnostics/domains and value formats; remove their copying adapters and the unused DocumentCard, buildText and OperationFrame paths.
- [ ] 2.8.2 Resolve direct host HTML versus retained React report rendering ownership, then replace the remaining frontend document/plot serialization families and copying adapters; keep only presentation state composed with generated contracts and prove the complete GUI candidate.
- [x] 2.9 Freeze PluginRun/Import Origin, reserve retired TestCase fields and migrate planning, persistence, SDK invocation envelopes and report/client projections.
- [x] 3.0.6 Switch CLI/desktop to the execution host and remove retired domain/client/server crates and candidate package suffixes.
- [x] 3.2 Materialize case-owned steps, criteria and streams for expanded parameters, retaining provider digest and process identity.
- [x] 3.3 Implement host execution ordering, retries, interactive commands, active-only cancellation and joined cleanup in `execution_manager.rs` and `run_control.rs`.
- [ ] 3.4 Complete explicit stream-axis declarations and source/receipt/durable-position semantics across live and imported evidence; source and receipt timestamps are retained today, while import clocks/admission remain in imported-executions.
- [x] 3.6 Show managed/remote plugin entries and availability in desktop Settings.
- [x] 3.8 Adopt protobuf client contracts, bounded transport/queues and the pinned Harmos SDK; remove interim facade dependencies.
- [ ] 3.8.1.2 Resolve the optional upstream decoded-memory admission follow-up against the selected SDK; existing wire/queue bounds do not establish decoded-memory admission and this was not an adoption gate.
- [ ] 3.8.2 Before stable release replace Cargo and Python Git dependencies with published registry versions, refresh all affected locks and verify release Git-dependency rejection. Keep current immutable development pins until the coordinated upstream work lands.
- [x] 3.9 Co-locate host operation declarations and handlers; remove server forwarding adapters and build-time host contract export.
- [x] 3.11 Compose startup, authorization, access credentials, status, execution, prompt and report lifecycles under the runtime owner.
- [x] 3.12 Persist open/closed/sealed execution evidence with frozen origin/schemas and restrict interrupted-run recovery to PluginRun origins.
- [x] 3.13 Implement attributed outcomes and evaluated-only cycle folding, including empty, declared-only and mixed outcomes in `execution_outcomes.rs` and `outcome.rs`.
- [x] 3.14 Expose origin/state-dependent execution capabilities and reject unsupported start/rerun/cancel paths.
- [x] 3.15 Separate document rendering and report creation from explicit durable report publication.
- [x] 4.1 Make `sdk/rust` the host-free product `dokime` authoring/client crate with generated service coverage.
- [x] 4.2 Provide `dokime new`, plugin, bundle, installer and protocol commands in the one CLI; remove cargo-dokime.
- [x] 4.2.1 Apply the 2026-11-19 plugin verbs: replace CLI `plugin add` with `plugin register` without an alias, retain bundle `plugin install`, remove Python's refusing `plugins.upload` method without adding an `install` stub, and update command consumers, README and the locked SDK specification. Prove CLI units/remote registration/managed installation and Python units/host-free integration.
- [x] 4.2.2 Replace the version subcommand with `--version`/`-V` and `--version --json`, migrate its story consumer and share source revision/channel derivation between CLI and desktop builds.
- [x] 4.3 Replace PyO3 with the pure Python authoring/client package, explicit close and shipped examples.
- [x] 4.4 Provide C# authoring/client APIs, generated roots, examples and scaffold output; retain tracked generated protocol output.
- [x] 4.5 Provide C++ authoring/client APIs, generated roots, examples and scaffold output; retain tracked generated protocol output.
- [ ] 4.6 Re-author the host telemetry plugin with its collectors and controls; its source is restored under `plugins/sidecars/host`, with collection scheduling and effect-journal integration still open.
- [x] 4.6.1 Re-author the OBC device operations, suites, documents and independent benchmark workloads on the Rust SDK; remove plugin analysis registration.
- [x] 4.6.2 Restore `plugins/sidecars/host` (`plugin-host`) with its flat settings, domain rows, area module tree, Build bundle and release plugin declaration (design: Host Telemetry Plugin Restoration).
- [x] 4.6.3 Expose duration settings on the operation context for observer intervals and other durations.
- [x] 4.6.4 Withdrawn: host streams are demand-driven collections (design: Host Telemetry Plugin Restoration); no native collection resource is retained outside listener demand; the CPU calibration scalar may remain cached.
- [ ] 4.6.9 Add the Rust SDK `Collection` (one measurement feeding several streams per tick, demand-bound start and stop hooks) and regroup every host stream row by availability without optional or repeated fields. Row regrouping is implemented; retain the current product producer scheduler unchanged until the library collection capability exists, then adopt it and delete the product scheduler.
- [ ] 4.6.5 Implement the effect journal as host plugin state on the library's state management: `journal.rs` implementing `controls::Journal`, restore newest first in `initialize`, refusal without durable state (landed); durable plugin state served by the library and bounded recovery storage (open).
- [x] Align Python SDK Formatting With the Shared Generator's Explicit Package Classification; Regenerate Rather Than Edit Bindings.
- [x] Stage Both Declared Default Plugins in the Native Kit and Portable Distribution Archive on Windows and Linux; Include Bundle Checksums.
- [x] Review Every Unix-Gated Host Source and Test Branch Against the Journal and Collection Signatures; No Native Linux Compilation Is Claimed.
- [x] Release Collection-Owned COM Usage on Stop and Probe-Owned Usage After Read/Write; Keep Only the CPU Calibration Scalar After All Calibration Threads Join.
- [x] Remove Unused Windows Adapter Helpers From devices.rs; Keep Bluetooth Inventory and the Controls-Owned Adapter Implementation.
- [x] Gate Mutating Host Controls at the Journal Boundary Before Any Platform Call When Plugin State Is Not Durable; Include CPU Load Holds and Preserve Read-Only Operations.
- [ ] Connect the Host Journal's Context Store to the Library's Durable, Revision-Checked Plugin State Once That Capability Exists; Prove Crash Recovery and Bounded Recovery Storage.
- [ ] Complete Hosted Native Linux Compilation and Focused Host Tests; Source Review on Windows Does Not Substitute for This Proof.
- [x] Complete This Host Integration Unit's Focused Windows Proof: Protocol Drift, Host and SDK Tests, Workspace Test-Target Compilation With Desktop testing, Packaging Tests, Format, Repository, Supply and Strict OpenSpec Validation.
- [ ] 4.6.6 Package 2: system, network and CPU collectors, streams, CPU load control, network notifications and benches.
- [ ] 4.6.7 Package 3: Wi-Fi link, scan and latency, and Bluetooth collectors and streams.
- [ ] 4.6.8 Package 4: radio and adapter controls gated on the effect journal, adapter inventory with its typed start snapshot, and power plan operations.
- [x] 4.7 Bundle executable plugins and provide shared Rust/Python/C#/C++ thermal host stories and language fixture producers.
- [x] 4.8 Restore desktop workspace membership, runtime embedding, generated client routing, native preference export and shipping build ownership.
- [x] 4.8.1 Remove the Capture Folder Allow-List, Its Desktop Settings and Refusal Stories; Keep Path, Size and Deadline Validation.
- [x] 4.8.2 Consolidate Native Desktop Stream Settings, Remembered Parameters and Update Selection in `settings.json`; Replace Interval Sampling With Engine / Streams, Without Migrating Retired Files.
- [x] 4.8.3 Use One Framed DataTable for Desktop Lists, Document Tables, Statistics and Grid Plots, With Header Descriptions, Ordered Controls, In-Place Failures and Persistent Pagination.
- [ ] 4.8.4 Complete Focused Frontend, Native Settings and Capture Proof and Compile All Workspace Test Targets With Desktop Testing; Report Documentation Impact to the Owner. Compilation, Frontend and Nine Selected Native Tests Pass; Capture Acceptance Is Blocked by the Remote Readiness REWORK in the Design, Before Capture Assertions Run.
- [ ] 4.8.5 Settle the Scope of the Inherited Updater's Module-Only Feature-Gate Refactor; Do Not Treat Passing Compilation as Convention Closure.
- [x] 5.1 Remove the template run/signal protocol, plugin walker, embedded plugins, native Python backend and old execution hand-off layer.
- [x] 5.6 Remove obsolete template/step/criterion/suite/catalog/plugin macros while retaining message/data derives.
- [x] 5.7 Remove separate client/server/domain crates, host-enabling SDK features, operations contract-only surfaces and their dead consumers.
- [ ] 5.3.1 Add or identify deterministic new-host client-disconnection-during-run and partial-capture interruption stories; prove admitted work survives observer loss and retained partial evidence never claims a complete drain.
- [ ] 5.5 Review retained application behavior, current SDK examples, README and authoring guidance after the three boundary/lifecycle cutovers; remove superseded contracts and source-policy findings without placeholders or hand-editing generated output.
- [ ] 5.11 Restore complete storage/plot/document/finalization/report benchmark coverage on the current host and Nightly selectors; the current storage window/publication benches and desktop harness do not replace every retired workload.
- [ ] 5.12 Complete CLI parity: interactive execution driving, planning shorthand and host/plugin development supervision with unexpected-exit cleanup; verify existing export, pairing, raw discovery/calls and installer commands rather than reimplementing them.
- [x] 5.12.1 Implement saved execution/cycle evidence export through the bounded, verified download path.
- [x] 5.12.2 Restore current-host GUI execution-matrix fixtures and independent native desktop story drivers.
- [x] 5.12.3 Add the global --connect-timeout override for direct and paired connections, retaining ordered endpoint fallback and reporting every failed endpoint.
- [x] 5.13 Remove the unimplemented Effects Create/Get/Set operations, their proto, generated bindings and every SDK's template-context `prepare_effect` / `confirm_effect`; effects are plugin state (4.6.5).
- [ ] 5.14 Resolve the supported media API with the owner and implement retained Media.Captures Create/Get/Watch/Stop behavior, currently Unimplemented; keep unavailable host capture controls explicit. Aggregates.Get implementation belongs to operation-boundary 5.1.
- [ ] 5.15 Complete local capture line matchers/markers and retained FTP/FTPS/SSH sources with feature-gated source operations, secret-backed settings, mirroring, one-shot semantics and bounded pending buffers; current capture engine explicitly defers matchers and FTP/FTPS and has no SFTP implementation.
- [x] 5.16 Implement document structure, bindings, summaries and layout before StructureComplete, then EvidenceCut and card results in `host/documents/get.rs`.
- [ ] 5.17 Prove every retained document expression/filter, higher-order aggregate, plot kind and trajectory/grid report playback through current-host journeys; existing implementations replace the old blanket claims of absent variants, but route presence alone is not complete behavior proof.
- [ ] 5.18 Complete host-owned backup/reset recovery with preserved bundles, settings, host identity and secrets, occupied-root refusal and verified backup; keep destructive desktop recovery unavailable until its public contract exists.
- [ ] 5.19 Expose connected-host build target/revision/channel to plugin release selection and verify operation discovery enables only actually served storage-source mutations; retain explicit unavailable controls while the contracts are missing.
- [ ] 6.1 Run final focused host, CLI, installed four-language SDK and generated-consumer proof on the combined published dependency candidate; include frozen-origin/schema evolution and plugin absence, bounded retained queries/watch gaps, authorization, Public peer calls/cancellation and execution-file live producer/follow behavior.
- [ ] 6.2 Run final managed/remote plugin stories, both native WebView lanes and packaged desktop/installer organization, licensing, signing and update journeys on native Windows and Linux. Source inspection and historical Windows results do not close this proof.
- [ ] 6.3 After all source owners hand off, complete configured format/lint/language checks, Tests producer, locked dependency metadata and authorized hosted CI/Nightly; repair failures without weakening checks and obtain delivery authorization before pushing.

## Locked Authoring Surface

- [x] Python: Resolve Starter and Example SDK Dependencies from Nightly Git, Preserve Working-Tree Test Overrides and Reject Python Git Dependencies for Stable Releases.
- [x] Python: Implement Product Exports, Typed Declarations, Generated Names, Plain Values and Visibility; Remove Raw Records and Sections.
- [x] Python: Pass Strict Installed-Wheel Stubs and Typed Authoring Consumers, Including Plain Payloads and Decorators.
- [x] Python: Resolve the Root Dispatcher's Directory-Derived Unit Lane Against the Declared Integration/System Lane Inventory Before Root Unit Dispatch Can Run.
- [x] Python: Migrate Every Python Example and README to the Locked Surface and Entrypoint Layout.
- [x] Python: Complete Native Integration Proof With Underscore Identifiers Against This Worktree's Host.
- [x] Python: Qualify Materialized Step and Criterion Instances With the Shared Identifier Function and Support Bound Methods.
- [ ] Rust: Follow the Locked Authoring Surface.
  - [x] Rust: Bind Local Template Declarations, Generate Code Names, Wrap Plain Values and Remove Plugin-Type Growth.
  - [x] Rust: Migrate the Owned Examples, OBC Plugin and Host Fixture Sources; Keep the Client Flow Separate.
  - [x] Rust: Complete SDK, Example, Host-Fixture, Format and Focused Language Proof in This Worktree.
  - [x] Rust: Migrate the Retired SDK Surface in Owner-Scoped `crates/dokime-runtime/benches/storage/{windows,publication}.rs` and `crates/dokime-gui/benches/system/fixture.rs` (Derive Attributes, Plugin Records, Window Type, Local IDs and Selectors).
  - [ ] Rust: Resolve Runtime-Defined Plain Context Operation Calls Against the Pinned Carrier's Static `OperationRequest` Contract.
- [x] C#: Follow the Locked Authoring Surface.
- [x] C#: Complete the Language Check and Host Fixture Producer With SDK-Owned Python Root Exports.
- [x] C++: Follow the Locked Authoring Surface.
- [x] C++: Pass Native Windows SDK Build, SDK Tests, Format and Language Check, Including the Debug Thermal Fixture Target.
- [x] C++: Complete the Prescribed Host-Fixture Producer Proof With This Task's Authorized Release Build.
- [x] Command Line: Implement Locked Point 11 With the Manifest-Project Amendment.
- [x] Command Line: Remove Install/Add Trust Flags, Refusal and Callers; Installation Asks Nothing.
- [x] Command Line: Delete Superseded Python Remote Control and Bootstrap Scripts.

- [x] Integration: Normalize Member Code Names and Qualify Short Prompts Across All Four SDKs.
- [x] Integration: Give the Python SDK Sole Ownership of Its Authoring Root Exports.
- [x] Integration: Migrate Thermal and OBC Evidence Selectors and the Python Plugin Fixture Entrypoint.
- [x] Integration: Add a Shared Four-Language Capture Description and Record Contract Story Under `sdk/tests`.
- [ ] Integration: Prove Generation, Ordered Host Fixtures, Four SDKs, Shared Stories, Command-Line Development Stories and Migrated Runtime Stories on the Combined Tree.
  - [ ] Windows: Complete the Ordered Native Proof on the Combined Authoring Candidate.
  - [ ] Linux: Complete the Same Ordered Native Proof After Authorized Delivery.
- [x] Integration: Complete Format and Focused Language Checks on the Combined Tree.
- [x] Integration: Remove Plaintext Execution-ID Report Assertions Superseded by the Shared Protobuf Capture Renderer.
- [x] Integration: Align Short Text Prompt Choice Archetypes Across Python, C# and C++ With the Rust SDK and Prove Native Evidence Parity.
- [x] Integration: Make CLI Development Stories Headless and Remove Their Unreachable Sibling-Desktop Data-Root Overrides.
- [x] Integration: Verify Manual Prompt Waiting and Answering in Both SDK Remote First-Hour CLI Journeys After the Dev Merge Without Changing Operation Policies.
- [x] Integration: Freeze Reports at Creation and Deliver Their Retained Bytes in Both Native Desktop Stories.
- [x] Shared Generation: Preserve Watch Metadata in Rust `host.rs` Request Bindings; Resolve the Unary/Streaming Fingerprint Mismatch Before Host Admission Proof.

## Plugin Project Manifests

- [x] Leave Python Starter Locks to the User's First `uv sync`, Remove Unused Repository Example Locks and Assert Both Created Inventories Contain No Lock.
- [x] Give Every Buildable Plugin and Test Starter a Root Manifest, Applicable Mise Tasks, Ignore File and Line-Ending Attributes in All Four Languages.
- [x] Keep the Python Operation Starter Serving With Closed Standard Input.

- [x] Pin the Launch-Capable Library Revision in the Workspace and Rust Consumer Manifests and Locks.
- [x] Read Strict `dokime.toml` Plugin Manifests With One Native Executable or Launch Command and Confined Relative Paths.
- [x] Build Manifest Projects Before Development or Bundling; Require Python Project Inputs Before Use.
- [x] Replace Executable Flags and Python-Folder Detection With Bundle or Manifest-Project Development Inputs.
- [x] Attach the Sibling Packaged Desktop When Present and Close It With the Disposable Host.
- [x] Add `dokime dev --no-desktop` to Skip Desktop Lookup and Use It in Every Automated Caller While Preserving Default Desktop Supervision.
- [x] Use Only `.pair` for CLI Remote Access Files, Default Exports, Tooling Callers and README Examples.
- [x] Leave Development Credential Namespace Cleanup With `dokime dev`; Remove Scratch Runner Credential Scanning and Its Superseded Test.
- [x] Complete Native Windows Proof of the Headless Development Flag With CLI Units, All CLI System Stories, Present Executable-Sibling Bypass, Formatting, Focused Compilation and Rust Lint.
- [x] Bound Native Catalog Registration Stack Use Without Changing Operation Policies, Thread Placement or Stack Sizes.
- [x] Embed Manifest Files in the Four Test Starters and Update README Command Examples.
- [x] Complete Focused CLI, Packaging and Runtime Unit Proof, Owned-File Formatting, Locked Metadata and Three-Package Compilation With Tests.
- [x] Migrate Product and Desktop-Fixture Tooling From Retired `--executable` Calls to Manifest Projects in `tools/python/src/dokime_tools/{build/rust/__main__.py,build/desktop_fixtures.py,dev/desktop.py}`.
- [x] Update the Compiler-Record Story to Stage Real Executable Fixtures Before Manifest Bundling and Pass Tooling Integration Tests.
- [x] Generate the Shared Report Template Before Independent Rust Fixture, Test-Archive and Lint Compilation; Propagate Generation Failures Before Compilation.
- [x] Preserve Focused Native JUnit Reports Before Scratch Cleanup and Keep Cleanup Failures Visible.
- [x] Keep the Launch-Folder Round-Trip Interpreter Outside the Shipped Inventory and Prove Source-Only Bundling and Execution Without Changing Byte Bounds.
- [x] Generate the Shared Report Template After Restoring Coverage Clients and Before Instrumented Runtime Compilation; Prepare It Before Storage-Only Benchmark Compilation and Audit Every Runtime Compiler Lane Without Committing Generated Reports.

## Desktop Reports and Paired Python

- [x] Share the Journey's Submit-and-Observe Report Save With Screenshots and Measure Thermal/OBC Creation, Transfer and Write.
- [x] Keep Desktop and Python Remote Access Files at `.pair`; Prove the Python Client Reads the Portable File.
- [ ] Complete the Ordered Pairing Endpoints Contract in remote-access-simplification: Reuse an Established Python Client Read Policy Before Adding Bounded Sequential Attempts and Prove Refusal, Aggregate Failure and Identity-Mismatch Fallback With Local Listeners. No Such Policy Exists in the Current Python SDK or Its Pinned Shared Transport; Keep This Item Open. Reconcile CLI `ca_certificate_pem` With Desktop/Python `certificate_pem` Before Claiming Cross-Client Portable Files.
- [ ] Add and Prove Python Drives a Paired Desktop Through the Real Remote Access Export Control and the Unmodified Starter.

## Single Plugin Paths

- [x] Share Entry Readiness Between CLI Add/Install and the Host Without Changing Their Failure Messages or Watch Policies.
- [x] Replace the Native Bundle Encoder With the Pinned Library Builder After Native Manifest and Install/Start Equivalence Proof; Remove Encoder-Only Dependencies.
- [x] Audit Standalone Product and Library Managed Process Supervision and Report the Required Library Surface Without Replacing Either Supervisor.
- [x] Prove the Three Crates' Units, All CLI Stories, Plugin Runtime Stories, Rust/Python Starter Bundles, Formatting, Focused Compilation and Rust Lint on Native Windows.

## Locked Python Shape

- [x] Adopt the Library Typed Subscription Pin and Regenerate All Protocol Projections and Tracked Locks; Repair Forced Rust Callers and Tooling Imports.
- [ ] Bind Python Typed Live Rows and Explicit Gaps to Library Subscription Demand; Prove Both Timestamps, Independent Close and Scope Cancellation.
- [ ] Remove the Unused Rust Settings Derive and Migrate Host Duration Reads Directly to the SDK.
- [ ] Complete Pin and Cleanup SDK Units, Integration, Lint, Type and Stub Checks; Focused Rust Proof, Workspace and Consumer Test Compilation, Binding, Supply, Build/Package, Generation Drift, Format, Repository and Strict Change Gates.

- [x] Python Peer Calls: Derive Frozen Requirements From Typed Handler References and Preserve the Existing Bound Call and One-Call Policy Override, Without Author Requirements.
- [x] Python Peer Calls: Prove Derivation, Conflicting Contracts, Structured Failures and Undeclared References With SDK Unit Tests.
- [x] Python Peer Calls: Prove Two Fixture Plugins Through the Host, Including Success, Scope, Callee Deadline/Cancellation, Lane Release and Actionable Unavailable/Undeclared Failures.
- [x] Python Peer Calls: Share the Scheduler Clock Adapter in `crates/dokime-runtime/src/clock.rs`, Add Manual Clock Control to the Native Fixture Host and Its Python Test Support, and Prove Runtime Policy Deadlines Without Advancing Only Execution Time.
- [x] Python Peer Calls: Complete SDK Checks, Python Build/Package, Workspace Test-Target Compilation, Format, Repository and Strict OpenSpec Proof; Report Native Linux and Other SDK Languages as Unproven.

- [x] (a) Rename the Python Package Alone per the SDK Surface Specification Ownership Inventory; Regenerate Imports, Wheel Paths and Guards Without Keeping `dokime` Aliases.
- [ ] (b) Implement the SDK Surface Specification's Python Authoring Classes, Separate Contexts, `ctx.plugin` and `plugin.serve()`; Resolve Its Named Capability Decisions Before Their Dependent Work.
  - [x] Bind Explicit Plugin, Suite and TestTemplate Classes, Derived Identities and Labels, Annotated Plain Operations and Separate Contexts; Move Documents, Plots and Parameters to Templates and Delete Replaced Authoring Entry Points.
  - [x] Prove Fixed Builder-Description Wire Equivalence, Scoped Template State, Primitive Records and Operations, and Cleanup After Failed or Cancelled Template Initialization With Python-Only Tests.
  - [ ] Complete Typed Stream Subscription and Real-Host Failure/Cancelled Plugin Initialization Finalization, Admission Draining and Repeated Shutdown Proof; the Operation-Encoded Hooks and Their Native Host Invocation Are Implemented.
  - [ ] Bind Typed Host/Plugin Stream Subscription to the Library's Admitted Demand, Backpressure and Close After Resolving the Live Row Shape and Public Export Ownership.
  - [x] Publish Records From Plugin Observers After the Library Allows Product Call-Context Resources on Scoped Sample Child Calls.
  - [x] Prove Product Failure Classifications, Causes, Identity and Actionable Details From Bound Python Operations.
  - [x] Expose and Prove a Decorated Operation's Immutable Typed Declaration Through `operations.declaration`.
  - [x] Complete and Prove Python Text/List/Choice/Required Parameters, Traceability Cards, Plot Markers, Section Repeats and Bound Record Fields.
  - [x] Adopt Library-Owned Minimum/Maximum Setting Bounds When Its Setting Declaration and Validation Support Them.
  - [x] Seed Scoped Observer Bindings With Execution Resources and Replace Inherited Resources for Each Dispatched Operation.
  - [x] Declare Host and OBC Scalar Setting Bounds Through the Library and Remove Local Scalar Range Validation.
  - [x] Refuse Offline Python Registration and Release Its Listener; Prove Library Host Default and Settings-Write Bounds With Fake Plugins.
  - [x] Remove Rust Reflective Setting Declarations and Migrate Their Consumer to Explicit `setting` Declarations.
  - [ ] Agree and Implement Stop-Completes-Run Across the Product Declaration, Runtime and Durable Sealing Owners; No Equivalent Contract Exists in This Candidate.
  - [ ] Adopt Library Observer Columns With Per-Row Optional Presence and Prove Absent, Zero and Nonzero Fields Through Product Storage.
    - [x] Prove Absent, Zero and Nonzero Values Through Python Observer and Window Carriers and Frozen Sample Decoding at Library Revision `3cc1166fa`.
  - [x] Generate Missing Host Frontend and Report Inputs for Focused Rust Fixtures Through the Existing Renderer.
  - [x] Replace Fixed Capture Catalog Counts With Member Lookup, Read Exported Fixture Locations and Stage the Rust History Reader.
  - [ ] Complete Native Fixture Production and Four-Language Capture Proof After the Library Pin's Rust Consumers and Runtime Context Overrides Are Migrated.
  - [x] Share Template/Client Record Query and Decode Ownership, Termination Values, Bound Operation Calls, Plugin-Owned Listener Activation and Artifact Publication; Complete Typed Statistics, Artifact Projection and Local Document Declaration Binding.
  - [ ] Admit Plugin Notes/Create and Resolve Complete Automatic Peer Requirements Against the Library's Frozen Activation Requirements; Authors Declare No Requirements.
  - [x] Complete Installed-Wheel Stub/Public-Home Consumer Proof With the Three Intentional Python Homes.
  - [x] Run the Required Authoring Host Stories on the Combined Candidate.
- [ ] (c) Implement the SDK Surface Specification's Python Bound Handles, Host Compositions and Configuration Precedence/Grammar With Secret Storage, Readback, Controls, Seal, Evidence, Notes, Artifacts and Explicit Close.
  - [x] Replace Owned Selector/Export Wrappers With Bound Client Resources, Current Host Compositions, Immutable Product Values, Structured Failures and Explicit Owned Registration Cleanup.
  - [x] Implement Working-Directory Configuration Precedence and the Shared Dotenv Grammar Without Dependencies or Environment Mutation; Preserve Ordered Pairing Attempts and Connection Budgets.
  - [x] Share Verified Archive/Report Downloads, Preserve Acknowledged Upload Fences and Child Request Identities, and Prove Frozen Decoding, Fixed-Cut Queries, Controls, Notes, Artifacts and Cleanup With Host-Free Services.
  - [x] Bind Host Execution Seal Outcomes and Sealed Progress; Prove Lazy Run/Watch Finality and Observation Close Without Execution Cancellation With Host-Free Services.
  - [ ] Bind Managed Bundle Installation and Client Invocation Policy Overrides After Their Host Contracts Land.
  - [x] Complete Real-Host Retained Archive Proof Without Substituting Readback for Seal.
  - [x] Resolve Shared Query, Termination, Bound Operation, Artifact Publisher and Listener Activation Ownership With Item (b).
  - [x] Complete Installed-Wheel Stub/Public-Surface and Typed Consumer Proof With the Migrated Inventories.
- [x] (d) Implement the Exact Manifest Schema, Dynamic Metadata, Implicit Assets, Bundle Identity/Version and Four-Language `dokime new` Embedding per the SDK Surface Specification.
  - [x] Replace Top-Level Manifest Scripts and Assets Lists With Strict Plugin, Compatibility and Scripts Tables; Resolve Native Metadata With Static Overrides.
  - [x] Bundle Implicit Assets, Referenced Metadata and Required Launch Inputs; Preserve User Locks and Exclude Private Inputs.
  - [x] Migrate the Starter Manifests and Shared Command-Line Fixture Consumers While Preserving Current Entrypoints and Build Commands.
  - [x] Supply Rust, C# and C++ Starter Native Metadata and Icons, Stage C# Publish Companions Under Assets and Migrate Runtime Manifest Consumers.
  - [x] Reconcile the Python Starter Source Inventory With Its Agreed Manifest Roots and Supply Native Metadata and Icons for That Inventory.
  - [x] Prove Installed Resolved Versions After the Library Plugin Manager Retains the Frozen Bundle Version Across Activation and Replacement.
  - [x] Complete Workspace Compilation, Packaging Units, Runtime Manifest Command Proof, Formatting, Repository Checks, Package Lints and Created Rust/C# Bundle/Inspect Proof.
  - [x] Complete CLI Units and Created Python/C++ Starter Stories After Their Source Inventory and SDK Artifacts Are Ready.
- [x] (e) Migrate Python Starters, Examples, Generation, Public/Stub Guards, Tests and Stories to the Reference Project Shape; Ship No Lock, Root Entry Scripts, Argument Parsing or Old Imports.
  - [x] Replace the Owned Python Starter Sources With the Reference-Shaped Single Project, Its Host-Free Description Test and Explicit Project/Package Substitution Markers.
  - [x] Migrate the Client/Thermal Example Packages, Python Fixture Launchers, Typed Consumer and Three-Home/Strict Stub Inventories; Keep Repository Host Stories Outside the Starter.
  - [x] Reconcile the Manifest/Asset and CLI Embedding Owners.
  - [x] Retire the Generated Public Client Facade Through Its Generator, Share One Three-Home Inventory and Complete Editable-Install Metadata and Listener-Address Readiness Support in the SDK.
  - [x] Complete Strict Installed Stubs, All Host-Free Integrations, Repository Checking, Isolated Editable Starter Checks and Native Listener/Interrupt Proof; Compose the Shared Capture Fixture From the Starter Suite.
  - [x] Run the Named Host Stories on the Combined Candidate: NativeAuthoringStory, StarterStory, EvidenceQueryStory, Runtime Thermal Client and Created Python Starter First-Hour Journeys; Complete Four-Language Capture Equivalence With Prepared Native Fixtures.
- [ ] (f) Implement the SDK Surface Specification's Rust SDK and Two-Binary Starters With Owned Generation, Focused Proof and No Superseded Client or Authoring Paths.
  - [x] Replace Plugin/Suite/Template Explicit Constructor Families and the Single-Template Shortcut With Marker Constructors and Final Suite Binding.
  - [x] Separate Operation and Template Contexts, Share Plugin-Owned State, Return Record Ordinals and Migrate Owned Declarations.
  - [x] Implement Connection Settings, Working-Directory Dotenv Grammar and Precedence, Direct Origins and Ordered Pairing Attempts.
  - [x] Add Bound Case/Cycle/Execution Handles, Lazy Start/Watch Finality, Stable Seal Outcomes, Verified Archives and Client Notes/Artifacts.
  - [x] Pin the Owned, Addressable, Closeable Rust Listener at the Shared Maintenance Revision; Register, Await Ready, Unregister and Close With Explicit Listener Configuration Without Copying Its Server or Mutating Process Environment.
  - [ ] Complete Typed Runtime Operation References and Stream Subscription After Their Shared Library Capabilities Land.
    - [x] Remove the Legacy Public Static-Request Context Call and `OperationRequest` Re-Exports; Keep Unsupported Runtime References Unexposed.
  - [ ] Bind Bundle Upload After the Product Host Publishes Its Installation Contract and Regenerated Bindings; Shared Plugins/Create Currently Accepts Placements Rather Than Bundle Uploads.
  - [ ] Complete the Remaining Declaration/Context Capability Surface and Catalog Validation, Including Operation-Scoped Observers, Typed Query Values and Template Notes/Artifacts.
    - [x] Share Client/Template Artifact Uploads, Preserve Writer Fences and Mutation Identities, Verify Publication and Abandon Only Unpublished Uploads.
    - [x] Add Plugin-Wide Per-Sample and Whole-Window Observers With Operation Contexts and Shared Plugin State; Refuse Template Contexts at Plugin Scope.
    - [ ] Bind Template Notes After `Notes/Create` Admits Trusted Plugin Callers; Its Current Protocol Annotation Is `CLIENTS` Only.
    - [x] Scope the OBC Measurement Observer and Stream to Each Capturing Template; Keep Execution State Out of Plugin-Wide Observers.
    - [x] Migrate OBC to Typed Context Record/Statistics, Settings, Prompt, Artifact and Traceability Readers; Retire Context Wire Responses and Generated Host Escape Hatches.
  - [x] Complete Registration-Owned Template Bindings, Cleanup, Resource Lists, Mutation Overrides and Frozen Typed Readers; Keep CLI and Product-Test Raw Calls in the Unpublished Workspace Client Crate and Remove Generated Host/Transport Escape Hatches From the SDK.
    - [x] Add Bound Case Lists, Fixed-Cut Cycle/Execution Lists, Refresh Operations and Frozen Typed Record/Sample Readers With Digest Verification and Continuation Validation.
    - [x] Add Operation-Specific Mutation Identity Overrides for Cycle/Execution Creation and Controls, Notes and Artifacts; Derive Replay-Stable Execution UUIDs From Request Identities.
    - [x] Add Template-Context Request Identity Overrides for Records, Progress, Logs, Traceability and Documents Without Policy Overrides.
  - [x] Run SDK Harness Cleanup After Failed or Cancelled Initialization, Preserving Partial State and Termination; Prove Both Deterministically.
  - [x] Create All Four Languages With `--from` or `DOKIME_NEW_FROM`, Checkout Starter Sources and Native Local SDK Dependencies; Keep Argument Precedence and Actionable Missing-Starter Errors.
  - [x] Migrate Starter-Creation Stories and Fixture Tooling to `--from`; Remove Test-Owned SDK Dependency Overrides and C++ Archive Coordinates.
  - [x] Prove Failed/Cancelled Product Initialization Cleanup Exactly Once With the Deterministic Runtime Story.
  - [x] Prove the Created Rust and C# Starters Through Build, Bundle, Inspect, Headless Development and Unmodified Client Runs; Prove Python Creation.
  - [x] Prove Shared Four-Language Capture on the Combined Candidate.
  - [x] Complete CLI Units/Stories, SDK Units/Integrations, Generated Checking, Repository Checking, Formatting, Scoped/Workspace Compilation and Final Rust Lint on This Candidate.
    - [x] Move the Raw Protocol Fixture to `crates/dokime-client-internal/src/bin/content_fixture.rs` Without Any SDK Dependency; Preserve Every Trusted-Plugin Stale-Revision, Replay and Upload-Writer Fence Probe.
    - [x] Validate Rust Project Names Against Rust Keywords and SDK Shadowing, and Run Created-Project Compiler Processes Through the Existing Owned Process-Tree Runner Without Changing Deadlines or Overriding Session Compiler Concurrency.
    - [x] Migrate Runtime and Desktop Benchmark Source Consumers to Marker Constructors, Native Template Contexts, Standard Plugin Run and the Unpublished Product Client; Typecheck Them in the Dev Profile Without Building or Running Benchmarks.
  - [x] Preserve Separate SDK Harness and Product Execution Sequencing Per the Owner Decision; Run Product Cleanup After Attempted Initialization, Including Failure and Cancellation.
  - [x] Replace the Three Rust Example Roots With One Two-Binary Starter, Migrate Their Consumers and Embed That Starter With Project Name Substitution; the Earlier Two-Root Manifest Table Is Superseded by This Task Decision.
  - [x] Prove Focused SDK Units and Integration/Capture Tests, the Protocol-Only Dependency Boundary, Isolated Generated-Surface Compilation, Formatting, Scoped All-Target Compilation and Repository Checking.
  - [ ] Complete Root Generated Checking and Scoped Rust Lint After the Integrating Owner Refreshes the Root Lock; Prove Shared Capture Against the Product Host and the Created Starter's Bundle/Inspect/Dev/Client Flow on the Required Native Platforms.
    - [x] Refresh the Root Cargo Lock and Validate the Consumer Lock; Build the Native Windows Product Host/CLI and Prove Rust Frozen Typed Record Reads and Verified Execution/Cycle ZIP Downloads.
    - [x] Prove Shared Capture in All Four Languages on the Combined Candidate.
    - [x] Pass Native Windows SDK Units, SDK Integration/Capture Tests, CLI Units/Stories, Formatting, Scoped/Workspace Compilation, Generated-Surface Checking, Repository Checking and Final All-Target Rust Lint on the Combined Candidate.

- [ ] (g) Implement the SDK Surface Specification's C# SDK and Shared-Library/Plugin/Client Starters With Owned Generation, Focused Proof and No Superseded Client or Authoring Paths.
  - [x] Prove the Created C# Starter Through Build, Bundle, Inspect, Headless Development, Its Unmodified Client and a Timestamped Retained Evidence Archive.
  - [ ] Resolve the Duplicate Artifact Upload and Story Listener Owners Listed in the Design's Overlap Inventory Before Removing Either Implementation.
- [ ] (h) Implement the SDK Surface Specification's C++ SDK and Two-Binary Starters With Owned Generation, Focused Proof and No Superseded Client or Authoring Paths.
  - [x] Introduce Root Plugin/Suite Classes and Separate Operation/Template Context Homes, Explicit Suite/Template Composition, Type-Derived Identities and Product Description Values; Remove the Single-Template Shortcut and Public Carrier Context Access.
  - [x] Implement UTF-8 Working-Directory Dotenv Parsing, Per-Key Address Precedence and Private Managed-Launch Precedence Without Process-Environment Mutation.
  - [x] Preserve Template State Through Failed Initialization Until Cleanup, Carry Cleanup Termination and Share the Author Plugin Across Both Contexts.
  - [x] Share Fenced Artifact Transfers Between Clients and Template Contexts; Add Capture, Environment, Copied Full Prompts and Mutation Identity Overrides and Declare Their Host Requirements Automatically.
  - [ ] Complete the Remaining Authoring Capability Inventory, Context Notes, Typed Stream Subscription and Automatic Peer Requirements Without Serial Admission or Settings Groups.
  - [x] Adopt the Library-Owned Addressable, Closeable C++ Listener for Plugin Registration; Do Not Copy the Server or Add Address Polling.
  - [x] Implement Connection/Pairing, Bound Case/Cycle/Execution Host Compositions, Lazy Run/Watch, Stable Seal Outcomes, Verified Evidence, Fixed-Cut Typed Readers, Controls, Prompt Answers and Structured Client Failures; Internalize Transfer Helpers.
  - [x] Complete Registration-Owned Template Bindings and Listener Cleanup Through the Library-Owned Listener.
  - [ ] Bind Managed Bundle Upload After the Product Installation Contract Lands.
  - [x] Replace Both C++ Starter Roots With the Requested Single Two-Executable Project, Native Static Manifest Metadata and the Unmodified Bound-Client Flow; Migrate the C++ CLI Embedding Functions.
  - [x] Integrate the SDK-Owned C++ Projection With the Shared Protocol Generator Under Its Owner; Keep Tracked Bindings Generated.
  - [x] Complete Native Windows C++ SDK Unit/Integration, Installed Debug CMake Consumer, Generated Binding/Compiler, Formatting and SDK Checking Proof.
  - [x] Complete Repository, Product-Host Shared Capture and Created-Project Bundle/Inspect/Dev/Client Proof; Linux Proof Remains Independent.
- [ ] Replace Python `plugins.create` With `plugins.register` and `plugins.install`, With Readiness Owned by the Host and Registered Entries Retained Until Explicit Remove.
- [x] Unify Rust, Tooling, Python, C# and C++ Library Pins at the Owner-Selected Maintenance Head and Regenerate Every Tracked Binding.
- [x] Encode Plugin Initialize and Finalize Hooks as Well-Known Operations in All Four SDKs and Remove Replaced Stop Callbacks.
- [x] Replace Retired Plugin Remote Arguments With Product Listener Coordinates in Runtime Launchers, C# Serving and Native Fixture/Benchmark Dispatch; Acknowledge Lifecycle Operations in the Settings Protocol Fixture.
- [x] Align the SDK and C++ Starter Native Dependency Baselines With the Pinned Library and Remove the Bootstrap's Hard-Coded Checkout Tag and Stale CMake Toolchain Configuration.
- [x] Stage Release Runtime Inputs From the C++ Starter's Normal Build for Bundling; Keep Debug Outputs in Its Build Tree and Preserve Bundle Bounds and Build Deadlines.
- [x] Align CLI Fixture Producers With Their Actual Build Outputs and Compiler Profiles; Prove the Rust Fixture and Both C++ Placement Journeys.
- [ ] Merge Current Dev Before Final Proof; Run Workspace Compilation, Rust Lint, Repository and Generation Checks, All SDK Units/Integrations, Four Created Projects and Capture, CLI Stories and the Requested Release Host/Python Stories; Resolve the Unanswered-Prompt Clock/Stream REWORK Before Closing Final Proof.
  - [x] Resolve the Current Dev Merge While Preserving Host Telemetry, Environment Context and Effects Removal Alongside Host Build Identity, Logs/List and Desktop Settings Consolidation; Regenerate Protocol, Python and Desktop Bindings From the Merged Sources.
- [x] Prove End to End That the Created Starter's Client Flow Runs Against a Host and Passes the Type Checker.

- [x] Substitute Python Starter Display Titles With Style-Guide Title Case and Prove Name, Package and Display Substitution.
- [x] Run Both Python First-Hour Journeys Through Product Development Commands Instead of a Hand-Launched Listener.

- [x] Supply Replay-Stable Execution UUIDs From Bound Python Handles and Prove Native Descriptor Readback With Synthesized JSON Names.
- [x] Return Frozen Admitted Suite and Template Metadata From Host Execution Readback; Prove Reopened Python Declaration Queries.
- [x] Run the Unmodified Created Python Client With CLI Prompt Answers, Execution and Cycle Seals, and Member-Identical Host Archives.
- [x] Run the Shared Capture Through a Product Host in All Four Languages While Retaining Their Current Authoring Surfaces.
- [x] Refresh the Owner's Scratch Python Project From the Shipped Starter and Leave Its Validated Evidence Archive Under `_output/`.
- [x] Prove Positive Python Retry Policies in NativeAuthoringStory and Both Python Thermal Runtime Stories; Reject Non-Positive Backoff at Operation Declaration and Keep the Explicit Fixture Delay.
- [x] Replace Python Template Setup With the Locked Initialize Lifecycle Across Declaration Binding, Execution Dispatch, Examples and Fixtures Without an Alias.
- [x] Prove Python Thermal Evidence Parity With the Locked Initialize Lifecycle and Preserve the Complete Evidence Comparison.
- [x] Resolve the Plugin Class Identity Override Versus Resolved Instance Readback: Expose the Resolved Identifier on Each Instance Before and After Registration.
- [ ] Keep Unanswered Starter Prompts Pending Across the Host Mutation Deadline and Explicit Controlled-Clock Advancement; Answer Them to Seal Passed With Retained Evidence. Resolve the Clock Jump Versus Continuous Stream Conflict Without Relaxing Either Policy.
- [x] Complete the Requested Native Windows Python Shape Proof Set on the Dev-Merged Candidate With Positive Retry Backoff, the Library's Frozen Bundle Version Fix and Final Rust/TypeScript Lints.
- [x] Rerun the Existing Scratch Python Project Against the Merged Host, Select Its Execution From CLI Summary JSON, Answer Through the CLI and Preserve Its Saved Archive Without Regenerating or Deleting Project Files.

## Hand-Run Plugin Registration

- [x] Add `PluginsCreateRequest.sdk_version` at Field 6 and Regenerate the Tracked Bindings.
- [x] Prove Python Paired Startup, Stable Identity, Refusal Output, Silent SDK Serving and Entry Retention.
- [x] Complete Rust Paired Startup, Stable Identity, Actionable Refusals and Entry Retention With Focused Proof.
- [ ] Prove Rust Remote Authoring With the Pinned Library's Registry Process-Ownership Fix.
- [ ] Complete C# Paired Startup, Stable Identity, Refusals and Entry Retention With Focused Proof.
  - [x] Prove Stable Registration, SDK Version Transmission, Refusal Handling, Listener Completion, Repeated Disposal, Explicit Removal and Client-Close Entry Retention.
  - [ ] Prove Automatic Paired C# Process Startup Against the Combined Host.
- [ ] Complete C++ Registration and Entry Retention After Preparing Its Native SDK Build.
- [ ] Complete the Remaining Registered-Listener Wait APIs Where Required; Keep the Python Run Surface at `serve` and Keep Readiness off Standard Output.
- [ ] Agree One Plugin-Presentation Contribution With the Host Owner and Carry Label, Description and Icon Through the Shared Description.
- [ ] Prove Every SDK's Library-Owned Launch Announcement at the Current Pin, With Credential Detection in Its Single Run Entry.
- [x] Complete Binding and Workspace-Test-Compilation Gates.
  - [x] Pass the Registration Wire Round-Trip and Compile Every Rust Consumer Test Target.
  - [x] Pass the Release-Context Python Binding, Strict Stub and Generated-Surface Checks.
  - [x] Pass Repository Checks and the Created Python Starter's Unmodified Client and Retained Archive Story.

## Library Pin, Installer Feature and SDK Lanes

- [x] Refresh Every Library Pin to the recorded revision, Its Locks, Vendored Protocol Inputs and Generated Bindings; Adapt Existing Consumers and Prove the Focused SDK Tests and Runtime Test Compilation.
- [x] Gate Installer Packaging and Its Exclusive Dependencies Behind the CLI-Selected `installer` Feature; Prove Isolated Consumer Dependencies and Packaging/CLI Units.
- [x] Expose Library-Owned Lane Declarations and Operation Assignment Through All Four Product SDKs; Prove Each Description and Preserve Library Validation.
- [x] Run the Requested Final Repository, Rust Lint, Release-Context Binding and Supply Checks Without Expanding Local Test Lanes.


## Registered Plugins and Launch Consumers

- [x] Present an Unreachable Remote Entry as Offline Without a Failure and Keep It Durable Until an Explicit Delete.
- [ ] Reuse a Registered Plugin's Entry by Its Identifier on Re-Registration, Preserving Replay, Conflict Refusal and Saved Settings When Its Address Changes; Blocked by the Library's Managed-Only Replace.
- [x] Retain an Offline Plugin's Last Reported Suites, Marked Unavailable, and Refuse Planning Them With an Action.
- [ ] Retain Offline Operation Contributions and Present Suite Availability in the Desktop Catalog; Operation Retention Needs the Library Catalog Owner, and the Desktop Catalog Is Outside This Worker's Files.
- [ ] Refuse Every Incompatible Remote Registration, Naming Both Versions; Supplied Versions Are Checked, but URL-Only CLI Registration Has No Product SDK Version Metadata and All SDK Callers Must Supply Their Built Version.
- [x] Show Placement and Status in the Desktop Plugin List and `dokime plugin list`, Filterable in the Desktop.
- [ ] Stop Deleting the Entry When a Registered Plugin Exits and Send the SDK Version From Every Product SDK.
- [ ] Reconnect Initially Unreachable Remote Entries Through the Library's Existing Policy and Distinguish Unreachability From Other Activation Failures; Blocked by the Pinned Library's Lifecycle and Failure Classification.
- [ ] Move Every Launch Readiness Consumer to `Launches/Announce` and Remove the Retired Listener Environment Configuration; Prove the Merged Starter and SDK Consumers.
- [ ] Prove Registry Removal Keeps Externally Owned Endpoints Serving With the Pinned Library's Ownership Fix.

## Desktop Failure Presentation

- [x] Normalize Library Postcard Carriers and Product Status Details to One Generated Failure Protobuf at the Desktop Bridge; Preserve Plain Status Codes and Messages.
- [x] Log Failed Host Requests and Deduplicated Plugin Failures at the Native Session Boundary With Operation, Code, Message, Action and Plugin Fields.
- [x] Share Full-Width Failure Presentation, Decoded Copyable Details and Supported Recovery Actions; Keep Dependent Plugin Sections Quiet and Limit Toasts to Operator Actions.
- [x] Keep Failed Plugin Settings Editable and Saveable Beneath Their Failure Card; Retain Identity and Secret Availability and Handle Disabled Undescribed Entries.
- [x] Record Operator Action Failures Through Explicit Notification Opt-In; Keep Passive Reads and Reopened History Details Out of History.
- [x] Retain Failed Connection Endpoints and Complete Notification Metadata in Copied Details.
- [x] Restore Focused Coverage for Failure Identity, Root-First Causes, Admission Identifiers, Startup Recovery and Preference Persistence Refusals.
- [ ] Complete the Release-Artifact-Dependent Binding Gate and Native Desktop Stories on the Required Platforms.

## Launch Repair Integration

- [x] Merge SDK Self-Registration and Current Dev, Regenerating Protocol Outputs and Keeping Shared Plugin Tables, Failure Cards, Placement and Offline Presentation.
- [x] Select the Four SDK Run Entries by Launch Credentials, Then Pairing, Then Standalone Serving; Point Every Starter at That Entry.
- [x] Remove Retired Address Configuration, SDK Readiness Helpers and Superseded Python Readiness Tests; Reuse C# Library Listeners and C++ Product Registration.
- [x] Identify and Repair the Registration Hang Without Changing Its Guard.
  - [x] Identify External-Endpoint Stop During Host Shutdown as the Cause; Preserve the Registration-Across-Restart Contract and Its Existing Guard.
  - [x] Pin the Library's Ownership Fix and Pass the Failing Registration Restart and Remote Authoring Cases.
- [ ] Pass Generation Drift, Workspace Test Compilation With Desktop Testing, the Requested Focused Stories and SDK Selections, Native SDK Compilation, Frontend Build/Units, Formatting, Repository/Supply/Binding Gates and Strict OpenSpec Validation.
  - [x] Pass Generation Drift and Compile All Workspace Test Targets With Desktop Testing and All Rust SDK Consumer Test Targets.
  - [x] Pass the Two Selected CLI First-Hour Stories, Native-Interpreter Launch Round Trip, GUI Host Story and Desktop Acceptance Matrix.
  - [x] Pass Python Units, Host-Free Integration and the Three Launch/Registration Process Cases; Preserve Cancellation Cleanup Until the Listener Settles.
  - [x] Build the C# Debug Solution and Packaged Starter; Pass Six Client Registration/Execution Cases and Direct Child Launch, Announcement and Clean Stop.
  - [x] Compile the C++ Debug SDK Test Targets, Build the Frontend and Pass Fourteen Plugin-List Unit Cases.
  - [x] Pass Formatting, Repository, Supply, Unused-Dependency, Release-Context Binding and Strict Change Validation Gates.
  - [x] Pass Runtime Registration Restart and Rust Remote Authoring After the Pinned Library Stops Sending Stop to External Endpoints.
- [x] Prove the Created Python Starter's Dev Task With Candidate Product Bytes in External Temporary Storage.
- [x] Prove User-Owned Reattachment With the Pinned Library's Remote-Removal Ownership Fix.

## Host Launch Handshake Integration

- [x] Merge the Managed Launch Handshake and Self-Registration Cutover From Dev; Preserve the Host Plugin, Default Shipping, Environment Context and Effects Removal; Regenerate Protocol and Desktop Contracts and Refresh Tracked Cargo Locks.
- [x] Keep the Host Plugin on the SDK Single Run Entry, With Launch Credentials Selected by the SDK and No Readiness Output or Address Variables.
- [x] Collapse the SDK Registration Refusal Guard Reported by Rust Lint Without Changing Listener Cleanup or Structured Host Failure Propagation; Import Its Named Types Without Aliases.
- [x] Obtain a Passing Rust Lint Result for the Corrected Candidate; the Single Allowed Local Invocation Passed Without Findings.
- [x] Prove a Stable Ready Host Plugin Through the Product Launcher With Library-Owned Collections and Revisioned Durable Plugin State; Observe Ready and Collection Rows Without Sleeps or Native Controls.
- [x] Complete the Independent Native Windows Generation, Workspace Test Compilation, Desktop Bench Compilation, Host Plugin Tests, Native Packaging Tooling, C++ Test Compilation, Format, Repository, Supply and Strict OpenSpec Proof; Keep Linux, Corrected Rust Lint and Ready Launch Proof Independent.

## Host Collection and Journal Adoption

- [x] Merge Dev's Library Collection and Durable Plugin State Pin, Regenerating Tracked Contracts and Refreshing Cargo Locks.
- [x] Delete the Product Collection Type, Producer Scheduler and Superseded Harness Scheduler and Tests; Register Native Library Collections.
- [x] Bind Native Collection Invocations to Shared Author State, Host-Resolved Interval Settings and Demand-Owned Acquisition/Release Hooks.
- [x] Ingest Native Collection Members as Individual Scoped Rows Instead of Legacy Observer Column Batches.
- [x] Implement the Context Journal Store on Revisioned Plugin State and Re-Read Conflicts Within the Operation Policy.
- [x] Prove Fake-Store Conflict Preservation and the Shipped Plugin's Ready State With Consecutive Collection Rows Through the Product Launcher.
- [ ] Complete the Requested Generation, Compilation, Focused Tests, Native Packaging, Lint, Format, Repository, Supply, Binding and Strict Change Gates.
  - [x] Pass Generation Drift, Workspace Test-Target Compilation With Desktop Testing, Desktop Bench Compilation and Isolated Rust SDK Consumer Test-Target Compilation.
  - [x] Pass the Safe Host Integration Tests, Network Baseline Test, Launcher and Collection Storage Tests, Focused Rust SDK Tests and Native Packaging Tooling Tests.
  - [x] Pass the Single Rust Lint Invocation, Format, Repository, Supply, Unused-Dependency and Strict OpenSpec Checks.
  - [x] Build and Package the Pure Python SDK in the Release Context and Pass the Binding Gate After Preparing Its Existing Example Distributions.
  - [ ] Resolve the Required Whole-Package Host Test Versus the No-Native-CPU-Load Rule: `collectors::cpu::workers::tests::active_workers_count_whole_batches` Starts Real Load; Keep This Proof Open Until the Owner Chooses Its Test Scope.
