## Implementation Inventory

- [x] 1.1 Domain/client: replace file schema and fixtures, remove old extension/CLI flag consumers, preserve strict validation and secret bounds.
- [x] 1.2 Server remote store: persistent CA, signed leaves, automatic interface discovery/ranking, reissue on name changes with same CA; targeted tests.
- [x] 1.3 Client/profile/settings: staggered verified endpoint race, remembered ordering, stable keyring association, import rollback; targeted tests.
- [x] 1.4 Native desktop and GUI: one export action, listener-off confirmation/enable, generated labels, rename, secret warning, native import/drop and automatic connect; targeted fakes.
- [x] 1.5 CLI and Python: --remote import, equivalent endpoint negotiation and remembrance, migrate all examples and tests.
- [x] 1.6 Packaging: NSIS private-network application inbound rule and uninstall removal, preserve Tauri defaults.
- [x] 1.7 Bindings/configuration/docs: regenerate affected contracts using tasks; update product guide, Python guide/examples, CLI table and affected README claims; reconcile superseded remote contracts, retire completed OpenSpec archives and preserve unfinished installer proof; apply canonical formatting and remove compiler-profile overrides reported by the repaired checker.
- [x] 1.8 Hub tooling: repair nontransitive task ordering and header validation of Rust template identifiers; validate focused regressions and install source checkpoints through the native local override task.

## Final Validation And Local Delivery

- [x] 2.1 Run requested focused deterministic tests for format, ranking, verified race, remembrance, CA reissue, export enablement and profile/keyring import.
- [x] 2.2 Run workspace check/format, TypeScript check/lint, Python lint/typecheck through mise tasks with UV_PROJECT_ENVIRONMENT unset; repair failures.
- [x] 2.3 Review full diff and strict OpenSpec validation; commit all owned work in atomic Conventional Commits without pushing; report actual proof and open issues.

## Workflow Follow-Up

- Owner decision: restore the declared GitHub dev required-status-check protection (strict `CI / Required`, GitHub Actions app 15368), then rerun the provider audit. This unresolved obligation remains outside the authorized local-only feature delivery and is not waived.
