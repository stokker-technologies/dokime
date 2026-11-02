# Tasks

Locked 2026-10-29; scheduled by the owner. Implementation in `feat/distribution-identity`.

## Source Inventory

Installer identity and native assembly: `dokime-packaging/src/installer/{manifest,distribution,native}.rs`, identity-only hooks and packaging tests; integrate the bundled-plugin startup inventory reader and its fixtures with `installer.json`. Plugin selection/staging remains with the host-plugin-features owner.

Shared identity validation: `dokime-domain/src/model/identity/distribution.rs`. Shared resolver and consumers: `dokime-runtime/src/paths.rs`, configuration snapshots and consumers (runtime settings, server diagnostics/log streams), desktop bootstrap/organization/settings/logging/local backend and native-session paths and cache consumers; server logging, CLI server startup and `dokime/src/installation.rs` GUI discovery. Preserve compiled protocol and updater identities.

Proof and guidance: resolver tests, installer tests, native desktop lifecycle identity coverage, product guide and README. Implement the shared identity contract before packaging and consumer integration. Native Windows and Unix walkthroughs remain separate validation obligations; this slice runs targeted proof only and never pushes.

The startup identity file is separate from `installer.json`, the inventory consumed by bundled-plugin startup. The 2026-10-30 owner decision retires installer inspection; no NSIS decoder repair remains in scope.

## 1. Implementation

- [x] 1.1 Installer manifest `[installer] identifier` with validation; payload identity checks accept the distribution identifier.
- [x] 1.2 `cargo dokime installer package` writes `distribution.json` into the payload inventory and uses the identifier for the NSIS install folder, uninstall registry key and DEB/RPM package names. **Reconciled:** Payload identity and native package naming are implemented in `crates/dokime-packaging/src/installer/native.rs` and `installer.nsi`; supplied Windows installer build/install evidence confirms the identifier folder and both payload manifests.
- [x] 1.3 One path resolver (desktop, CLI, server) reading `distribution.json` with the compiled identifier as fallback; replace `app_data_dir()` and `app_log_dir(identifier)` uses; set the WebView data directory.
- [x] 1.4 Product guide: Native Installers and Organization sections; README claims.
- [x] 1.5 Tests: resolver unit tests, packaging tests for the written file and names, a native lifecycle check that a branded and a plain installation keep separate directories. **Reconciled:** Resolver, native naming and packaging guards are landed in runtime `tests/integration/settings_backends/distribution.rs`, packaging `tests/integration/installer.rs` and GUI `e2e/native-desktop.mjs` (distribution identity journey). Supplied Windows lanes passed at #166 and #170; Unix walkthrough remains 2.1.
- [x] 1.6 Installer display-name repair: organization labels in native display fields; identifier-based install paths and registry keys. Evidence: `crates/dokime-packaging/src/installer/native.rs:623` (`distribution_names`) and `installer.nsi`; all 22 targeted installer unit tests pass.

## 2. Validation

- [ ] 2.1 Targeted proof, then the native desktop walkthrough on Windows and Unix with a branded distribution. **Remaining:** Windows resolver/packaging/desktop lanes and supplied branded installation evidence pass; a native Unix branded-distribution walkthrough is still unproven.

## Workflow Follow-Up

- Consolidation merge proof: locked workspace/all-targets check passes; runtime integration `settings_backends` passes all three stories and packaging integration `installer::` passes both stories. Desktop delivery keeps verified-save ordering and resolves the open cache from `ApplicationPaths`; identity inventory and organization/explicit bundle staging remain integrated. Native platform walkthroughs remain open.

- Integration preserves the distribution slug in native filenames and the release layout's Rust target in NSIS filenames. Verified organization-folder and explicit plugin staging are preserved; Linux plugin destinations follow the distribution directory. Bundled-plugin startup reads and fingerprints `installer.json`; `distribution.json` retains only the startup identity. First-start installation, upgrade-only reconciliation and preservation of disabled state/settings remain with the plugin store.
- The owner retired native installer inspection on 2026-10-30 in `retire-installer-inspect`; the NSIS decoding repair and diagnostic fixture are superseded. Native Windows and Unix walkthroughs remain open under 2.1.
