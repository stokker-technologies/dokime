# Tasks

Locked 2026-10-29; scheduled by the owner. Implementation in `feat/distribution-identity`.

## Source Inventory

Installer identity and native assembly: `dokime-packaging/src/installer/{manifest,payload,native,inspect}.rs`, identity-only hooks and packaging tests. Plugin selection/staging remains with the host-plugin-features owner.

Shared identity validation: `dokime-domain/src/model/identity/distribution.rs`. Shared resolver and consumers: `dokime-runtime/src/paths.rs`, configuration snapshots and consumers (runtime settings, server diagnostics/log streams), desktop bootstrap/organization/settings/logging/local backend and native-session paths and cache consumers; server logging, CLI server startup and `dokime/src/installation.rs` GUI discovery. Preserve compiled protocol and updater identities.

Proof and guidance: resolver tests, installer tests, native desktop lifecycle identity coverage, product guide and README. Implement the shared identity contract before packaging and consumer integration. Native Windows and Unix walkthroughs remain separate validation obligations; this slice runs targeted proof only and never pushes.

Triage: `ApplicationPaths` already owns basic root/data/plugins paths (`crates/dokime-runtime/src/paths.rs:34`); extend it. Existing `distribution.json` is the inspection inventory (`crates/dokime-packaging/src/installer/inspect.rs:138`); replace its role with a separate inventory so the startup identity file itself is verified. None of 1.1–1.5 is already complete.

## 1. Implementation

- [x] 1.1 Installer manifest `[installer] identifier` with validation; payload identity checks accept the distribution identifier.
- [ ] 1.2 `cargo dokime installer package` writes `distribution.json` into the payload inventory and uses the identifier for the NSIS install folder, uninstall registry key and DEB/RPM package names.
- [x] 1.3 One path resolver (desktop, CLI, server) reading `distribution.json` with the compiled identifier as fallback; replace `app_data_dir()` and `app_log_dir(identifier)` uses; set the WebView data directory.
- [x] 1.4 Product guide: Native Installers and Organization sections; README claims.
- [ ] 1.5 Tests: resolver unit tests, packaging tests for the written file and names, a native lifecycle check that a branded and a plain installation keep separate directories.

## 2. Validation

- [ ] 2.1 Targeted proof, then the native desktop walkthrough on Windows and Unix with a branded distribution.

## Workflow Follow-Up

- Implementation for 1.2 and coverage for 1.5 are present, but their closure remains blocked by an unexplained Windows NSIS inspection failure: `none of the supported compression methods succeeded`. Two subsequent current-fixture runs passed without a production decoder or compression change. Preserve this as a repair obligation; do not waive it as unrelated or infer that rerunning explains it. A failing fixture is retained on future failures for diagnosis.
- Owner question: if diagnosis confirms a decoding dependency defect, authorize that repair beyond this identity slice or assign it to the decoder/installer owner. No dependency replacement, compression workaround or weakened inspection was introduced here. Native Windows and Unix walkthroughs remain open under 2.1.
