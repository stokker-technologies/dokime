# Tooling Streamline Tasks

## 1. Contract

- [x] 1.1 Lock entry-point argument names and workflow matrices from `mise.toml`, the dispatcher and project-owned CI tasks.
- [x] 1.2 Lock `mise.toml`, `tools/python`, `.github/workflows`, setup action, contributor guide and tooling guide as the source inventory.

## 2. Tasks

- [x] 2.1 Expose language/layer dispatch through the entry points in `proposal.md`.
- [x] 2.2 Merge generation into `generate` and route organization/report preparation through build and package paths.
- [x] 2.3 Remove every `CARGO_INCREMENTAL` setting from `mise.toml` and tooling and set it once in the workflows.
- [ ] 2.4 Delete superseded language/layer task leaves and migrate their callers; retain only the agreed public entry points and necessary workflow transport operations.
- [ ] 2.5 Remove PyO3 wheel, binding and native fixture tasks and jobs when the pure Python SDK lands; replace them with pure-package proof, coordinated with `grpc-plugins` 4.3 and 5.3.

## 3. Tooling

- [x] 3.1 Adopt the shared build tooling without the storage preflight and delete the wrapper code that existed only for it.
- [x] 3.2 Delete the duplicated and dead modules in `tools/python` and keep one task dependency graph.
- [x] 3.3 Pin installable uv 0.13.7 in the task declaration and lockfile; remove reliance on a temporary tool-version override.

## 4. Workflows

- [x] 4.1 Rewrite the CI, Nightly and Release workflows so each job runs one task, per the shape task 1.1 records.
- [x] 4.2 Provision Python before native CI setup and guard Windows-only process and filesystem APIs by platform.

## 5. Final Validation

- [ ] 5.1 Run every entry point locally on Windows and Linux, and confirm hosted CI and Nightly run the same tasks.
- [x] 5.2 Update `docs/CONTRIBUTING.md` and the tooling guide.
- [ ] 5.3 Verify the Linux product build and repair platform-specific failures through the product build task; Windows proof does not close this task.
- [ ] 5.4 Reconcile machine Hub repository-check support for the entry-point model before final repository-check proof; keep Hub implementation outside this product.

## Workflow Follow-Up

- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- Run only focused tasks until combined source closure; final native proof remains separate.

## Build Measurements

- [x] Record the Windows development-build trials from 2026-11-12. The machine had 16 physical cores, 32 logical processors and 80 GB memory; other repositories compiled concurrently. Cold targets retained the shared compiler cache, and the linker change created a new fingerprint namespace. These single observations do not establish isolated speedups. Values below are seconds before/after bundled LLD and desktop freshness changes.

| Selection | Before (s) | After (s) |
| --- | ---: | ---: |
| Cold Domain Check | 139.87 | 134.18 |
| Cold CLI Check | 485.85 | 1039.17 |
| Cold CLI Build | 613.44 | 632.83 |
| Cold Tests Artifact | 1338.27 | 1517.80 |
| Leaf Check / Build / Test | 10.09 / 17.67 / 31.25 | 6.55 / 31.02 / 46.38 |
| Middle Check / Build / Test | 26.43 / 98.03 / 152.70 | 52.80 / 76.62 / 63.26 |
| Lowest Check / Build / Test | 84.98 / 75.08 / 80.97 | 108.87 / 135.39 / 93.27 |
| Unchanged Desktop Build | 67.85 | 7.24 |

- [x] Preserve the trial method and limits: CLI consumer loops made then restored one public-constant edit in CLI (leaf), runtime (middle) or domain (lowest), using focused CLI check/build and the exact CLI identity-file integration story with `--timings`. The runtime cold compile took 231 seconds (164 before code generation, 66 in code generation); domain compiled twice in 117/81 seconds for different dependency closures, and the runtime integration target took 290/272 seconds before/after. Tests compiled 45 binaries and 16 compile contracts. Alternating CLI final compile/link medians were 18.95 seconds with MSVC and 11.70 with LLD. Desktop embedded-file addition/removal rebuilt both native targets in 36.53/40.24 seconds, then an unchanged build took 7.89 seconds.
- [x] Preserve concurrency/cache observations: one/two/four concurrent macro public-API edits completed in 26/37/64 seconds, with four-way individual times of 36–64 seconds; drives, warmth and external load varied. Isolated cache probes restored all 14 cacheable registry dependencies in 18.85/15.66 seconds across drives; a warm-cache new-root CLI check still took 888 seconds. The shared 20 GB cache remained full. Procedural-macro DLL linker bytes differed despite identical Rust metadata, preventing whole-byte sccache 0.19 hits; upstream metadata hashing was not yet published. Deterministic-linker, build-dependency optimization and jobserver probes gave no consistent improvement. Vite took 1–3 seconds and generated TypeScript checking about 16 seconds. Linux, wheel/C#/C++ builds, branch switches and isolated full-graph concurrency were not measured; release/Nightly optimization stayed unchanged.
