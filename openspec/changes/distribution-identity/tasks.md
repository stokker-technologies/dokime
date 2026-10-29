# Tasks

Locked 2026-10-29; deferred until the owner schedules it.

## 1. Implementation

- [ ] 1.1 Installer manifest `[installer] identifier` with validation; payload identity checks accept the distribution identifier.
- [ ] 1.2 `cargo dokime installer package` writes `distribution.json` into the payload inventory and uses the identifier for the NSIS install folder, uninstall registry key and DEB/RPM package names.
- [ ] 1.3 One path resolver (desktop, CLI, server) reading `distribution.json` with the compiled identifier as fallback; replace `app_data_dir()` and `app_log_dir(identifier)` uses; set the WebView data directory.
- [ ] 1.4 Product guide: Native Installers and Organization sections; README claims.
- [ ] 1.5 Tests: resolver unit tests, packaging tests for the written file and names, a native lifecycle check that a branded and a plain installation keep separate directories.

## 2. Validation

- [ ] 2.1 Targeted proof, then the native desktop walkthrough on Windows and Unix with a branded distribution.
