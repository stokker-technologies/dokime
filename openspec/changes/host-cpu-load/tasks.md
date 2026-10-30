# Host CPU Load Tasks

## Implementation

- [x] 1.1 Record the locked design and amendment; inspect current branch, resources and pinned Harmos cancellation semantics.
- [x] 1.2 Define and validate LoadHold, LoadTarget, LoadSummary and the control evidence stream in plugin-host; expose existing calibration only.
- [x] 1.3 Centralize configuration and extract the counted calibration kernel for duty workers.
- [x] 1.4 Implement shared fixed-cadence CPU sampling; migrate both existing readers and prove concurrent reads preserve samples.
- [x] 1.5 Implement PI control, anti-windup, duty workers, weighted summaries and prompt worker shutdown with focused tests.
- [x] 1.6 Use Harmos 4e2669fa cooperative cancellation with owner-approved Option<u32> threads; wire Public dispatch, control publication, lifecycle settlement and summary sample count.
- [x] 1.7 Add an OBC start/hold/cancel usage template and affected suite tests; update docs/plugins.md, docs/plugin-obc.md and Dokime authoring guidance; review README impact (no root claim changes needed).
- [x] 1.8 Integrate all source on feat/host-cpu-load and declare SOURCE COMPLETE after focused proof; no other writers are assigned.

## Final Validation And Delivery

- [x] 2.1 Run workspace check-only proof and formatting through project tasks, focused plugin-host tests and affected OBC tests.
- [x] 2.2 Run openspec validate host-cpu-load --strict and assess locked scenarios.
- [ ] 2.3 Commit logical items with Conventional Commits; report actual checks and unresolved work, without push or PR.

## Workflow Follow-Up

Only this worktree's owner edits these files. Source is complete on feat/host-cpu-load; definitions, runtime, consumers and guidance are combined and no writers remain.
Owner resume resolves upstream blockers: merge origin/dev, use Option<u32>, bound workers to available logical cores and return the typed final summary with sample count. Keep this change tracked for review as explicitly requested. No release builds/installers or full lanes.
