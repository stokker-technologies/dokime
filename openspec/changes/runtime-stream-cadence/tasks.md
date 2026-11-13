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

## Validation and Delivery

- [x] 2.1 Validate specifications; run targeted plugin-host and SDK tests through focused mise task.
- [x] 2.2 After source closure, run requested workspace check and format.
- [x] 2.3 Commit owned Dokime changes without local override or path-resolved lockfile; do not push.

## 3. Released Dependency Proof

- [x] 3.1 Pin published Harmos 0.1.14 in workspace declarations and lockfile with no local override.
- [ ] 3.2 Validate cadence consumers against the unpatched published dependency graph through focused native tasks; migrate the host-tick cadence in the gRPC adoption without restoring stream-owned intervals.

## Workflow Follow-Up

- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- Run only focused tasks until combined source closure; final native proof remains separate.
