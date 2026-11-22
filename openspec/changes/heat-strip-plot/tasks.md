# Heat Strip Tasks

## 1. Agreement and Source Inventory

- [x] 1.0 Author proposal, design, spec scenarios and task inventory; `openspec validate heat-strip-plot --strict` passes.
- [x] 1.1 Agree proposal, design, scenarios and inventory; owner activates `heat-strip-plot` on `feat/heat-strip-plot` using `stkr specification start`; verify the marker before implementation.
- [x] 1.2 Confirm required scope includes protocol builders/validation, snapshot proto, plot/document projection/types/grouping/registration and tests outside the initially named catalog folder; verify no concurrent ownership conflicts. The continuation grants live-data.ts request mapping and its focused tests. Exclude Watch cadence, docs, manifests, locks and CI.

## 2. Protocol and Authoring

- [x] 2.1 Extend `proto/dokime/model/v1/plots.proto` and `proto/dokime/host/v1/plots/snapshots.proto` with the kind, cell/facet expressions and reused reduction enum; regenerate tracked Rust/Python/C#/C++/TypeScript outputs only through `mise run generate`; prove generation drift and unchanged reserved tags.
- [x] 2.2 Extend `crates/dokime-protocol/src/builders.rs` and `src/model/plots.rs` validation, consumed by `sdk/rust/src/`; prove Rust document round-trip and invalid combinations in existing protocol/SDK tests; report SDK documentation impact.
- [x] 2.3 Extend `sdk/python/src/dokime_sdk/templates/{_plots.py,__init__.py}` and existing unit/integration authoring tests; prove default LAST and explicit reducer/cell/facet wire round-trip; report SDK documentation impact.
- [x] 2.4 Extend `sdk/csharp/src/Dokime/Authoring/Plots.cs` and `sdk/cpp/include/dokime/plots.hpp` with existing builder tests; prove equivalent serialized declarations, without adding a parallel authoring family.
- [x] 2.5 Extend one existing plot-demonstrating SDK example using already validated identities and expressions; prove its description/round-trip test and review README claims without editing unaffected content.

- [x] 2.6 Delete duplicate protocol `GridDefinition`, regenerate bindings and migrate request mapping, exports and storage/tests to `GridChart`.

## 3. Storage and Operations

- [x] 3.1 Extend `crates/dokime-storage/src/plot_selection.rs` and `plot_kinds.rs` and metadata-only source bounds through `expressions.rs` using existing projection and library accumulators; prove one grouped read, LAST/MAX/MEAN/COUNT, absence, typed cell identity, facets, deterministic ties, filters/cuts and budget refusal with focused deterministic tests.
- [x] 3.2 Map cell/facet in `crates/dokime-operations/src/host/documents/plot.rs` and add kind dispatch only in `src/host/plots/snapshots/list.rs`; prove request mapping and a Heat Strip snapshot including contribution times with focused tests. Leave snapshot Watch files untouched.
- [x] 3.3 Verify existing report capture obtains whole-execution Heat Strip snapshots at the sealed cut; change only required capture dispatch if needed after ownership confirmation; prove parity independently of live window and later evidence. Report query/architecture documentation impact.

- [x] 3.4 Implement owner-approved numeric ordering in Dokime storage for mixed Float/integer/unsigned/Exact CELL values, without f64 identity loss; prove fractions, large unsigned values, signs and numeric ties.

## 4. Shared Rendering

- [x] 4.1 Add `packages/plots/src/catalog/HeatStripPlot.tsx`, extend `components/document/PlotGroupGrid.tsx`, `grouping.ts`, `lib/types/plot.ts` and `lib/api/native-plot-projection.ts` only where required by kind dispatch; prove numeric/text ordering, facets, shared scale/legend, equal square geometry, empty slots and hover semantics in package unit/integration tests without pixel/prose assertions.
- [x] 4.2 Extend kind/expression projection in `packages/document/src/lib/api/native-documents.ts`; verify `packages/report/src/native-report.ts` consumes Heat Strip with the same renderer, changing report consumers only if required. Prove frozen capture decoding/rendering in package tests and report GUI documentation impact.

- [x] 4.3 Complete desktop request mapping of cell/facet and LAST/COUNT in `crates/dokime-gui/src/plots/live-data.ts`; prove it in `tests/unit/plots/plot-pipeline.test.ts` and desktop typecheck. Preserve cadence.

## 5. Final Proof and Handoff

- [x] 5.1 After every implementation item closes, run focused `mise run test` commands for touched crates and TypeScript packages; run Python SDK install and unit/integration tasks once. Proof: storage 6, protocol 11 (generation check), snapshot 1, Rust SDK 1, migrated catalog 1; TypeScript 31; Python unit 45, integration 49/52 initially plus all 3 failed tests passing individually after setting `DOKIME_TEST_HOST` to the task-built `target/debug/dokime-host.exe`. REAL validation setup: the new TypeScript fixture lacked required fields and Python had no prepared host path; both corrected without changing checks. No SUPERSEDED or REWORK test failures.
- [x] 5.2 Compile every touched package test target and all workspace test targets through the project task with desktop `testing` when required; use dev/test profiles and `CARGO_BUILD_JOBS=6`, worktree target and a session-owned build-drive watcher. The continuation reran `mise run test -- --changed Cargo.toml --no-run`: all 15 workspace packages compiled with `dokime-gui/testing` (exit 0). No lint/clippy, benches or whole test lanes.
- [x] 5.3 Run `mise run generate -- --check`, desktop `pnpm exec tsc --noEmit`, TypeScript lint, `mise run format`, repository check and `openspec validate heat-strip-plot --strict`; all exit 0 using discovered task equivalents. Desktop typecheck caught the incomplete new test fixture, then passed after its correction; rerun only this failed gate. Generation includes 11 protocol integration tests. TypeScript lint reports warnings only; no suppressed checks.
- [x] 5.4 Run the required bindings gate and Python build/package gates only with their expressly required profiles; resolve any conflict with the general no-release-build rule before starting them. Do not alter manifests or locks.
- [x] 5.5 Commit explicit owned paths in coherent Conventional Commits; hand back local SHAs, actual proof, remaining work, DOCS and OVERLAP findings in at most ten lines. Never push; leave final proof items open until performed.

## Workflow Follow-Up

- The owner documents the newly exported `dokime::builders` authoring surface in `docs/sdks/rust.md:1`; existing README claims remain valid.
- The owner updates `docs/guides/query-api.md:22`, `docs/guides/gui.md:459` and `docs/project/architecture.md:646` for Heat Strip and corrects stale report ownership at architecture lines 653-655 and GUI line 8.
- Owner approved deleting `GridDefinition` and migrating every snapshot consumer to `GridChart`.
- The owner integrates this complete feature with concurrent work and owns delivery authorization, hosted platform validation and OpenSpec retirement.

- Owner approved consolidating Bar/Pie and Heat Strip grouping in `crates/dokime-storage/src/plot_kinds.rs`; delete the old category loop and preserve each presentation's absence/ordering rules.
- Owner approved one colour-domain owner in `packages/plots/src/colors.ts`, consumed by `catalog/{GridChartPlot,ContourPlot,HeatStripPlot}.tsx`; delete local loops and prove domain edge cases.
- Owner approved deleting raw `Plot::new/line/unit` in `crates/dokime-protocol/src/model/plots.rs` and migrating the only remaining callers in `plugins/sidecars/obc/src/{suites/benchmark/plots,streams/measurement}.rs` to `builders.rs`; compile their test targets.
- [x] 6.1 Prove and close the approved storage grouping, colour-domain and raw-builder consolidations; no duplicate mechanism remains.
