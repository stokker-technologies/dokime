# Repository Tooling

Mise is the public entry point. Python modules in `tools/python/src/dokime_tools/`
own environment setup, synchronization, version metadata, binding generation,
frontend preparation, native packaging, CI and
test execution. The shared Python environment lives at repository-root `.venv/`.

After changing Python dependencies, `mise run prepare --refresh-lock` refreshes
the workspace `uv.lock` before the ordinary locked synchronization. The test group
includes both optional dataframe libraries; focused tabular tests also block their
imports to verify dependency-free iteration and missing-extra errors.

The desktop GUI owns its web tooling under `crates/dokime-gui/`: Biome, the
pnpm lock and the RPC generator in its `tools/`. Rust
performance workloads live under the owning crate's `benches/`.

Desktop development process trees are owned by the task-built `cargo-dokime`
executable with the application's validated packaging settings. `mise run dev`
selects that executable from its existing Cargo compiler record. The unchanged
`tauri:dev` and `build:report:watch` package commands select the development
artifact from `mise run build:rust` by default; `--build-json <record>` selects
another successful task build. Missing artifacts fail explicitly; these
launchers never build or install a supervisor. `tauri:dev --print-config`
continues to inspect frontend launch configuration without an artifact.
`operational.packaging.process_stop_ms` bounds each cleanup stage separately:
Unix gets one graceful-wait budget and one forced-reap budget, up to twice the
configured duration; Windows uses only the forced-reap budget. Surviving owned
descendants are terminated even when their leader already exited.

Each Python package carries its own `[tool.ty]` configuration. `lint:python`
type-checks the repository tooling with `ty check --project tools/python`;
`check:python:generated` type-checks the client package after the build has
verified its generated modules.

`mise run build:rust:tests` packages the host and OBC fixture bundles from the
Tests compiler output under `target/build/<profile>/<target>/fixtures/plugins/`.
Rust stories resolve those fixtures from the Tests export, so the archive owns
the bundles its integration lane requires.

## Focused Python Client Checks

`mise run test:python:unit --select <expression>` checks current client source in
the root environment without a staged wheel. A focused integration selection uses
`mise run test:python:integration --select <expression>` and also checks source.
Full Python unit and integration tasks retain their built-wheel input.

## Hosted-Runner Test Budgets

Test-time budgets are doubled so hosted runners, whose disks can stall for over a
minute, finish stories reliably on Linux and Windows. `[conventions.testing.deadlines]`
in `tools/project.toml` sets each lane's per-story deadline, which the test runner
passes to nextest, pytest and Vitest. Rust fixture operations and in-story waits
share `STORY_BOUND` and `FIXTURE_POLICY` in
`crates/dokime-runtime/tests/support/settings.rs`; the desktop stories size their
waits with the constants at the top of `crates/dokime-gui/e2e/native-desktop.mjs`.
Product operation policies keep their own deadlines.
