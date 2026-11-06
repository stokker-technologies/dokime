# Implementation Tasks

## Source Inventory

- [x] 1.1 Merge fresh origin/dev into the owned invocation worktree, inspect authoring/calls and record locked producer/consumer contracts.
- [x] 1.2 Prove one declared template invocation in tests/invocation.rs (registered integration target) with the local Harmos candidate before migrating remaining calls.
- [x] 1.3 Migrate execution/{calls,instances,context/authoring,mod}.rs to StepOperationHost and shared Invocation; preserve stage, readiness, cancellation, built-ins and harness behavior.
- [x] 1.4 Preserve Called authorization and built-in exemptions through macros/catalogs; migrate criteria, observers and all trait implementers/bounds.
- [ ] 1.5 Migrate source, examples, tests and starter; remove request-taking call, template host access and superseded code without aliases.
- [x] 1.6 Migrate docs/plugins.md reference steps, product docs and guest/starter guidance; synchronize authoring references and affected generated.lock digests without project update.
- [x] 1.7 Pin merged Harmos, remove local override, resolve lockfiles, adapt grpc config across #110 and refresh bindings if affected.
- [x] 1.9 Persist each Harmos operation attempt in the execution log through the awaited middleware callback, with stage, instance, attempt and outcome.
- [x] 1.8 Focused proof: declared/undeclared template operations, redundant built-in requirements, built-ins, stage routing, cancellation, configured retry/defaults and attempt evidence.

## Final Validation and Delivery

- [ ] 2.1 Merge fresh origin/dev again, migrate newly landed calls and close the full consumer source inventory.
- [ ] 2.2 Run focused sidecar/guest proof, starter test, check:rust, lint and repository checks; inspect final diffs and scan prohibited names.
- [ ] 2.3 Validate OpenSpec and prepare the final authorized ready-PR candidate and review description.

- [x] 2.4 Keep the known repository-wide convention baseline out of scope under the owner's delivery decision; document it in the PR.
