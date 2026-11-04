# Direct Secret Lookup Tasks

## Implementation

- [x] 1.1 Replace domain secret references and binding wire types with direct declarations and availability; remove per-plugin secret operation.
- [x] 1.2 Replace runtime preferences, initialization and execution resolution; discard saved references and preserve v1 storage; provide typed required-secret failure.
- [x] 1.3 Derive secret catalog references from declarations, including inactive installed programs; migrate server adapters and all Rust consumers/tests.
- [x] 1.4 Update macros, starter and SDK examples; remove environment secret handling and obsolete binding tests.
- [x] 1.5 Replace GUI binding fields/mutations/contracts with read-only declared secrets and existing Add Secret dialog prefill; migrate GUI stories.
- [x] 1.6 Regenerate Python and GUI contracts through project tasks and migrate affected client consumers.
- [x] 1.7 Update README, product guide, GUI guide and authoring skill/copies; review external example-plugin migration without editing it.

## Validation And Delivery

- [x] 2.1 Run focused runtime tests for existing, missing typed identifier, optional absence and Retry after adding; run focused GUI declaration/prefill tests.
- [x] 2.2 Run workspace check, format, TypeScript check/lint and Python lint/typecheck through mise tasks with UV_PROJECT_ENVIRONMENT removed.
- [x] 2.3 Validate OpenSpec, review complete diff and commit locally using Conventional Commits; do not push.
