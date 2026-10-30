# Dokime Plugins

`plugins/sidecars/host/` supplies the six host telemetry streams through the supervised
plugin ingest boundary: `host.system`, `host.network` (with OS-reported
`internet`), `host.wifi.link` (the current connection), `host.wifi.scan` (one row
per nearby network), `host.wifi.latency` (round trip and loss to the latency host)
and `host.bluetooth`. Separate operation, stream and observer catalogs share
the same declarations; each stream has its own demand-driven observer,
which runs only while a lease holds that stream. Operation definitions own the
policies and `HostTelemetryStream::interval()` owns the default cadences; Windows
and Unix implementations are selected at module declarations. Windows probes use
native APIs and start no processes. Its Collection settings retain the
Wi-Fi/Bluetooth adapter selectors and latency host, plus the minimum interval
between active Wi-Fi scans; a scan due sooner reports the cached network list.
Empty adapter selectors use the native default. Native repeated fields
carry lists; acquisition timestamps travel in framework publication metadata.
`host.network` carries aggregate counters, `internet`, and the default-route
adapter id. Timestamped `link_up`, `link_down`, `address_added`, `address_removed`
and `default_route_changed` rows come from Windows IP Helper callbacks or Linux
route netlink notifications, held only for the observer's lease. A receive
overrun fails observation visibly; inventory is never polled to infer events.
The Public `dokime_host::host::adapters::Adapters` query (`host.network.adapters`)
returns adapter identity, kind, MAC, status, MTU, speed, IPv4/IPv6 prefixes and
scope, gateways, DNS servers, DHCP state and default-route ownership. Linux
uses `getifaddrs`, kernel routes and NetworkManager's per-device DNS/DHCP data;
DHCP is unknown when NetworkManager does not manage an interface.
The installed host contributes a synchronous `RunStart` capture: every execution
records its adapter inventory before the first step.

The Public `host.network.radio.set` request is
`{ radio: "wifi" | "bluetooth" | "cellular", state: "on" | "off", allow_disconnect: false }`.
It uses Windows.Devices.Radios or NetworkManager/BlueZ native switches.
`host.network.adapter.set` takes `{ adapter_id, enabled, allow_disconnect: false }`;
Windows uses quiet SetupAPI and refuses without administrator rights, without UAC.
Linux requires root or CAP_NET_ADMIN for interface writes.
Before disabling, the host maps the trusted backend PID's established native TCP
sockets to adapters. A change affecting a live connection refuses unless
`allow_disconnect: true` comes from an execution step; a standalone request flag
cannot bypass the guard. Bluetooth device presence uses native enumeration.

Radio, adapter and `host.power.plan.set { plan_id }` changes preserve the exact
prior state as invertible transactions. During an execution the backend durably
journals the inverse before writing, records native apply/restore timestamps,
and restores in reverse order after completion, failure or cancellation.
Cancellation keeps native work owned until it settles. On restart, pending
inverses run before startup returns; a failed or unavailable inverse blocks
startup and stays pending. Keep the restoring plugin installed. Standalone
changes persist without run-end inversion. Development storage stays v1; use a
fresh storage root for incompatible schemas and preserve the previous root.
The Public `host.power.plan.get` query reads the active Windows power scheme
or Linux ACPI platform profile; unavailable native reads return a refusal.
The Public `host.cpu.load` query and `host.system` rows expose calibrated
capacity and estimated load in GFLOP/s. At plugin start, a bounded kernel runs
eight independent f64 fused multiply-add accumulators on each available logical
CPU. Each FMA counts two operations; completed work divided by total wall time
gives kernel-equivalent capacity. The kernel budget is half the CPU operation's
attempt policy. Estimated load is capacity multiplied by OS utilization / 100;
it is a workload-equivalent estimate, rather than a measured instruction count
or the processor's theoretical peak. Calibration reflects the compiled kernel,
current machine and power state at startup. The method is beside `CpuLoad`.
The runtime decodes these rows through the same codec as other plugins.
Direct `RuntimeHandle::publish_telemetry_batch` calls also retain original
canonical payload bytes. If their value map omits `timestamp`, the runtime
infers it from the sample instant using the contract's column type: `Datetime`
preserves the full instant, while `Number` carries seconds since the Unix epoch.
Explicitly supplied timestamp values remain subject to contract validation.
`dokime-host` is one crate: with no features it is the light definitions crate
(operation requests and responses, ids, policies and stream rows); the `runtime`
feature adds the operation implementations, platform dependencies and the
sidecar executable, which the product build enables. It is never published to
crates.io; the host ships as the `host.dokime` plugin artifact. A plugin outside
this repository calls host operations through the definitions from a Dokime
release tag, matching the tag used for its `dokime` SDK dependency:

```toml
dokime-host = { git = "https://github.com/stokker-technologies/dokime", tag = "v0.2.0", default-features = false }
```

