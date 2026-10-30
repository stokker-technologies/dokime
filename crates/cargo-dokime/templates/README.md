# {{package}}

A native Dokime plugin driving a thermal bench. The bench is simulated, so this
runs on any machine; `src/bench.rs` is the one file that pretends, and the first
one to replace with a real driver.

```text
src/bench.rs                       the simulated sensor and heater
src/error.rs                       the one error every boundary answers with
src/lib.rs                         the plugin, its contributions, and the handles opened once
src/domain/model/chamber/          what the chamber answers, and its temperature row
src/domain/model/prompts/          the one question it asks an operator
src/domain/model/settings/         the one settings section an operator edits
src/domain/operations/chamber/     the chamber requests a template sends
src/domain/operations/status/      the request for what the bench reports about itself
src/domain/operations/             the catalog of operations the plugin provides
src/operations/chamber/            reads the chamber; the one road a template has to it
src/operations/status/             answers the bench status request
src/listeners/unit_connected.rs    what the bench hears between runs
src/observers/temperature.rs       the observer that measures the chamber
src/suites/soak/hold.rs            the procedure: steps, samples, criteria, cleanup
tests/soak.rs                      the whole procedure, run in process
mise.toml                          the tasks every CI job runs
.github/workflows/, .gitlab-ci.yml CI for either provider
```

`src/domain/` holds what the plugin declares, by kind first and domain second:
`model/` for messages, rows, prompts and settings, `operations/` for the
requests it serves. Their bodies live under `src/operations/`, one folder per
domain with the same name.

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

Each request is its operation: a `#[dokime::operation]` struct under
`src/domain/operations/` carries an explicit wire id, its response, a policy
and the `resources = [Bench]` it reads. Its body is `impl Operation for
ReadChamber`, whose `run(self, ctx)` consumes the request and reaches the bench
with `ctx.resource::<Bench>()`; it holds no state between calls. Empty requests
are empty structs. `Lifecycle::prepare(config, setup)` registers the shared bench
with `setup.resource(...)` and prepares suite resources; `initialize` activates
resources that need the running runtime. Observers read the bench through
`ctx.resource::<Bench>()`. A template names the operations it calls with `requires = Required`
and invokes `ctx.call::<ReadChamber>(ReadChamber {}).await?`; the call is
routed through the host. Adding an operation means a request variant in
`domain::operations::Operations` and its `impl Operation`.

The SDK and `dokime-host` definitions use the same Dokime release tag. Install the host telemetry bundle before loading this template, which subscribes to `dokime_host::System`; missing publishers are refused per stream.

Install `cargo-dokime` and the `dokime` host, then run from this directory:

```sh
cargo dokime sidecar dev{{selection}}
```

Select **Thermal Soak → Hold at Temperature** in the development host and
execute the case. Turn the **Ask the operator to sign off** parameter on to see
the checkpoint prompt. Keep dev running while editing: successful builds cancel
active plugin work and reload the plugin while the browser stays connected.
Click **Re-run**, or start dev with `--rerun` to repeat your latest case after
each reload. Type `r` and Enter to retry, `q` and Enter to quit. Dev launches
the installed host through the `dokime` CLI, so `dokime` must be on `PATH`
(or passed with `--dokime`).

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
CI installs the locked dependencies and tools. Build and Package install the pinned `cargo-dokime` raw executable from Dokime's official GitHub release assets, selecting the host platform; the executable is not a crates.io package. `mise run check` verifies source and formatting; Build runs configuration-matched Clippy before compiling. `mise run build` compiles the plugin and freezes its catalog
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
