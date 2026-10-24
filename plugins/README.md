# Dokime Plugins

`obc-plugin/` is the maintained onboard charger demo and validation plugin. Its
`runtime/` crate owns the simulated testbench. The demo is installed explicitly
and uses the same native delivery conventions as other plugins.

A plugin is a Harmos plugin: `impl dokime::Plugin` carries its configuration,
error, resources and lifecycle (`prepare`, `initialize`, `bind_operations`,
`shutdown`, `cleanup`), and its resources are shared behind one `Arc` with every
operation, listener and observer. `#[dokime::plugin]` sits on that `impl`
block and carries the identity, the operations it provides and requires, the
restart policy and the contributions it adds to the host — suites, records and
listeners, evidence streams and settings — each owned by a named
`#[dokime::catalog]` enum. Device behaviour is `#[dokime::operation]`s listed in
`provides = operations::Operations`.
Cargo supplies package identity, version and description. Suite declarations
produce their execution factories; authors do not maintain a second runtime
inventory.

`cargo dokime sidecar build` compiles the selected native executable, runs its bounded
catalog export, and creates a `.dokime` bundle containing the executable,
frozen registrations and `assets/`. Ordinary `cargo build` only compiles.
Catalog export must be deterministic: declaration identities use stable seeds,
while runtime execution identities remain fresh.

`mise run build:plugins:dev` builds development and test bundles explicitly.
Operators select or install bundles; the product does not seed these fixtures. A card's probe
and dropdown sources come off the authored settings aggregate —
`PluginSettings::status_operation()`, `PluginSettings::option_providers()` — read
from the admitted catalog; the host has no compiled plugin adapters.

Operators install bundles through **Settings → Plugins**. Staging inspects frozen
bytes without running author code. Consent binds an installation UUID to its
SHA-256 in `settings.json`; immutable bytes live at
`<application root>/plugins/<UUID>.dokime`. Files copied there manually are not
admitted and the boot sweep deletes them. `DOKIME_PLUGINS_DIR` (or
`--plugins-root`) names a development directory whose bundles load for one
process; it is never a store. A corrupt `settings.json` fails explicitly; missing
or changed bundle bytes remain visible by UUID and error, without displaying
unverified author metadata.

Consent rechecks configured and installed owner collisions. Replacing an installed
package marks its previous version for next-boot removal; package names identify
installations, while executable IDs identify dependencies. Boot selects native
files for the current platform, verifies frozen and live registrations before
initialization, and admits catalogs only after every surviving package initializes.
A failed package loses all its executable siblings and dependent packages.
A bundle that declares a Harmos guest is refused at admission.

New catalog ownership is explicit. Host registrations survive plugin removal.
Legacy catalogs without ownership remain readable but authorize no executable;
an admitted bundle can replace a matching unclassified source. Other legacy
sources remain untouched. Bundle removal removes only its tracked catalogs.
