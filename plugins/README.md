# Dokime Plugins

`sidecars/host/` supplies the six host telemetry streams through the supervised
plugin ingest boundary. Its single catalog contains the collection operations,
stream declarations and demand-driven observers. Operation definitions own the
policies; Windows and Unix implementations are selected at module declarations.
Its Collection settings retain the Wi-Fi/Bluetooth adapter selectors and latency
host. Empty adapter selectors use the native default. Native repeated fields
carry lists; acquisition timestamps travel in framework publication metadata.
The runtime decodes these rows through the same codec as other plugins.

`sidecars/obc/` is the test-only onboard charger plugin. Its `runtime/` crate owns
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
