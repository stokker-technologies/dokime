## Slice 1: Static Structure and Query Summaries

- [x] 1.1 Runtime documents/current, document/bounded/binding/resolve: static structure and query-backed summaries/statistics; preserve plot path.
- [x] 1.2 Remove estimated row-work admission and source-row ceilings; enforce actual output/expansion budgets with card diagnostics.
- [x] 1.3 Add deterministic short-run large-document, beyond-old-limits and exact-statistics proof; review README and update product/query guides.
- [x] 1.4 Run focused proof, check:rust and bindings generation if wire contracts change; scan diff for forbidden terms.

## Slice 2: Independent Cards

- [x] 2.1 Domain/operations/runtime, storage numeric-index reads and server wire/value output contracts: independently query cards at a fixed cut, reuse lossless existing indexed statistics and isolate slow/failed/oversized cards.
- [x] 2.2 GUI document API and viewer: immediate structure, per-card loading/retry/ErrorCard; update docs/gui.md and product guide.
- [x] 2.3 Deterministic independent completion and budget isolation tests; focused Rust/GUI proof, check:rust, prepare:bindings and tracked outputs.

## Slice 3: Indexed Evaluation

- [x] 3.1 Record/query ownership: replace nested TimeBetween and PerCase scans with linear sorted merge/partition evaluation and deterministic tests.
- [x] 3.2 Storage ingest/query and document runtime ownership: distinct-value index and paged repeat expansion, domain/operation/server continuations and GUI page controls; preserve complete distinct values and fixed cuts.
- [x] 3.3 Reuse/extend maintained count/min/max/mean/RMS at ingest; exact median query; prove raw-row equivalence and update affected guides.
- [x] 3.4 Focused tests, check:rust, GUI tests/bindings if touched and diff scan.

## Slice 4: Frozen Reports and Benchmark

- [x] 4.1 Document query/repeat/page, indexed record/distinct storage, report scope/materialization and runtime query/view owners: share storage/configuration-backed card evaluation at sealed cuts, drain repeated pages and retain full authored tables. Retire raw acquisition, RuntimeDocumentData/Input, resolve_cycle_document, the obsolete Document view/frame and their superseded tests; migrate meaningful scope/layout behaviors to current captures. Bound live record summaries and indexed repeats by the prepared journal epoch/revision as well as the physical evidence cut.
- [x] 4.2 Add deterministic sealed report parity proof and update product/GUI/query guides and affected README claims.
- [x] 4.3 Integration-runtime bench inventory, bounded real-ingest fixture and shared storage resource probe: three days, several million samples, 30 plots, structure/card latency including plot payloads and peak-memory scaling with Linux and Windows RSS support. Preserve native Criterion/resource outputs.
- [x] 4.4 Run focused proof, check:rust and applicable GUI/binding checks; run requested benchmark and scan diff.

## Workflow Follow-Up

- For each slice, push its proved candidate, verify remote SHA, open its ready PR to dev and immediately squash-merge with the owner's specified subject, without waiting for CI and without full local simulation.
- After each of the first three merges, branch the next slice from freshly fetched origin/dev in this worktree and rebind the same specification marker from its exact previous branch.
- Include strict specification archival with the completed final slice; after integration preserve proof, cargo clean the owned target, and retire the worktree and local task branches through project housekeeping.
- Final report stays under 200 words and contains each slice's PR and merge SHA, benchmark numbers and remaining obligations.
