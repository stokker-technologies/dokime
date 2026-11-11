# Tasks

## Source Inventory

Installer identity and native assembly: `dokime-packaging/src/installer/{manifest,distribution,native}.rs`, identity-only hooks and packaging tests; integrate the bundled-plugin startup inventory reader and its fixtures with `installer.json`. Plugin selection/staging remains with the host-plugin-features owner.

Shared identity validation: `dokime-domain/src/model/identity/distribution.rs`. Shared resolver and consumers: `dokime-runtime/src/paths.rs`, configuration snapshots and consumers (runtime settings, server diagnostics/log streams), desktop bootstrap/organization/settings/logging/local backend and native-session paths and cache consumers; server logging, CLI server startup and `dokime/src/installation.rs` GUI discovery. Preserve compiled protocol and updater identities.

Proof and guidance: resolver tests, installer tests, native desktop lifecycle identity coverage, product guide and README. Implement the shared identity contract before packaging and consumer integration. Native Windows and Unix walkthroughs remain separate validation obligations; this slice runs targeted proof only and never pushes.

The startup identity file is separate from `installer.json`, the inventory consumed by bundled-plugin startup. The  owner decision retires installer inspection; no NSIS decoder repair remains in scope.

## 1. Implementation
- [x] 1.1 Installer manifest `[installer] identifier` with validation; payload identity checks accept the distribution identifier.
- [x] 1.2 `cargo dokime installer package` writes `distribution.json` into the payload inventory and uses the identifier for the NSIS install folder, uninstall registry key and DEB/RPM package names.
- [x] 1.3 One path resolver (desktop, CLI, server) reading `distribution.json` with the compiled identifier as fallback; replace `app_data_dir()` and `app_log_dir(identifier)` uses; set the WebView data directory.
- [x] 1.4 Product guide: Native Installers and Organization sections; README claims.
- [x] 1.5 Tests: resolver unit tests, packaging tests for the written file and names, a native lifecycle check that a branded and a plain installation keep separate directories.
- [x] 1.6 Installer display-name repair: organization labels in native display fields; identifier-based install paths and registry keys.

## 2. Validation
- [ ] 2.1 Targeted proof, then the native desktop walkthrough on Windows and Unix with a branded distribution.

## Workflow Follow-Up

- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- Run only focused tasks until combined source closure; final native proof remains separate.
