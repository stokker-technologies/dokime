# Host CPU Load Tasks

## Implementation

- [x] 1.1 Record the locked design and amendment; inspect current branch, resources and pinned Harmos cancellation semantics.
- [ ] 1.2 Define and validate LoadHold, LoadTarget, LoadSummary and the control evidence stream in plugin-host; expose existing calibration only.
- [ ] 1.3 Centralize configuration and extract the counted calibration kernel for duty workers.
- [ ] 1.4 Implement shared fixed-cadence CPU sampling; migrate both existing readers and prove concurrent reads preserve samples.
- [ ] 1.5 Implement PI control, anti-windup, duty workers, weighted summaries and prompt worker shutdown with focused tests.
- [ ] 1.6 Resolve the pinned Harmos cancellation and u16 limitations without silently changing the locked response contract; wire Public dispatch and lifecycle settlement.
- [ ] 1.7 Add an OBC start/hold/cancel usage template and affected suite tests; update docs/plugins.md, docs/plugin-obc.md and Dokime authoring guidance; review README impact.
- [ ] 1.8 Integrate all source on feat/host-cpu-load and declare SOURCE COMPLETE after focused proof; no other writers are assigned.

## Final Validation And Delivery

- [ ] 2.1 Run workspace check-only proof and formatting through project tasks, focused plugin-host tests and affected OBC tests.
- [ ] 2.2 Run openspec validate host-cpu-load --strict and assess locked scenarios.
- [ ] 2.3 Commit logical items with Conventional Commits; report actual checks and unresolved work, without push or PR.

## Workflow Follow-Up

Only this worktree's owner edits these files. Implement definitions/configuration/kernel/sampling/control before dispatch and consumer wiring.
The upstream cancellation decision is pending; independent source work continues. Keep this change tracked for review as explicitly requested.
