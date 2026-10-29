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
- [ ] 3.2 Confirm the plugin resident lease wants no stream while idle.

## 3A. Shared Operations And Definitions Crate

- [ ] 3A.1 `visibility = Shared` on adapters, radio/adapter set, power plan read/set and CPU load; `*.collect` stay private.
- [ ] 3A.2 `dokime-host` `default = []`, `runtime` gates implementations and optional platform dependencies; bin requires `runtime`; sidecar build and packaging enable it.
- [ ] 3A.3 No-feature guard; document the git-tag dependency for external plugin authors; proof: no-feature check, runtime build, bundle build, workspace check.

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

- [ ] 6A.1 Runtime: stop linking and boot-installing `dokime_host`; remove `PluginOwner::Host`, `StreamOwner::Host`, built-in host sampling and every host special case; host streams and leases follow ordinary plugin ownership; GUI drops `<brand> Host` labels.
- [ ] 6A.2 Release: publish `dokime-host` per-target `.dokime` bundles and their `plugins.json` entry (generic release layout); tests and fixtures install the bundle explicitly.
- [ ] 6A.3 Organization pack `plugins/`: packaging verifies digest, target and compatibility and embeds the bundles; first start and upgrades install absent or older bundles as ordinary plugins without downgrading; update the organization-installers guidance and product guide.
- [ ] 6A.4 Proof: runtime boots without the host plugin; installed host bundle streams under its own name; packaging rejects a mismatched bundle; upgrade keeps a newer installed version.

## 7. Validation

- [ ] 7.1 Targeted proof per item; workspace check; `mise run format`.
