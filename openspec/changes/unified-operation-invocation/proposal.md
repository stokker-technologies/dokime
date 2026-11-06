# Unified Operation Invocation

## Why

Dokime templates currently split declared calls and built-ins between `ctx.call` and `ctx.host()`. Adopt Harmos's shared SDK invocation while preserving authorization, stages, harness execution and evidence.

## What Changes

- Use `ctx.operation(request).await?` returning `Response<T>` on templates, criteria and observers; `.start()` retains owned instance behavior.
- Adopt `OperationHost` and `StepOperationHost`, preserve `Called<C>` and existing built-in exemptions, and retire request-taking calls and template host access.
- Carry configured call-site retry through host routes without raising declared deadlines or duplicating macro policy; record every attempt and reject unsafe overrides.
- Migrate all source, starter, docs/plugins.md reference examples and identical authoring reference copies, replacing affected generated.lock digests only.

## Impact

Producer: paired Harmos OpenSpec change and worktree `../harmos.wt/invocation`. Use a local Cargo override only for the minimal template proof; Harmos lands first and the delivered graph pins its verified merged SHA. Rebase on fresh dev before migration and delivery. Do not edit the private plugin repository. Validation is focused proof, check:rust and lint, with no full simulation; delivery is an immediately squash-merged PR into dev and owned-worktree cleanup.
