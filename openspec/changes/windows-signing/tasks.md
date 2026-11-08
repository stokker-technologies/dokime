## Implementation

- [ ] 1.1 Implement pre-minisign Authenticode and skipped-signing proof. Locked ownership: `crates/dokime-packaging/src/installer/signing.rs`, `authenticode.ps1`, `native.rs`, packaging tests and `.github/actions/sign-windows/action.yml`. Preserve existing manifest signing and updater semantics.
- [ ] 1.2 Implement machine-wide selection with unchanged manifest and per-user default. Locked ownership: `crates/dokime-packaging/src/installer/installer.nsi`, `hooks.nsh`, `native.rs` and packaging tests. Verify scope, privilege checks, registry and PATH selection.
- [ ] 1.3 Complete `docs/deployment.md` with secrets, exact release wiring, detection, silent commands, upgrade/rollback, Linux verification and network/privilege qualification. README reviewed; adding its guide link belongs to the orchestrator because README is outside ownership.

## Validation

- [ ] 2.1 Run packaging focused tests and focused Cargo check through mise.
- [ ] 2.2 Run format, project check, one final lint:rust and commit scan; report every failure and unavailable certificate/native-platform proof.

## Workflow Follow-Up

The orchestrator owns release/nightly workflow edits, combined batch validation and delivery. This worker commits only its owned paths and never pushes or opens a PR. Native Linux/Windows managed installation and actual certificate signing remain qualification obligations.
