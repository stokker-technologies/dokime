# Heat Strip Tasks

## 1. Agreement and Source Inventory

- [x] 1.0 Author proposal, design, spec scenarios and task inventory; `openspec validate heat-strip-plot --strict` passes.
- [ ] 1.1 Agree proposal, design, scenarios and inventory; owner activates `heat-strip-plot` on `feat/heat-strip-plot` using `stkr specification start`; verify the marker before implementation.
- [ ] 1.2 Confirm required scope includes protocol builders/validation, snapshot proto, plot/document projection/types/grouping/registration and tests outside the initially named catalog folder; verify no concurrent ownership conflicts. Exclude Watch cadence, live-data.ts, docs, manifests, locks and CI.

## 2. Protocol and Authoring

- [ ] 2.1 Extend `proto/dokime/model/v1/plots.proto` and `proto/dokime/host/v1/plots/snapshots.proto` with the kind, cell/facet expressions and reused reduction enum; regenerate tracked Rust/Python/C#/C++/TypeScript outputs only through `mise run generate`; prove generation drift and unchanged reserved tags.
- [ ] 2.2 Extend `crates/dokime-protocol/src/builders.rs` and `src/model/plots.rs` validation, consumed by `sdk/rust/src/`; prove Rust document round-trip and invalid combinations in existing protocol/SDK tests; report SDK documentation impact.
- [ ] 2.3 Extend `sdk/python/src/dokime_sdk/templates/{_plots.py,__init__.py}` and existing unit/integration authoring tests; prove default LAST and explicit reducer/cell/facet wire round-trip; report SDK documentation impact.
- [ ] 2.4 Extend `sdk/csharp/src/Dokime/Authoring/Plots.cs` and `sdk/cpp/include/dokime/plots.hpp` with existing builder tests; prove equivalent serialized declarations, without adding a parallel authoring family.
- [ ] 2.5 Extend one existing plot-demonstrating SDK example using already validated identities and expressions; prove its description/round-trip test and review README claims without editing unaffected content.

## 3. Storage and Operations

- [ ] 3.1 Extend `crates/dokime-storage/src/plot_selection.rs` and `plot_kinds.rs` using existing projection and library accumulators; prove one grouped read, LAST/MAX/MEAN/COUNT, absence, typed cell identity, facets, deterministic ties, filters/cuts and budget refusal with focused deterministic tests.
- [ ] 3.2 Map cell/facet in `crates/dokime-operations/src/host/documents/plot.rs` and add kind dispatch only in `src/host/plots/snapshots/list.rs`; prove request mapping and a Heat Strip snapshot including contribution times with focused tests. Leave snapshot Watch files untouched.
- [ ] 3.3 Verify existing report capture obtains whole-execution Heat Strip snapshots at the sealed cut; change only required capture dispatch if needed after ownership confirmation; prove parity independently of live window and later evidence. Report query/architecture documentation impact.

## 4. Shared Rendering

- [ ] 4.1 Add `packages/plots/src/catalog/HeatStripPlot.tsx`, extend `components/document/PlotGroupGrid.tsx`, `grouping.ts`, `lib/types/plot.ts` and `lib/api/native-plot-projection.ts` only where required by kind dispatch; prove numeric/text ordering, facets, shared scale/legend, equal square geometry, empty slots and hover semantics in package unit/integration tests without pixel/prose assertions.
- [ ] 4.2 Extend kind/expression projection in `packages/document/src/lib/api/native-documents.ts`; verify `packages/report/src/native-report.ts` consumes Heat Strip with the same renderer, changing report consumers only if required. Prove frozen capture decoding/rendering in package tests and report GUI documentation impact.

## 5. Final Proof and Handoff

- [ ] 5.1 After every implementation item closes, run focused `mise run test` commands for touched crates and TypeScript packages; run Python SDK install and unit/integration tasks once. Record actual test counts and failures classified REAL, SUPERSEDED or REWORK.
- [ ] 5.2 Compile every touched package test target and all workspace test targets through the project task with desktop `testing` when required; use dev/test profiles and `CARGO_BUILD_JOBS=6`, worktree target and a session-owned build-drive watcher. No lint/clippy, benches or whole test lanes.
- [ ] 5.3 Run `mise run generate -- --check`, desktop `pnpm exec tsc --noEmit`, TypeScript lint, `mise run format`, repository check and `openspec validate heat-strip-plot --strict`; use the discovered task equivalents and run each slow gate once after source closure.
- [ ] 5.4 Run the required bindings gate and Python build/package gates only with their expressly required profiles; resolve any conflict with the general no-release-build rule before starting them. Do not alter manifests or locks.
- [ ] 5.5 Commit explicit owned paths in coherent Conventional Commits; hand back local SHAs, actual proof, remaining work, DOCS and OVERLAP findings in at most ten lines. Never push; leave final proof items open until performed.

## Workflow Follow-Up

- The owner updates `docs/guides/query-api.md:22`, `docs/guides/gui.md:459` and `docs/project/architecture.md:646` for Heat Strip and corrects stale report ownership at architecture lines 653-655 and GUI line 8.
- The owner decides whether to remove the pre-existing `GridDefinition`/`GridChart` duplication; this change does not remove either mechanism.
- The owner integrates this complete feature with concurrent work and owns delivery authorization, hosted platform validation and OpenSpec retirement.
