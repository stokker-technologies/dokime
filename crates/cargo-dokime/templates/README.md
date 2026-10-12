# {{package}}

A native Dokime plugin driving a thermal bench. The bench is simulated, so this
runs on any machine; `src/bench.rs` is the one file that pretends, and the first
one to replace with a real driver.

```text
src/bench.rs                  the simulated sensor and heater
src/settings.rs               the one settings section an operator edits
src/error.rs                  the one error every boundary answers with
src/lib.rs                    the catalogs, the settings, and the handles opened once
src/listeners/unit_connected.rs  what the bench hears between runs
src/prompts/checkpoint.rs     the one question it asks an operator
src/routes/chamber.rs         reads the chamber; the one road a template has to it
src/routes/status.rs          what the bench reports about itself
src/observers/temperature.rs  the chamber's row, and the observer that measures it
src/suites/soak/hold.rs       the procedure: steps, samples, criteria, cleanup
tests/soak.rs                 the whole procedure, run in process
```

Each contribution belongs to a named `#[dokime::catalog]` enum in its owning
module. `src/lib.rs` composes these on `#[dokime::plugin]`: `provides =
operations::Operations` names the operations it serves, and `contributions(...)`
lists the suites, records and listeners, evidence streams and settings it adds
to the host. Public `Streams` and `Records` catalogs in `lib.rs` register
process-wide outputs exactly once. Each
observer file defines its own public `Streams` and `Records` catalogs as permitted
subsets, keeping empty catalogs explicit. Demand observers run while any declared
stream or record has an active subscriber; registration alone does not start them.

Device behaviour is a `#[dokime::operation]` on an inherent impl: an explicit
wire id, a version and a policy, with request and response types read off its
`run`. Empty requests use `()`. A template names the operations it calls with
`requires = Required` and invokes `ctx.call::<ReadChamber>(()).await?`; the call
is routed through the host. Adding an operation means a variant in
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
each reload. Type `r` and Enter to retry, `q` and Enter to quit. Use
`cargo dokime doctor{{selection}}` to check your setup.

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
cargo dokime build{{selection}}
```

The SDK tracks Dokime’s `dev` branch during development. Generation is offline;
the first build resolves the SDK and Rust dependencies. Commit `Cargo.lock`
to retain the resolved revisions. The development command
runs its bundled backend by default; no separate Dokime installation is needed.
Use `--dokime PATH` to test an external host explicitly. The backend serves no UI;
Dokime uses the Tauri desktop application.
