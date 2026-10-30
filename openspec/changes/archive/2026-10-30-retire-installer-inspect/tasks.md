# Tasks

## 1. Implementation

- [x] 1.1 Restore stock NSIS solid compression in `crates/dokime-packaging/src/installer/installer.nsi`; preserve deliberate identity, display and PATH behavior.
- [x] 1.2 Remove `installer/inspect.rs`, its public exports and native assembly caller; preserve startup inventory and relocate signing helpers to their owner. Delete inspector-only dependencies and tests.
- [x] 1.3 Remove cargo-dokime's inspect subcommand and the Python adapter's post-build inspection and output files; retain installer package and sidecar inspect.
- [x] 1.4 Remove obsolete product-guide inspection instructions, review README and Dokime skills, and reconcile superseded active distribution-identity guidance.
- [x] 1.5 Record the owner decision in the organization-installers delta, including Tauri/NSIS defaults and removal of inspection obligations.

## 2. Validation And Handoff

- [x] 2.1 Run workspace/all-targets check with UV_PROJECT_ENVIRONMENT unset, Rust formatting and remaining packaging installer unit tests through project tasks; do not build or package installers.
- [x] 2.2 Run strict OpenSpec validation, archive with `openspec archive retire-installer-inspect --yes`, retain the archive and fill placeholder Purpose text.
- [x] 2.3 Review the final diff and prepare only owned changes for a local Conventional Commit; no push or PR.
