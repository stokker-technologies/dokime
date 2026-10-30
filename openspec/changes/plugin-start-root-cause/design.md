# Design

## Agreed Contract

Harmos owns startup classification: within one prepare pass, Runtime preparation/construction failures are roots and all MissingResource errors are consequences regardless of order. Other errors stay roots; without a construction failure missing resources are roots. No dependency graph or string matching is introduced.

## Locked Inventory

- Harmos producer: ../harmos.wt/rootcause, branch fix/startup-error-root-causes; SDK report/carrier/classification, tests and guide.
- Dokime consumer: runtime startup mapping and shared CLI/error Display, failure wire/view projection, native adapter, notifications/history, plugin card/settings, Secret Storage Add Secret dialog, focused Rust/frontend tests, embedded report resource, docs and README.
- Reuse #114 editable forms and restart retry. Set Secret opens Secret Storage with the identifier prefilled; no stored-secret picker or new binding mechanism.
- Generated clients follow the domain shape through existing generation tasks only. No hilti-hcl changes, committed local override, local-path lock resolution or dependency pin adoption.

## Integration And Adoption

The temporary untracked .cargo/config.toml patches Harmos packages to ../harmos.wt/rootcause. Dokime still needs the separately owned chore/adopt-harmos-dev port of e9d86d8b (including cancellation_timeout). Merge origin/dev only once that adoption lands; do not duplicate its changes. Generated client/type checks require a matching task-built cargo-dokime export. Final published Harmos pin adoption belongs to the orchestrator after both commits land.

## Remaining Proof

Workspace check and the focused runtime test stop at dokime-operations/src/lib.rs:210 and src/runtime/mod.rs:30: OperationPolicy requires cancellation_timeout from the separate adoption port. Generated TypeScript checking stops at the missing staged Product generated modules before tsc. After adoption, regenerate through the task-built cargo-dokime exporter, run the full workspace check and focused runtime/store/settings tests, then check generated TypeScript. Do not count conceptual compatibility or passing frontend mocks as compiled runtime proof.
