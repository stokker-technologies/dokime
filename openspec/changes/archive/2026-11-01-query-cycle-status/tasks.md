## Implementation

- [x] 1.1 Single owner: extend domain query page/selection and operation validation; runtime SQLite history captures cycle lifecycle and counts Running executions at the same cut, preserving phase selection. Publish the admitted cycle header before provider dispatch in `runtime/command/handle.rs` so durable discovery includes queued/running work without query-side flushing.
- [x] 1.2 Regenerate descriptors and Python/TypeScript bindings only through `mise run prepare:bindings`; migrate all affected Rust constructors. Restore the standard compiler-output preflight in generation and Rust lint tasks; make the generated TypeScript check consume its freshly checked prerequisite instead of restoring stale Product artifacts. Task fixes stay in `mise.toml` and `prepare/generated.py`.
- [x] 1.3 Add focused runtime snapshot/filter/count and server transport/validation tests. Targeted execution and final Rust checks remain in 2.1.
- [x] 1.4 Complete dependent Python discovery, exports, stubs, tests and README/remote guide in `client-typed-status`; root README impact reviewed there. No product settings changes or superseded paths required; validation task fixes are owned by 1.2.

## Validation And Delivery

- [x] 2.1 Run Rust check, generated-client checks, focused Rust tests and strict OpenSpec validation; client change owns Python proof.
- [x] 2.2 Review and create the query commit followed by the client commit; do not push. Keep these owner-requested change artifacts for review.
