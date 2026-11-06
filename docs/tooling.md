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

## CI Compiler Caches

Compiler caches are separated by OS, architecture, build profile and producer.
All use `SCCACHE_CACHE_SIZE=8G`; archive-only test consumers need no compiler cache.
The existing Product identities remain unchanged:

```text
Product: ci-v5-sccache-<os>-<arch>-<profile>-<hash(mise.lock)>-<hash(Cargo.lock)>
Tests:   ci-v5-sccache-<os>-<arch>-<profile>-tests-<hash(mise.lock)>-<hash(Cargo.lock)>
Bench:   ci-v5-sccache-<os>-<arch>-bench-bench-<hash(mise.lock)>-<hash(Cargo.lock)>
```

Each identity first restores its exact key, then its prefix ending before
`hash(Cargo.lock)`. Tests and Bench then fall back to the matching Product exact
key and prefix; Bench uses the release Product cache. This seeds new identities
while preserving test-only and benchmark-only compilation units in later runs.

Only the corresponding Build / Product, Build / Tests or Build / Bench job saves
its cache, after a successful build and only on an exact-key miss. PRs restore
only. Dev pushes save the dev Product and Tests caches; Nightly on dev runs in
`READ_WRITE` mode and saves release Product, release Tests and bench Bench caches.
`.github/actions/setup/run.py` resolves `CACHE_PLAN` using the build job's
`COMPILER_CACHE_KIND`; `.github/actions/save-caches` enforces the Tests and Bench
save policy. The workflows own producer selection.

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

## Workstation Builds

Compiling tasks run through `tools/python/src/dokime_tools/build/machine.py`, which starts the
shared compiler cache for the command. Nothing serializes compilation: tasks in
any repository or worktree run at the same time, and no worker or compiler
thread limits are added.

For non-CI runs, `mise.toml` selects `RUSTC_WRAPPER=sccache`,
`SCCACHE_DIR=<OS user cache>/sccache`, `SCCACHE_CACHE_SIZE=20G` and disables
incremental compilation. Each worktree keeps its own `target/`; matching
cacheable crate compilations can reuse results across worktrees. CI retains
its existing per-profile cache settings. sccache is already a pinned mise tool.
Local wrappers reuse the existing `ci/compiler.py` foreground-server helper with
an owned endpoint, so an older daemon cannot silently select another store.
Concurrent tasks each own a server on the same store. Only that owned server is
stopped after the command; each task prints its cache hit and miss counts.
If allocation failures recur, disable it for one run by setting
`RUSTC_WRAPPER` to an empty value:

```sh
RUSTC_WRAPPER= mise run build:rust
```

In PowerShell use `$env:RUSTC_WRAPPER = ''; mise run build:rust`, then remove
the override with `Remove-Item Env:RUSTC_WRAPPER`.

## Simulation Scratch Recovery

Simulation scratch stays in the system temporary directory. Each workspace is
published with a self-contained ownership marker before it receives source or
build outputs. A native lease outside the disposable directory protects the
owner through cleanup. Normal completion and failures preserve diagnostics in
`target/ci/simulation/` and remove owned scratch. The next simulation recovers
unlocked scratch after a crash, even when the original checkout's records are gone.

Legacy, unmarked, linked, malformed or live scratch is retained with its path and
an explanation. Inspect those paths and establish that their owner has stopped
before removing them manually. New versioned stable paths allow simulations to
start while unverifiable legacy directories remain. Never delete a native lease
file or redirect TEMP into the checkout.
