# Design

Locked by the owner on 2026-10-29.

## Child Processes

```rust
// dokime_domain::process — the only Command constructor; CREATE_NO_WINDOW on Windows, a no-op elsewhere.
pub fn command(program: impl AsRef<OsStr>) -> std::process::Command;
// Async callers: tokio::process::Command::from(command(program))
```

`clippy.toml` disallows `std::process::Command::new` and `tokio::process::Command::new` workspace-wide. `process-wrap` job objects re-apply the flag through a `CreationFlags` wrapper.

## Host Streams

| Stream | Reads | Source (Windows / Linux) | Default cadence |
|---|---|---|---|
| `host.wifi.link` | adapter, state, `associated`, SSID, BSSID, signal (RSSI dBm, %), channel, band, PHY rates, adapter throughput | `WlanQueryInterface` (current connection, RSSI, channel number) / NetworkManager (`nmcli`) plus `/proc/net/wireless` and `iw` | 1 s |
| `host.wifi.scan` | one row per BSS: SSID, BSSID, signal, channel, band, security, `associated`; per-channel and per-scan counts; congestion score | `WlanScan` + `WlanGetNetworkBssList` + `WlanGetAvailableNetworkList` / `nmcli device wifi list --rescan` | 60 s, never faster than the configured minimum scan interval |
| `host.wifi.latency` | target, RTT, loss | `IcmpSendEcho2`/`Icmp6SendEcho2` / `ping` through the process helper | 10 s |
| `host.network` | existing counters plus `internet` | Network List Manager `GetConnectivity` (IPv4/IPv6 internet flags) / NetworkManager connectivity (`nmcli networking connectivity`) | 1 s |

- Cadences have one source: `HostTelemetryStream::interval()` in the `dokime-host` definitions crate, read by each observer's `INTERVAL`.
- Bounds come from each collect operation's `POLICY`; the echo waits half the latency attempt timeout so a lost reply is a `loss` sample, not a timed-out collection.
- Settings (`host_telemetry` → Collection): `latency_host` (default `1.1.1.1`), `minimum_scan_interval_s` (default 30). A scan request inside the minimum interval reuses the cached BSS list without a new active scan and reports `scan_status = "cached"`.
- The link no longer carries scan-derived congestion; the scan computes it per scan from its own associated BSS. The Wi-Fi occupancy plot binds to `host.wifi.scan` (`y = network_count`, grouped by `wifi_channel`); the congestion map joins `host.wifi.scan`.
- Platform code lives in `windows.rs`/`unix.rs` selected by gated `mod` declarations only.

## Spin-Down

Every host observer declares `Activation::Demand` with its stream as the demand key. Harmos holds one resident activation per observer only while the stream is wanted and drops it when the last lease releases, which cancels the observe invocation. The plugin lifecycle's `submit_resident` keeps the sidecar service (process) resident; it does not want any stream, so no observer runs while idle.

## Adapters (Read)

```text
host.network.adapters () -> { adapters: [Adapter] }
Adapter { id, name, description, kind: wifi|ethernet|cellular|virtual|loopback|other, mac, status, mtu, speed_bps,
          ipv4: [{address, prefix}], ipv6: [{address, prefix, scope}], gateways, dns_servers, dhcp, primary }
```

- Windows `GetAdaptersAddresses`; Linux `getifaddrs` plus routes or NetworkManager. `primary` carries the default route.
- `host.network` keeps `internet`, the primary interface id and optional per-adapter rx/tx counters, and publishes change events (link up/down, address added/removed, default route changed) from `NotifyIpInterfaceChange`/`NotifyUnicastIpAddressChange` (Windows) and netlink or NetworkManager signals (Linux), without polling.
- Execution start records one `host.network.adapters` snapshot in the execution evidence.

## Radios And Adapters (Write)

