## Implementation

- [x] 1.1 Native Session: connect backup destination/root IPC, enforce failed local start and occupied-root refusal, retain archive result.
- [x] 1.2 Runtime Plugins: capture installed bundle files and readable metadata before clear without resurrecting orphan or removed installations, restore enabled state/valid settings and secret identity/catalog, tolerate refused bundles.
- [x] 1.3 GUI: expose recovery with explicit path confirmation, existing session lifecycle, archive result and reinstall progress; regenerate shared embedded report resources.
- [x] 1.4 Stories (GUI storage/backends unit files, native integration/storage.rs and runtime integration/plugins/recovery.rs): deterministic backup failure, reinstallation/state/settings, orphan/removal exclusion, refusal/restart and GUI BackendRunning/confirmation scenarios.
- [x] 1.5 Documentation: update docs/gui.md and review README impact.

## Targeted Validation

- [x] 2.1 Run touched TypeScript unit stories and GUI checks.
- [ ] 2.2 Run focused Rust recovery tests and check:rust.
- [x] 2.3 Review diff and ensure forbidden-term diff search is empty.

## Delivery

- [ ] 3.1 Finish and archive the agreed change before committing the candidate.
