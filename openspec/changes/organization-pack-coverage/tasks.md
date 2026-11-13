# Organization Pack Coverage Tasks

## 1. Contract

- [x] 1.1 Lock the source inventory from the findings: first paint in `crates/dokime-gui/{index.html,report.html,src-tauri/src/app/organization.rs}` and `crates/dokime-runtime/src/report/{output.rs,render/cycle.rs,saved.rs}` plus the generated `crates/dokime-runtime/dist-report/report.html`; domain plugin compatibility and distribution identity consumers; interface labels in GUI Plugins, native picker filters and runtime traceability; distribution file names in GUI native secrets/logging, runtime exports and server logging/bundles; messages in runtime plugin admission, execution, storage and shutdown plus desktop startup and server host errors; report product and remote certificate subject; pack replacement in runtime organization resolution and packaging native assembly; remove desktop import bindings, Settings rows and organization navigation, archive/promotion helpers and their tests; migrate organization tests and update `docs/{product-guide,gui,privacy}.md` and affected README claims. Findings D/I remain toolchain contracts, E executable names and F distribution identity remain unchanged, G/J contract identifiers and upstream attribution remain unchanged. Ownership excludes remote-access implementation except picker display labels and certificate subject. Targeted proof covers changed crates, including narrowing two integration-only telemetry fixture helper gates in `storage/sqlite/data/telemetry/{flush,writer}.rs` so warning-denied unit builds remain available; task 4.1 belongs to the orchestrator.

## 2. Operator Surfaces

- [x] 2.1 Produce the initial desktop and report title and icon from the resolved or captured appearance; replace the fallback report title.
- [x] 2.2 Read the label for the traceability version row, plugin compatibility text and file picker filters.
- [x] 2.3 Derive suggested diagnostics, support bundle, secrets and log file names from the distribution identifier, changing writers and readers together.
- [x] 2.4 Pass the resolved label into operator-facing error and status messages, with the distribution default before the pack loads.
- [x] 2.5 Set the product name in report data from the captured appearance and issue new remote access certificates under the label.

## 3. One Pack Path

- [x] 3.1 Make a pack supplied at packaging replace the bundled default.
- [x] 3.2 Delete pack import, the pending and active packs in the application folder, their overlay resolution and the Settings surface for them.

## 4. Final Validation

- [ ] 4.1 Run focused packaged desktop, report and packaging proof with a supplied pack and with the default appearance; Linux and packaged desktop stories remain unproven.
- [x] 4.2 Repair the installer sidebar fixture to create its optional installer table explicitly.
- [x] 4.3 Update the remote log-stream fixture and desktop distribution story to use distribution-derived log names.
- [x] 4.4 Replace the removed runtime pack-import journey with installed-pack loading, ignored application-folder packs and retained update-policy coverage; remove Organization navigation from the settings journey.
- [x] 4.5 Run the Windows merge-and-repair focused Rust tests, package checks, settings unit tests and Rust check.

## 5. Documentation

- [x] 5.1 Update the product guide's organization section and affected GUI, privacy and README claims for installer-only appearance.

## Workflow Follow-Up

- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- Run only focused tasks until combined source closure; final native proof remains separate.
