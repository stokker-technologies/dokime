# Tasks

## Implementation Inventory

- [x] 1.1 Domain/execution: mirror the document index and context handle; validate and upsert sections/rows; preserve step-only capability and captured criteria boundary.
- [x] 1.2 Runtime/macros/transport: remove hook/context/future/phase and all generated callers; preserve buffered capture and durable ordered section edits.
- [x] 1.3 Producers/fixtures: migrate OBC, cargo-dokime starter, authoring fixtures and tests; fold testing report edits by identity.
- [x] 1.4 Consumers: review storage/recovery, server, GUI, reports, plots and Python; migrate any single-capture assumptions while retaining read wire shapes.
- [x] 1.5 Guidance: update README, topic guides and dokime-authoring source; synchronize .claude/.github copies byte-for-byte; remove superseded hook guidance.

- [x] 1.6 Repair the packaging acceptance story's policy include to read the existing `.stkr/project.toml`, matching Python tooling and preserving its configured deadline; required workspace compilation exposed the missing root `stkr.toml` include.

## Owner-Authorized Validation And Delivery

- [x] 2.1 Run `mise run format:rust` with UV_PROJECT_ENVIRONMENT removed.
- [x] 2.2 Pass `env -u UV_PROJECT_ENVIRONMENT mise run build:rust:focused -- check --workspace --all-targets`.
- [x] 2.3 Commit the completed feature with Conventional Commit naming; do not push or create a PR.
