# Tasks

## Source Inventory

- [x] 1.1 Migrate host settings.rs, six host/*/observing.rs owners and preparation resources; share declared and settings defaults in settings.rs.
- [x] 1.2 Migrate host domain telemetry contracts, preserving schema and replay windows.
- [x] 1.3 Migrate Dokime SDK integration, macros, runtime, OBC, starter templates and all affected tests/examples.
- [x] 1.4 Add meaningful settings and controlled-clock cadence tests; audit unchanged operation policies and other constants.
- [x] 1.5 Update product guide and applicable authoring/architecture guidance; review README impact.
- [x] 1.6 Review paired integration and remove obsolete cadence paths; all writers hand off.
- [x] 1.7 Migrate domains/telemetry/leases.rs to authoritative observer inventory; preserve published-stream behavior, update lease/capture tests and finalization benchmark fixture cadence.

- [x] 1.8 Correct observer defaults: required validated interval_ms, optional activation override, migrate all consumers, add fallback/override validation coverage and update maintained docs/specification; host defaults remain settings-owned.

## Validation And Delivery

- [x] 2.1 Validate specifications; run targeted plugin-host and SDK tests through focused mise task.
- [x] 2.2 After source closure, run requested workspace check and format.
- [x] 2.3 Commit owned Dokime changes without local override or path-resolved lockfile; do not push.

## Workflow Follow-Up

- [ ] 3.1 After upstream publication, pin verified Harmos, remove local override, resolve the published lockfile and validate the unpatched graph. **Remaining:** Cargo.toml and Cargo.lock use an immutable Harmos git revision, with no local Cargo override present. The requested subsequent published release pin and unpatched released-graph validation are still pending.
