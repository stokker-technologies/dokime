## Implementation
- [x] 1.1 Runtime/settings: retain running-generation values, execution snapshots and launch seeds, compare saved fields and remove restart-on-save.
- [x] 1.2 Operations/server/clients: replace plugin Retry with Reload for installation/program targets and regenerate bindings.
- [x] 1.3 Lifecycle/observers: ensure complete observer stop/restart and new cadence; retain pending/refused admission.
- [x] 1.4 GUI: status column, direct actions, page Reload and backend-owned pending field highlights; update consumers.
- [x] 1.5 Tests: save without restart, differences across view reads, reload clears differences, active-cycle pending and plugin observer cadence.
- [x] 1.6 Documentation: update GUI/plugins/product guides and descriptions; review README impact and remove superseded behavior.

- [x] 1.7 Tooling: admit the already supported run operation in the focused task parser to build the required native fixture.

## Validation And Delivery
- [x] 2.1 Strict OpenSpec validation, touched TypeScript unit tests, GUI checks, focused Rust tests, check:rust and check:typescript:generated; no full simulation.
- [x] 2.2 Review diff and forbidden terms; include regenerated contracts and embedded resources in the candidate.
