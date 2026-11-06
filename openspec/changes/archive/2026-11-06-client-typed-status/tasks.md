## Implementation

- [x] 1.1 Update `client/query/discovery.py` and `client.py`: typed statuses, enum filters, cycle status and running count; preserve existing string filters and finite snapshots.
- [x] 1.2 Add `client/query/schemas.py` from the Python skeleton: lazy catalog paging, revision/identity checks, ordered immutable record schema/field/value-type models.
- [x] 1.3 Update client/package exports and package stubs; update package/root READMEs and `docs/remote/python.md` with typed filters, busy refusal and declared-column discovery.
- [x] 1.4 Extend fake-transport discovery tests for every status, filters, counts, lazy schema paging, declaration fidelity and invalid catalog replies; run focused tests.

## Validation And Delivery

- [x] 2.1 Run Python format, lint and typecheck through mise tasks with `UV_PROJECT_ENVIRONMENT` removed; require no new typing warnings. Run generated-client check if contracts change.
- [x] 2.2 Review the complete diff and commit the requested Conventional Commit after the query commit; do not push. Retain both owner-requested change artifacts for review.

## Workflow Follow-Up

- Owner decision on 2026-11-01 extends `Query.ListCycles` additively, preserving finite snapshots; `query-cycle-status` owns domain/runtime/server and generation work. This remains the worktree's active specification, incorporating the tightly coupled extension.
- Source inventory includes all existing client edits and the query extension in the current branch. No other writers, coupled repositories or unresolved API decisions remain. Query generation also receives its missing standard compiler-output preflight in `tools/python/src/dokime_tools/prepare/generated.py`.
