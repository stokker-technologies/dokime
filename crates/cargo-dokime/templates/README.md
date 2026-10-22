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
requests it serves. The handlers live under `src/operations/`, one folder per
domain with the same name, and are bound through `operations::Implemented`.

Each contribution belongs to a named `#[dokime_runtime::catalog]` enum in its owning
module. `src/lib.rs` composes these on `#[dokime_runtime::plugin]`: `provides =
domain::operations::Operations` names the operations it serves, and `contributions(...)`
lists the suites, records and listeners, evidence streams and settings it adds
to the host. Public `Streams` and `Records` catalogs in `lib.rs` register
process-wide outputs exactly once. Each
observer file defines its own public `Streams` and `Records` catalogs as permitted
subsets, keeping empty catalogs explicit. Demand observers run while any declared
stream or record has an active subscriber; registration alone does not start them.

Each request is its operation: a `#[dokime_runtime::operation]` struct under
`src/domain/operations/` carries an explicit wire id, its response, a version
and a policy. Its handler is a bare `#[dokime_runtime::operation]` on an
inherent impl whose `run` borrows that request. Empty requests are empty
structs. A template names the operations it calls with `requires = Required`
and invokes `ctx.call::<ReadChamber>(ReadChamber {}).await?`; the call is
routed through the host. Adding an operation means a request variant in
`domain::operations::Operations` and a handler variant in
`operations::Implemented`.

Install `cargo-dokime` and the `dokime` host, then run from this directory:

```sh
cargo dokime dev{{selection}}
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
`dokime_runtime::plugin::testing::Harness`: parameters, settings and secrets as builder
calls, the awaited record filed the way the host files it, and a typed report of
the verdict, the samples, the records, the traceability and the evidence files.
No host and no hardware are involved.

To create the distributable bundle:

```sh
cargo dokime build{{selection}}
```

The SDK tracks Dokime’s `dev` branch during development. Generation is offline;
the first build resolves the SDK and Rust dependencies. Commit `Cargo.lock`
to retain the resolved revisions.

Run `mise install` once and commit the `mise.lock` it writes: CI installs
exactly the locked tools. `mise run check` formats, lints and type-checks.
`mise run build` writes the shipping bundle under
`target/build/<profile>/<target>/plugins/{{package}}.dokime`.
`mise run build:rust:tests` builds the archive consumed by `test:rust:unit` and
`test:rust:integration`. `mise run package:rust` copies and inspects the existing
bundle under `target/package/<profile>/<target>/rust/` without recompiling.
The CI workflows invoke those same producers and consumers.