Host stream rows and contracts live in `dokime_host::domain::telemetry` (the rows are also exported at the crate root). Host document defaults and `wifi_channel_occupancy` live in `dokime_host::domain::document`. Dokime itself owns no host stream definitions; its `dokime::host` module contains only application lifecycle records. Install the host bundle before admitting templates that subscribe to `host.*`: absent publishers receive the ordinary per-stream refusal, and valid leases remain independent. Application-registered catalogs retain `PluginOwner::Host` and appear as Built-in.

Its sidecar owner is `host_telemetry`; `host` names application-owned operations
in Harmos. The bundle remains `host.dokime`, with `host.*` stream and operation IDs.
The host sidecar declares no automatic restart: a process fault deactivates it
until an explicit reload or the next startup.

Host observers run only while their streams are leased. Execution subscriptions
and live plot viewers hold those leases; releasing the last lease stops the
observer while the plugin process can remain idle.

`plugins/sidecars/obc/` is the test-only [onboard charger plugin](obc-plugin.md). Its `runtime/` crate owns
the simulated testbench. It is built explicitly for tests and is never included
in the product's default bundles or installer.

A plugin declares its identity, requirements and contributions on
`#[dokime::plugin] impl dokime::Plugin`; `const CATALOGS` lists its executable
`#[dokime::harmos::catalog]` operation catalogs. Contribution catalogs use
`#[dokime::catalog]`. Each request struct declares its id, policy and
`resources = [...]`, and implements `Operation::run`.

A separate `impl dokime::harmos::Lifecycle` owns configuration, errors and the
`prepare` → `initialize` → serve → `stop` → `cleanup` lifecycle. Preparation
registers every shared resource with `setup.resource(...)` and prepares suite
resources with `dokime::execution::prepare::<Self, Suites>(setup)`.
Initialization activates resources that need a running runtime. Operations,
listeners and observers read prepared resources from their execution contexts;
mutable resources register as `Arc<tokio::sync::Mutex<T>>` or
`Arc<tokio::sync::RwLock<T>>`. Observer attachments are prepared from their catalog
and the plugin's declared streams.
Cargo supplies package identity, version and description. Suite declarations
produce their execution factories; authors do not maintain a second runtime
inventory.

`cargo dokime sidecar build` compiles the selected native executable, runs its bounded
catalog export, and creates a `.dokime` bundle containing the executable,
frozen registrations and `assets/`. Ordinary `cargo build` only compiles.
Catalog export must be deterministic: declaration identities use stable seeds,
while runtime execution identities remain fresh.

`mise run build:rust` produces the shipping `plugins/host.dokime` bundle.
`mise run build:rust:tests` prepares the explicit OBC test fixture under
`fixtures/plugins/obc-plugin.dokime`.
Operators select or install bundles; the product does not seed these fixtures. A card's probe
and dropdown sources come off the authored settings aggregate —
`PluginSettings::status_operation()`, `PluginSettings::option_providers()` — read
from the admitted catalog; the host has no compiled plugin adapters.

Release staging groups shipping sidecars into one native archive per target:
`dokime-sidecars-<version>-<target>.zip` on Windows and `.tar.gz` on Unix.
Members keep their declared unversioned `.dokime` filenames. `plugins.json`
format 2 lists plugin names, kinds, versions, Dokime requirements and target
archive/member/hash mappings. The desktop lists every compatible plugin from the
selected update source, verifies the archive against SHA256SUMS and the requested
member against the feed, then installs only that member. There is no v1 reader
or individual-plugin release-asset fallback. See the
[release feed example](product-guide.md#organization).

Custom installers use top-level `[[plugin]]` entries with exactly one of `name`
(a release feed name) or `path` (a local bundle relative to the organization
pack). The pack's `plugins/` folder is storage only; unlisted files never ship
and plugin files are not overlaid. Repeat `--plugin <path>` for equivalent
external bundle or sidecars archive inputs relative to the invocation directory.
Names come from feed and product declarations; they do not depend on crate naming.
See [Distribution From A Release](product-guide.md#distribution-from-a-release).

Operators install bundles through **Settings → Plugins**. Installation validates
the complete bundle and frozen registrations without running author code, then
atomically binds an installation UUID to its SHA-256 in `settings.json`; immutable bytes live at
`<application root>/plugins/<UUID>.dokime`. Files copied there manually are not
admitted and the boot sweep deletes them. `DOKIME_PLUGINS_DIR` (or
`--plugins-root`) names a development directory whose bundles load for one
process; it is never a store. A corrupt `settings.json` fails explicitly; missing
or changed bundle bytes remain visible by UUID and error, without displaying
unverified author metadata.

Installation checks configured and installed owner collisions before publication. Replacing an installed
package marks its previous version for next-boot removal; package names identify
installations, while executable IDs identify dependencies. Boot selects native
files for the current platform, verifies frozen and live registrations before
initialization, and admits catalogs only after every surviving package initializes.
A failed package loses all its executable siblings and dependent packages.
A bundle that declares a Harmos guest is refused at admission.

Catalog ownership is explicit. Host registrations survive plugin removal.
Provider reconciliation rebuilds the catalogs owned by configured, embedded and
installed plugins from their admitted registrations.
