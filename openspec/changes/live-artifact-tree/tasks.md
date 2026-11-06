# Tasks

## Implementation

- [x] 1.1 Update runtime artifact path allocation, UUID identity and collision validation in runtime/storage/artifacts.rs and native output consumers, including native_outputs/reader.rs and documents/follow.rs media attribution.
- [x] 1.2 Publish Stream captures live; exclude untouched baselines, preserve new empty files and rotation segments in domains/files/watch.rs; update harness producer parity.
- [x] 1.3 Add deterministic native/harness proofs for baselines, append/new files, live growth/completion, uploads, collisions and outcomes/report consumers; reuse GUI viewer, refresh execution files on operation changes in app/live and features/cycle, show Live/Finished/Interrupted tree states in components/ArtifactTreeView.tsx, and test path/state and refresh behavior; make the FTP fixture await observable watch passes rather than assuming each recoverable pass advances.
- [x] 1.4 Update docs/product-guide.md and docs/gui.md; review README impact, refresh the shared generated dist-report/report.html, and generate bindings only if wire shapes change.

## Validation And Delivery

- [x] 2.1 Run focused native/harness tests, GUI unit tests, check:rust and check:typescript:generated; strict OpenSpec validation; scan diff for forbidden owner terms. No full simulation, as requested.

## Workflow Follow-Up

- Archive after implementation and the requested focused proof; commit and push, verify the remote SHA, create the authorized PR to dev and squash immediately using the requested subject.
- Clean the owned compiler target with Cargo and retire the owned worktree and branch after exact-review integration verification.
