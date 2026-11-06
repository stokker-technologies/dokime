# {{package}}

A native Dokime plugin driving a thermal bench. The bench is simulated, so this
runs on any machine; `src/bench.rs` is the one file that pretends, and the first
one to replace with a real driver.

```text
src/bench.rs                       the simulated sensor and heater
src/error.rs                       template and evidence errors
src/lib.rs                         the plugin, contributions and resource ownership
src/domain/model/chamber/          the shared temperature row and host stream contract
src/domain/model/prompts/          the one question it asks an operator
src/domain/model/settings/         the one settings section an operator edits
src/domain/model/soak.rs           shared summary data
src/operations/mod.rs             the catalog of operations the plugin provides
src/operations/chamber/<name>.rs  each chamber request, response, policy and handler
src/operations/status/get.rs      the bench status request, policy and handler
src/operations/soak/encode_summary.rs JSON encoding behind an operation
src/listeners/unit_connected.rs    what the bench hears between runs
src/observers/temperature.rs       the observer that measures the chamber
src/suites/soak/hold.rs            the procedure: steps, samples, criteria, cleanup
tests/soak.rs                      the whole procedure, run in process
mise.toml                          the tasks every CI job runs
.github/workflows/, .gitlab-ci.yml CI for either provider
```

`src/domain/` holds shared data. Each `src/operations/<domain>/<operation>.rs`
keeps its request, response, policy, handler and private helpers together.
Templates and observers use operations, shared domain data, Dokime and std;
driver and library work stays behind operations. Keep calls acyclic and directed
from higher-level operations to lower-level ones. Keep operations thin and fast;
return batches when an observer's acquisition rate needs them. Requests and
responses never expose driver types; wrap results in plugin-owned serializable
responses and convert driver errors to `Refusal`. Operations shared with other
plugins gate their handler and driver imports with `runtime` in the same file.

Each contribution belongs to a named `#[dokime::catalog]` enum in its owning
module. The plugin id is `{{crate}}`, the crate name: Harmos serves each operation
as `<plugin id>.<operation id>`, so the id is one segment of lowercase letters,
digits and `_`, and every declared id here starts with it. `src/lib.rs` composes
these on `#[dokime::plugin]`: `const CATALOGS` in `impl dokime::Plugin`
names the executable Harmos operation catalogs it serves, and `contributions(...)`
lists the suites, records and listeners, evidence streams and settings it adds
to the host. Public `Streams` and `Records` catalogs in `lib.rs` register
process-wide outputs exactly once. Each
observer file defines its own public `Streams` and `Records` catalogs as permitted
subsets, keeping empty catalogs explicit. Demand observers run while any declared
stream or record has an active subscriber; registration alone does not start them.

Each request is its operation: a `#[dokime::operation]` struct carries an
explicit wire id, its response, a policy and the `resources = [Bench]` it reads.
The plugin owns the sensor and heater in its shared `Bench`; templates retain
only procedure state. `Lifecycle::prepare(config, setup)` registers that same
state with `setup.resource(...)`, and handlers retrieve it with
`ctx.resource::<Bench>()`. Plugin cleanup releases remaining handles.
Observers read through operations, and template cleanup calls `ReleaseChamber`.
A template lists its calls in `requires = Required` and sequences them as:

```rust
// Read Chamber Temperature
let operation = ReadChamber {};
let reading = ctx.call(operation).await?;
```

Steps may use returned values for records and criteria. Add each operation to
`operations::Operations`; do not split its declaration into `domain/operations/`.

The optional `cloud_token` setting reads the shared Secret Storage identifier
`cloud_token` directly. Add it in Settings → Secret Storage and retry
the plugin. A bare `Secret` field is required and refuses startup when
missing; `Option<Secret>` permits absence. Secret fields have no saved bindings,
environment sources or defaults.

Workspace Clippy lints deny `todo!` and `unimplemented!`. When generated inside an
existing Cargo workspace, the starter inherits its lints and adds these defaults
where they are missing.

The SDK and `plugin-host` definitions use the same Dokime release tag. Install the host telemetry bundle before loading this template, which subscribes to `plugin_host::System`; missing publishers are refused per stream.

Run `mise install`, then run from this directory:

```sh
cargo dokime sidecar dev{{selection}}
```

Select **Thermal Soak → Hold at Temperature** in the development host and
execute the case. Turn the **Ask the Operator to Sign Off** parameter on to see
the checkpoint prompt. Keep dev running while editing: successful builds cancel
active plugin work and reload the plugin while the browser stays connected.
Click **Re-run**, or start dev with `--rerun` to repeat your latest case after
each reload. Type `r` and Enter to retry, `q` and Enter to quit. Dev launches
the host through the `dokime` CLI that `mise.toml` pins (or one passed with
`--dokime`); without `--headless`, it opens the installed Dokime desktop GUI.

```sh
cargo test
```

runs `tests/soak.rs`, which drives the template through
`dokime::plugin::testing::Harness`: parameters, settings and secrets as builder
calls, the awaited record filed the way the host files it, and a typed report of
the verdict, the samples, the records, the traceability and the evidence files.
No host and no hardware are involved.

To create the distributable bundle:

```sh
cargo dokime sidecar build{{selection}}
```

Ordinary `sidecar build` compiles and packages the plugin. Advanced build
systems can collect existing compiler JSON, then package those exact bytes:

```sh
cargo dokime sidecar build{{selection}} --from-cargo-json compiler.json --metadata-json metadata.json --json > collected.json
cargo dokime sidecar package --build-json collected.json --output dist
```

The collection path and `sidecar package` do not compile the plugin.

The SDK tracks Dokime’s `dev` branch during development. Generation is offline;
the first build resolves the SDK and Rust dependencies. Commit `Cargo.lock`
to retain the resolved revisions.

Run `cargo generate-lockfile` and `mise install` once, then commit both lockfiles:
CI installs the locked dependencies and tools. `mise.toml` pins `cargo-dokime` and the `dokime` CLI to the Dokime release that generated this project, from Dokime's official GitHub release assets for the host platform; neither is a crates.io package. Set both versions to `nightly` to follow the nightly prerelease. `mise run check` verifies source and formatting; Build runs configuration-matched Clippy before compiling. `mise run build` compiles the plugin and freezes its catalog
under `target/build/<profile>/<target>/plugins/{{package}}.dokime`.
Standalone development builds retain line tables with separate debug information,
matching Dokime's development profile while keeping the packaged executable bounded.
Workspace members inherit their workspace root's profiles; configure the same
development settings there rather than adding ignored member profiles.
`mise run build:rust:tests` prepares the shared test archive; `mise run test:rust:unit`
and `mise run test:rust:integration` consume it.

`mise run package` copies and inspects that exact built bundle under
`target/package/<profile>/<target>/rust/plugins/`, without compiling. Profiles
default to `dev`; use `--profile release` for release outputs. The CI workflows
run these same tasks. Before publishing, configure this plugin repository's
native `[release]` and `[version]` policy in `.stkr/project.toml`, including its provider,
targets and bundle assets, and enable the required protected checks. Publication
uses that policy and refuses an unconfigured release.