```text
host.network.radio.set   { radio: wifi|bluetooth|cellular, state: on|off }   // Windows.Devices.Radios / NM radio switches; no admin
host.network.adapter.set { adapter_id, enabled }                            // Windows: administrator required (MSFT_NetAdapter or SetupAPI)
```

- Both are invertible Harmos transactions (`Invertible`); the inverse restores the prior state.
- Every change made during an execution is recorded and inverted at run end (complete, cancel or failure); a backend starting after a crash applies pending inverses before anything else.
- Each change is evidence with timestamps, so reports show the blackout window.
- Without administrator rights `adapter.set` refuses with a clear "requires administrator" error and never prompts UAC.
- Guard: switching off the radio or adapter that carries the active remote client or backend connection is refused unless the step explicitly allows it.

## Remaining Host Features

- Bluetooth on/off through `host.network.radio.set`; device presence when cheap.
- Power plan read (`PowerGetActiveScheme`/`PowerReadFriendlyName`) and invertible set (`PowerSetActiveScheme`).
- CPU load in GFLOP/s: a calibrated measure (peak GFLOP/s from a bounded calibration kernel at plugin start, scaled by measured utilization); the method is documented beside the operation.

Every parameter comes from operation policy or settings; no spawned processes where a native API exists.

## Shared Operations

`host.network.adapters`, `host.network.radio.set`, `host.network.adapter.set`, the power-plan read and set and CPU load declare `visibility = Shared` so other plugins may call them. The periodic `*.collect` operations stay private.

## Definitions Crate

`dokime-host` stays one crate and is the light definitions crate:

```toml
[features]
default = []
runtime = ["dep:windows", "dep:tokio", "dep:sysinfo", ...]   # every operation implementation and platform dependency

[[bin]]
required-features = ["runtime"]
```

- Request and response types, ids and policies compile with no features; runtime-only resources named in `resources = [...]` exist or are gated so the no-feature build holds, and Harmos generates `Requires` only under `runtime`.
- Operation implementations (`windows.rs`/`unix.rs`) and platform dependencies (`optional = true`) are gated by `runtime`; the sidecar build and packaging enable `runtime` explicitly.
- A caller-style guard builds `dokime-host` with `--no-default-features`.
- `publish = false` stays: the host ships only as the `host.dokime` plugin artifact. Plugins outside this repository depend on the definitions through a Dokime release tag, which pulls definitions only:

```toml
dokime-host = { git = "https://github.com/stokker-technologies/dokime", tag = "v0.2.0", default-features = false }
```

## Host Is An Ordinary Plugin

Locked 2026-10-29, clarified by the owner on 2026-10-29 for 6A.1. `dokime-host` ships as ordinary per-target `.dokime` bundles; the runtime production library neither links nor boot-installs it. `PluginOwner::Host` stays for catalogs registered by the embedding application, with GUI `<brand> Built-in` and server `Built-in` labels. It does not identify the host plugin. `StreamOwner::Host` and built-in sampling remain removed.

`state::is_host_stream` is removed: each `host.*` subscription without a publisher receives the same visible refusal as any other unpublished stream, independently of other subscriptions and their leases. Host rows, contracts, cadences, document defaults and the Wi-Fi occupancy binding move to `dokime-host` definitions; all runtime fixtures, GUI bindings, OBC suites, templates and guidance migrate without `dokime::host` stream re-exports. Application lifecycle records remain owned by Dokime. Default features remain definitions only, `runtime` gates implementations, and `publish = false` stays; external plugins use a git release tag. Clean breaks retain schema v1 and never reset existing storage.

Organization packs carry an optional `plugins/` folder of `.dokime` bundles (for example the host plugin downloaded from a Dokime release). `cargo dokime installer package` verifies each bundle's digest, target and Dokime compatibility against the installer and fails packaging on any mismatch, then embeds them. On first start and after an installer upgrade, bundled plugins install as ordinary bundle-owned plugins under their own names when absent or older; a newer installed version is never downgraded. Later updates come from the configured release source.

