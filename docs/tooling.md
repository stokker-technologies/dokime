# Repository Tooling

Mise is the public entry point. Python modules in `tools/python/src/dokime_tools/`
own environment setup, synchronization, version metadata, binding generation,
frontend preparation, native packaging, CI and
test execution. The shared Python environment lives at repository-root `.venv/`.

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

## Focused Python Client Checks

`mise run test:python:unit --select <expression>` checks current client source in
the root environment without a staged wheel. A focused integration selection uses
`mise run test:python:integration -- -- -k <expression>` and also checks source.
Full Python unit and integration tasks retain their built-wheel input.

## Organization Asset Preparation

`dokime-organization` owns the organization pack materializer used by the runtime,
desktop startup and compiled frontend defaults. The Python organization task runs
`cargo run --locked -p dokime-organization --bin dokime-organization` against the
authored `organization/` folder. Cargo checks freshness on every invocation, so a
fresh checkout needs no previously built Dokime executable. This small target
depends on portable domain definitions and the CSS/image readers; it does not
build the host runtime, desktop, storage engine or report renderer.

The runtime retains organization overlay resolution, update policy, bundled
assets and atomic publication of runtime logo files. Desktop and report assets
use the same materialized logos and CSS; generated files remain checked in.
