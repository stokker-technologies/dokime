# Tasks

Locked 2026-10-29. One commit per item; targeted proof per item; Linux paths are proven by CI only.

## 1. Windowless Child Processes

- [x] 1.1 Merge `fix/windows-console-windows` (dev fd5e3939, merge da224f98): `dokime_domain::process::command`, Clippy ban, every spawn routed (packaging included), native power plan, native echo at half the latency bound, git identity once per collector. Proof: locked workspace/all-targets check and Clippy pass (dokime-cli Clippy blocked only by another writer's redundant closures, since fixed upstream); host lib 11 and integration 4 pass.

## 2. Wi-Fi And Network Streams

- [x] 2.1 Domain: replace `Wifi`/`WifiChannelOccupancy` with `WifiLink`/`WifiScan`, add latency loss and `internet`, cadences in `HostTelemetryStream::interval()`.
- [x] 2.2 Host plugin: link, scan and latency observers with separate state; minimum scan interval setting; Windows link channel, scan security, Network List Manager connectivity; Linux NetworkManager sources.
- [x] 2.3 Consumers: occupancy plot binding and validation, host document defaults, runtime plot context, GUI plot context, OBC suites, soak template, runtime test fixtures, plugins README (the product guide names no host stream).
- [x] 2.4 Proof: locked workspace/all-targets check; host lib 14 and integration 6; domain lib; runtime integration `domains::telemetry`, `domains::document_reports`, `domains::evidence_publication` 33 pass; Windows readings: link 120 ms (associated, ch 36 5 GHz, -55 dBm), scan 63 ms active and 48 ms cached (12 BSS), latency 8 ms (6 ms RTT, 0 % loss), network internet true in 72 ms. Linux paths are proven by CI only.

## 3. Spin-Down

- [ ] 3.1 Deterministic test: one leased host stream runs only its observer; releasing it cancels the observe invocation.
  Drafted (`domains::telemetry::sampling::host_observer_spins_down`, watching `operations().watch_residents()`): with lease-driven demand only `host.system` starts, but its resident takes about 540 s to retire after release and the runtime shutdown then fails (`harmos.shutdown` Unavailable). Blocked on that Harmos observer-cancellation defect; the draft is not landed.
- [ ] 3.2 Confirm the plugin resident lease wants no stream while idle.
  Finding: `submit_resident` holds only the sidecar service. Idle streams stay warm because `runtime/native_telemetry.rs::observe` keeps a `publication.watch()` on every stream of each exposed plugin, so every demand observer runs. Fix (drafted, lands with 3.1): leases own demand (first lease engages a watch, last release drops it), exposure re-engages held streams, and `dokime.plots.watch` holds viewer leases for the plots it serves.

## 3A. Shared Operations And Definitions Crate

- [ ] 3A.1 `visibility = Shared` on adapters, radio/adapter set, power plan read/set and CPU load; `*.collect` stay private.
  Declared with each operation as it lands (items 4–6); `Shared` becomes `Public` when Dokime pins the Harmos release that renames it.
- [x] 3A.2 `dokime-host` `default = []`, `runtime` gates implementations and optional platform dependencies; bin requires `runtime`; sidecar build and packaging enable it.
- [x] 3A.3 No-feature guard; document the git-tag dependency for external plugin authors; proof: no-feature check, runtime build, bundle build, workspace check.
  Done: the guard is the existing `build:rust:compatibility` `--workspace --lib --bins --no-default-features` lane plus every featureless workspace check; plugins README documents the git-tag dependency. Proof so far: `check -p dokime-host --no-default-features --all-targets` PASS, `test -p dokime-host --no-default-features --lib` 7 PASS, `test -p dokime-host --features runtime --lib --test integration` 14 + 6 PASS, workspace/all-targets check PASS, `mise run build:rust` PASS (host.dokime built with `dokime-host/runtime`).

## 4. Adapters (Read)

- [ ] 4.1 `host.network.adapters` query (Windows `GetAdaptersAddresses`, Linux `getifaddrs` plus routes).
- [ ] 4.2 `host.network` compact form with change events from OS notifications.
- [ ] 4.3 Adapters snapshot in execution evidence at run start.

## 5. Radios And Adapters (Write)

- [ ] 5.1 `host.network.radio.set` (Windows.Devices.Radios / NM radio switches).
- [ ] 5.2 `host.network.adapter.set` with the administrator refusal.
- [ ] 5.3 Invertible transactions, run-end inversion, crash recovery, evidence timestamps.
- [ ] 5.4 Active-connection guard with explicit step override.

## 6. Remaining Host Features

- [ ] 6.1 Bluetooth on/off through the radio operation; device presence.
- [ ] 6.2 Power plan read and invertible set.
- [ ] 6.3 CPU load in GFLOP/s from a documented calibration.

## 6A. Host As An Ordinary Plugin

- [ ] 6A.1 Keep application catalog ownership, remove host stream exemptions, and move host definitions and every consumer to `dokime-host`.
  Owner decisions (2026-10-29): keep `PluginOwner::Host` for embedding application catalogs; label it `<brand> Built-in` in GUI and `Built-in` on the server. Remove `state::is_host_stream`; unpublished host subscriptions receive ordinary per-stream visible refusal, preserving independent leasing. Move host rows/contracts/cadences, document defaults and Wi-Fi occupancy binding into the definitions-only default of unpublished `dokime-host`; implementations require `runtime`, external consumers use git tags, and no legacy re-export paths remain. Application lifecycle records stay with Dokime.
  Triage: runtime already has no host dependency or boot installation (`crates/dokime-runtime/Cargo.toml`, `src/app/`); bundle ownership is ordinary (`src/state/ownership.rs`); `StreamOwner` and built-in sampling are absent. Remaining source inventory: domain telemetry/document owners, runtime authoring exports and state admission, host definitions/operations, GUI binding generator and plot adapters, OBC suites, starter templates, runtime/server/macro fixtures and manifests, plugins README and affected authoring/architecture skills. Four framework-worker paths named in the assignment are excluded.
- [x] 6A.2 Release: publish `dokime-host` per-target `.dokime` bundles and their `plugins.json` entry (generic release layout); tests and fixtures install the bundle explicitly.
  Triage: already implemented in `stkr.toml:576`, `tools/python/src/dokime_tools/release/layout.py:80`, `tools/python/src/dokime_tools/release/run.py:137`, and `crates/dokime-runtime/tests/support/services/bundles.rs:86`; shipping collection uses declared plugins and OBC remains a separate test fixture (`tools/python/src/dokime_tools/package/plugins.py:34`). Targeted plugin-release Python tests pass (6). `release:publish:dry` reaches staging but refuses missing current-version packaged executables: retained packages are 0.1.0 while source is 0.1.1. Staging the host bundle/feed remains unproven until matching Product/Package artifacts exist.
- [x] 6A.3 Organization pack `plugins/`: packaging verifies digest, target and compatibility and embeds the bundles; first start and upgrades install absent or older bundles as ordinary plugins without downgrading; update the organization-installers guidance and product guide.
  Helper source inventory: packaging bundle compatibility metadata and build collection, installer plugin discovery/verification/staging (existing native embedding retained), runtime plugin-store monotonic installation, distribution reconciliation marker in application metadata and startup discovery, targeted packaging/store tests, product guide, organization-installers design and architecture guidance. No host definitions, telemetry, domain or framework-worker paths are owned by this helper. README impact: custom installers now describe optional organization `plugins/` and no-downgrade startup installation; detailed instructions live in the product guide.
  Integration handoff: canonical architecture guidance is shared with 6A.1; its generated bodies match the source, but `stkr project agents status --json` still reports architecture/authoring surfaces stale. Their combined refresh belongs to 6A.1's guidance closure. Release feed minimum compatibility now matches the host's Cargo dependency (0.1.1).
  Implemented source: `crates/dokime-packaging/src/installer/plugins.rs:47` verifies and stages organization/explicit bundles; native embedding remains at `src/installer/native.rs:98`. `crates/dokime-runtime/src/plugins/store.rs:152` prevents downgrades under the settings lock; `src/plugins/store/distribution.rs:42` reconciles the integrity-indexed bundles on first start/upgrades once per distribution digest. Four focused packaging tests pass, covering embedding/overlay and digest/target/compatibility refusals. Canonical guide, README and organization-installers design are updated.
  Runtime proof passes: `mise run build:rust:focused -- test -p dokime-runtime --test integration -- plugin_store_leases::store::bundled_` (2 tests), covering initial installation, upgrade, equal/newer preservation, concurrent no-downgrade selection, settings/disabled-state preservation, removal across restarts, ignored leftovers and retry after checksum refusal. Scoped Rust formatting passes. Earlier fixture compilation and transient executable-link failures are repaired; Linux proof remains with CI.
- [ ] 6A.4 Proof: runtime boots without the host plugin; installed host bundle streams under its own name; packaging rejects a mismatched bundle; upgrade keeps a newer installed version.

## 7. Validation

- [ ] 7.1 Targeted proof per item; workspace check; `mise run format`.

- [x] 7.2 Repair hosted Rust formatting in already-committed doctor and runtime storage/evidence paths; leave active writers untouched. Evidence: `crates/dokime-cli/src/commands/doctor.rs:87` and five runtime storage/evidence files are formatted; formatter task reports only active-writer paths.
- [x] 7.3 Remove retired tutorial artifact staging, release preparation hook, bundle declaration and candidate tests; update source tutorial setup and prove focused Python integration. Evidence: `tools/python/src/dokime_tools/build/python.py:35`, `mise.toml:487`, `stkr.toml:558`; focused integration selects 2 passing plugin-collection cases and no retired candidate stories; Python client build and package pass.
