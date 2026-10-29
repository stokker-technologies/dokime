# Dokime Plugins

`plugins/sidecars/host/` supplies the six host telemetry streams through the supervised
plugin ingest boundary: `host.system`, `host.network` (with OS-reported
`internet`), `host.wifi.link` (the current connection), `host.wifi.scan` (one row
per nearby network), `host.wifi.latency` (round trip and loss to the latency host)
and `host.bluetooth`. Its single catalog contains the collection operations,
stream declarations and demand-driven observers; each stream has its own observer,
which runs only while a lease holds that stream. Operation definitions own the
policies and `HostTelemetryStream::interval()` owns the default cadences; Windows
and Unix implementations are selected at module declarations. Windows probes use
native APIs and start no processes. Its Collection settings retain the
Wi-Fi/Bluetooth adapter selectors and latency host, plus the minimum interval
between active Wi-Fi scans; a scan due sooner reports the cached network list.
Empty adapter selectors use the native default. Native repeated fields
carry lists; acquisition timestamps travel in framework publication metadata.
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
