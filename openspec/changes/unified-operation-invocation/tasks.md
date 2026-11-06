# Implementation Tasks

## Source Inventory

- [x] 1.1 Rebase owned invocation worktree on fresh origin/dev, inspect authoring/calls and record locked producer/consumer contracts.
- [ ] 1.2 Prove one declared template invocation in tests/invocation.rs (registered integration target) with the local Harmos candidate before migrating remaining calls.
- [ ] 1.3 Migrate execution/{calls,instances,context/authoring,mod}.rs to StepOperationHost and shared Invocation; preserve stage, readiness, cancellation, built-ins and harness behavior.
- [ ] 1.4 Preserve Called authorization and built-in exemptions through macros/catalogs; migrate criteria, observers and all trait implementers/bounds.
- [ ] 1.5 Migrate source, examples, tests and starter; remove request-taking call, template host access and superseded code without aliases.
- [ ] 1.6 Migrate docs/plugins.md reference steps, product docs and guest/starter guidance; synchronize authoring references and affected generated.lock digests without project update.
- [ ] 1.7 Pin merged Harmos, remove local override, resolve lockfiles, adapt grpc config across #110 and refresh bindings if affected.
- [ ] 1.9 Persist each Harmos operation attempt in the execution log through the awaited middleware callback, with stage, instance, attempt and outcome.
- [ ] 1.8 Focused proof: declared/undeclared template operations, built-ins, stage routing, cancellation, configured retry/defaults and attempt evidence.

## Final Validation And Delivery

- [ ] 2.1 Rebase on fresh dev, migrate newly landed calls and close the full consumer source inventory.
- [ ] 2.2 Run check:rust and lint, inspect final diffs and scan prohibited names.
- [ ] 2.3 Push and verify remote SHA, open ready PR and squash immediately without waiting for CI.
- [ ] 2.4 Finish OpenSpec, preserve evidence, cargo clean and retire the exact integrated worktree and branch.
