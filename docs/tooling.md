# Repository Tooling

Use `mise tasks` to discover entry points, with arguments after `--`. Preparation
installs the root `.venv/` and frontend dependencies. Build, test and checks use
project tasks; compiler caches belong to the workstation.

## Focused Development

```sh
mise run prepare
mise run build -- --package dokime-runtime-next --check
mise run build -- --cargo check --locked --offline --package dokime-operations-next
mise run test -- --package dokime-runtime-next --target system --story storage_root_admission
mise run test -- --package dokime-protocol --target integration
mise run test -- --changed crates/dokime-runtime-next/tests/system/manager_lifecycles.rs --plan
mise run format
mise run check -- --language rust
mise run check -- --language python
mise run check -- --language typescript
mise run check -- --repository
```

Focused tests use nextest and compile only the selected package/target. `--exact`
selects a test, `--story` selects a module, and `--filterset` takes a nextest expression.
`--artifact tests` consumes the matching prepared archive without compilation.
An empty selection fails. Clippy runs through lint/check, never build/test.

## Fixtures and Stories

```sh
mise run build:host-fixtures
mise run build:cli-fixtures
mise run test -- --package dokime-runtime-next --target system
mise run test -- --package dokime-cli-next --target system
mise -C sdk/rust run sdk:test
mise -C sdk/python run sdk:test
mise -C sdk/csharp run test
mise -C sdk/cpp run test
mise run build -- --artifact tests
```

Host fixture production builds the Rust reference, content fixture and host;
C# and C++ producers use their SDK tasks. `build:host-fixtures --language
rust|python|csharp|cpp` selects one producer. Fixtures are exported under
`target/build/<profile>/<target>/fixtures/host/`. CLI fixtures include a generated
Rust starter built through its own mise task. Stories never compile plugins.

`HARMOS_SOURCE`, `DOKIME_HARMOS_CACHE` and `DOKIME_CPP_BUILD_DIR` may select
workstation source/dependency caches; do not put them in project configuration.
Python consumes the built pure wheel and the pinned library SDK. Linux stories
own an isolated session bus/keyring for credential storage; install `dbus` and
`gnome-keyring`. Windows uses its operating system credential store.

## Desktop Integration

The native desktop is outside Cargo membership until its host switch merges.
The restoration checklist in `openspec/changes/grpc-plugins/tasks.md` owns native
Product builds, generated preference/settings checks, system stories, benchmarks,
installer packaging and signing. The frontend retains formatting/lint and protobuf
message generation. Unit/integration stories independent of native preferences
remain selected. The preferences unit journey returns with its native declaration
and default exporter; its source is retained in the desktop tree.

`mise run generate` generates frontend protobuf modules from the authoritative
checked-in descriptor, without compiling a host. `mise run generate -- --protocol`
regenerates checked-in Rust protocol output; never edit generated files.

## Build and Delivery Artifacts

`mise run build` produces the current host/CLI workspace. Tests Build produces
one archive plus host/CLI fixtures and the vendored protobuf compiler input.
No native desktop, sidecar plugin or removed macro/compiler fixture is selected.
The current Cargo workspace contains protocol, storage, operations-next,
runtime-next, CLI-next, SDK and bundle inspection packages.

Product outputs live under `target/build/<profile>/<target>/`; packages under
`target/package/<profile>/<target>/`. CLI archives retain the `dokime-next` binary.
The distribution ZIP contains that CLI and legal notices. Desktop installers and
shipping sidecar feeds return only after their restoration/re-authoring tasks.
The pure Python wheel/source pair uses the shared `any/python/` target.

`tools/project.toml` owns package/release/test policy; `mise.toml` owns commands
and tool pins. `.stkr/project.toml` contains only Hub source policy. Configuration
under `tools/config/` is selected explicitly. Do not redirect another project's
Python environment or place TEMP/TMP inside a checkout.

## Hosted Validation

Pull requests and Nightly retain native Linux and Windows host/SDK validation.
PR host fixtures select Rust/Python; Nightly adds C#/C++ equivalence. All local
host stories are available by target. Dev pushes run quick checks/builds; PRs run
stories. Full validation belongs to hosted CI and Nightly after combined source
closure. Local work uses focused proof and never substitutes Windows for Linux.
Nightly publication depends on packages; its final report appends suite outcomes.
Release publication requires separately authorized exact-commit green evidence.

Native test reports remain under `target/test-reports/` and traces under
`target/traces/`. Preserve them before native `cargo clean`. Disk monitoring and
cleanup are workstation responsibilities, never project preflights.
