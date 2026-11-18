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
- [x] 4.3 Replace PyO3 with the pure Python authoring/client package, explicit close and shipped examples.
- [x] 4.4 Provide C# authoring/client APIs, generated roots, examples and scaffold output; retain tracked generated protocol output.
- [x] 4.5 Provide C++ authoring/client APIs, generated roots, examples and scaffold output; retain tracked generated protocol output.
- [ ] 4.6 Re-author the host telemetry plugin with its collectors and controls; only the OBC plugin is present under `plugins/sidecars/`.
- [x] 4.6.1 Re-author the OBC device operations, suites, documents and independent benchmark workloads on the Rust SDK; remove plugin analysis registration.
- [x] 4.7 Bundle executable plugins and provide shared Rust/Python/C#/C++ thermal host stories and language fixture producers.
- [x] 4.8 Restore desktop workspace membership, runtime embedding, generated client routing, native preference export and shipping build ownership.
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
- [ ] 5.13 Implement retained Effects Create/Get/Set and prove inverse retention, owner-qualified confirmations, reverse restoration and failed-startup restoration; all three handlers currently return Unimplemented.
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
- [ ] Python: Resolve the Root Dispatcher's Directory-Derived Unit Lane Against the Declared Integration/System Lane Inventory Before Root Unit Dispatch Can Run.
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

- [x] (a) Rename the Python Package Alone per the SDK Surface Specification Ownership Inventory; Regenerate Imports, Wheel Paths and Guards Without Keeping `dokime` Aliases.
- [ ] (b) Implement the SDK Surface Specification's Python Authoring Classes, Separate Contexts, `ctx.plugin` and `plugin.serve()`; Resolve Its Named Capability Decisions Before Their Dependent Work.
  - [x] Bind Explicit Plugin, Suite and TestTemplate Classes, Derived Identities and Labels, Annotated Plain Operations and Separate Contexts; Move Documents, Plots and Parameters to Templates and Delete Replaced Authoring Entry Points.
  - [x] Prove Fixed Builder-Description Wire Equivalence, Scoped Template State, Primitive Records and Operations, and Cleanup After Failed or Cancelled Template Setup With Python-Only Tests.
  - [ ] Resolve Carrier Activation Contexts and Stream Subscription Support; Prove Plugin initialize/finalize Order, Failed/Cancelled Initialization Finalization, Admission Draining and Repeated Shutdown.
  - [x] Share Template/Client Record Query and Decode Ownership, Termination Values, Bound Operation Calls, Plugin-Owned Listener Activation and Artifact Publication; Complete Typed Statistics, Artifact Projection and Local Document Declaration Binding.
  - [ ] Admit Plugin Notes/Create and Resolve Complete Automatic Peer Requirements Against the Library's Frozen Activation Requirements; Authors Declare No Requirements.
  - [ ] Complete Installed-Wheel Stub/Public-Home Consumer Proof After Item (e) Updates Its Inventories, Then Run the Required Host Stories on the Combined Candidate.
- [ ] (c) Implement the SDK Surface Specification's Python Bound Handles, Host Compositions and Configuration Precedence/Grammar With Secret Storage, Readback, Controls, Seal, Evidence, Notes, Artifacts and Explicit Close.
  - [x] Replace Owned Selector/Export Wrappers With Bound Client Resources, Current Host Compositions, Immutable Product Values, Structured Failures and Explicit Owned Registration Cleanup.
  - [x] Implement Working-Directory Configuration Precedence and the Shared Dotenv Grammar Without Dependencies or Environment Mutation; Preserve Ordered Pairing Attempts and Connection Budgets.
  - [x] Share Verified Archive/Report Downloads, Preserve Acknowledged Upload Fences and Child Request Identities, and Prove Frozen Decoding, Fixed-Cut Queries, Controls, Notes, Artifacts and Cleanup With Host-Free Services.
  - [x] Bind Host Execution Seal Outcomes and Sealed Progress; Prove Lazy Run/Watch Finality and Observation Close Without Execution Cancellation With Host-Free Services.
  - [ ] Bind Managed Bundle Installation and Client Invocation Policy Overrides After Their Host Contracts Land; Complete Real-Host Retained Archive Proof Without Substituting Readback for Seal.
  - [x] Resolve Shared Query, Termination, Bound Operation, Artifact Publisher and Listener Activation Ownership With Item (b).
  - [ ] Complete Installed-Wheel Stub/Public-Surface and Consumer Proof After Item (e) Migrates Its Inventories.
- [ ] (d) Implement the Exact Manifest Schema, Dynamic Metadata, Implicit Assets, Bundle Identity/Version and Four-Language `dokime new` Embedding per the SDK Surface Specification.
  - [x] Replace Top-Level Manifest Scripts and Assets Lists With Strict Plugin, Compatibility and Scripts Tables; Resolve Native Metadata With Static Overrides.
  - [x] Bundle Implicit Assets, Referenced Metadata and Required Launch Inputs; Preserve User Locks and Exclude Private Inputs.
  - [x] Migrate the Eight Starter Manifests and Shared Command-Line Fixture Consumers While Preserving Current Entrypoints and Build Commands.
  - [x] Supply Rust, C# and C++ Starter Native Metadata and Icons, Stage C# Publish Companions Under Assets and Migrate Runtime Manifest Consumers.
  - [ ] Reconcile the Python Starter Source Inventory With Its Agreed Manifest Roots and Supply Native Metadata and Icons for That Inventory.
  - [ ] Prove Installed Resolved Versions After the Library Plugin Manager Retains the Frozen Bundle Version Across Activation and Replacement.
  - [ ] Complete Packaging and CLI Units, Created-Starter Bundle/Inspect/Dev Stories, Focused Compilation, Repository Checks and Package Lint.
- [ ] (e) Migrate Python Starters, Examples, Generation, Public/Stub Guards, Tests and Stories to the Reference Project Shape; Ship No Lock, Root Entry Scripts, Argument Parsing or Old Imports.
- [ ] (f) Implement the SDK Surface Specification's Rust SDK and Two-Binary Starters With Owned Generation, Focused Proof and No Superseded Client or Authoring Paths.
- [ ] (g) Implement the SDK Surface Specification's C# SDK and Shared-Library/Plugin/Client Starters With Owned Generation, Focused Proof and No Superseded Client or Authoring Paths.
- [ ] (h) Implement the SDK Surface Specification's C++ SDK and Two-Binary Starters With Owned Generation, Focused Proof and No Superseded Client or Authoring Paths.
- [ ] Replace Python `plugins.create` With `plugins.register` and `plugins.upload`, With Readiness Owned by the Host and Registered Plugins Unregistered on Client Close.
- [ ] Prove End to End That the Created Starter's Client Flow Runs Against a Host and Passes the Type Checker.
