# Imported Executions Design

## Ownership and Dependency Order

`grpc-plugins/design.md` owns execution, origin, outcome, schema freezing and
render versus publish. Its host manager is not built yet and must start on those
contracts. This change supplies import-specific behavior after its SDK wave and
`storage-interface`. Protocol owns wire definitions; operations own validation,
policy and handlers; runtime owns lifecycle and transactions; storage owns
persisted shapes, staged publication and conditional updates. Rendering retains
temporary output ownership; SDKs depend on protocol, never host storage.

The imported execution stays open in draft storage until seal. Successful seal
closes evidence and freezes the document/report before publishing the sealed
execution and membership atomically. This uses execution open/closed/sealed,
without inventing run phases or a second execution-definition type. Recovery
can resume or reclaim staged work and retain draft receipts; it cannot run the
interrupted-run finalizer against drafts.

A seal revalidates the target cycle's admission and final document at the expected
draft revision. Any failure preserves the draft and prior cycle contents. Cycle
creation/membership must be in the seal publication boundary; preparing a cycle
selector during upload must not expose a provisional member or empty new cycle.
An active preview reads a pinned revision; successful replacement advances the
handle revision without changing that snapshot. Successful mutation receipts
outlive transient handles and must remain usable for retries after restart;
receipt retention/compaction must not turn an acknowledged retry into a new write.

## Open Points Before Implementation

These are gaps in the locked decisions, not additional locked policies. Each
has an explicit task; dependent implementation waits for its resolution.

| Review Issue | Decision Still Required | Task |
| --- | --- | --- |
| 4 | Define stable cycle selectors and atomic name resolution/create; decide ambiguity behavior for duplicate labels and what happens if the target closes between draft creation and seal. Closed admission must never be bypassed. | 1.2 |
| 11 | Define retention of disorder/equal timestamps, timestamp tie order and gap presentation. Durable positions exist, but the model has not locked duplicate identity or first/last ordering. | 1.3 |
| 12 | Choose an import default plot window and outlier presentation for wide source coverage; retain typed query-limit failures without silent truncation. | 1.4 |
| 23 | Decide document-specific projection preparation/backfill versus bounded raw-query fallback for arbitrary preview and final documents, including unsupported-query failures. | 1.5 |
| 24 | Reconcile inherited bounded-materialization wording with plot-pipeline costs; choose preview cancellation, cache policy, host-owned work/concurrency budgets and exhaustion behavior. Repeated preview has no promised cheapness. | 1.6 |
| 26 | Define compatible-unit checks, calibration/identity separation, quality counts and sample/time weighting, duplicate and first/last statistical semantics across imports. Equal field names alone cannot establish equivalence. | 1.7 |

## Review Issue Dispositions

The issue numbers refer to the owner's 30-item risk review. This table is the
maintained disposition; no external scratch file is required to implement it.
`G` denotes `grpc-plugins/tasks.md`; `I` denotes this change's `tasks.md`.

| Issue | Risk | Decision or Explicit Open Point | Owning Task |
| --- | --- | --- | --- |
| 1 | Execution requires a case | Execution is the evidence container; Origin freezes TestCase or Import without fake plugin identity. | G 2.9, 3.12 |
| 2 | Manager lacks import lifecycle | Live run state is manager-local; import admission bypasses its queue. | G 3.3, 3.12; I 2.1 |
| 3 | Missing outcome becomes Passed | Attributed outcome; no evaluated results never yields cycle Passed. | G 3.13 |
| 4 | Cycle labels are ambiguous | Open: stable selection, name resolution and close race above. Membership publishes only at seal. | I 1.2, 2.4 |
| 5 | Mixed-cycle verdict misleads | Fold evaluated only; count declared/unevaluated separately in summaries and reports. | G 3.13; I 4.1 |
| 6 | Step/attempt traceability | Allow marked importer traceability; refuse step/attempt references and criteria. | I 2.2, 4.2 |
| 7 | Run controls on imports | Backend states start/rerun/cancel/inspect/export capabilities; desktop shows only those. | G 3.14; I 4.1 |
| 8 | Source and host clock conflict | Each sample retains UTC source time, receipt time and durable position; stream declares axis. | G 3.4; I 3.1 |
| 9 | History time conflates measurement | History orders/filters imports by import time; measurement period is separate source coverage. | I 3.1, 4.1 |
| 10 | Mixed clocks corrupt correlation | Require absolute UTC or UTC start plus unit-bearing elapsed array; reject unmapped clocks. Reset clocks must be normalized by the script. | I 3.1 |
| 11 | Disorder/duplicates/gaps | Open: ordering, duplicate identity and gap policy above; timestamps alone are not a locked deduplication rule. | I 1.3, 3.3 |
| 12 | Windows hide imported history | Open: default window, outliers and limit behavior above. | I 1.4, 3.3 |
| 13 | Live/follow completion undefined | Upload has no visible execution; committed draft revision has a finite frontier; seal publishes final visibility. Follows cannot track a replacement revision implicitly. | I 2.1, 3.3 |
| 14 | Run recovery closes drafts | Recovery closes interrupted runs' executions only; drafts have distinct expiry and staging recovery. | G 3.12; I 2.5 |
| 15 | Visibility differs by surface | Draft handle only; no ordinary history/search/cycle/export or other-client visibility before seal. | I 2.4, 4.1 |
| 16 | Retry overwrites replacement | Store-scoped import key differs from request identity; revision conditions and durable success receipts prevent replay writes. | I 2.3 |
| 17 | Preview/seal/expiry races | Pin preview revision against expiry; conditional replacement/seal/discard; stale handles conflict. | I 2.3, 2.5 |
| 18 | Restart versus fresh root | Store identity scopes keys, handles and receipts; restart retains them, new root has a new namespace and permits cross-store duplicates. | I 2.3, 4.3 |
| 19 | Discard leaks storage | Independent draft storage unit includes attachments, columns, projections and metadata; reclaim on discard/expiry and clean orphan staging separately. | I 2.5 |
| 20 | Seal implies authenticity | Execution/cycle reports explain persistence-only attestation and attribute declarations. | I 4.2 |
| 21 | Provenance omits processing | Host hashes uploads; export origin, claimed importer/SDK, inventory/hashes, script processing description and declaration. Unattached source hashes stay claims. | I 2.2, 4.2 |
| 22 | Preview stores a report | Pure snapshot rendering with temporary output; publication is a separate operation for runs too. | G 3.15; I 3.4 |
| 23 | New documents need projections | Open: document validation and query preparation strategy above. | I 1.5, 3.4 |
| 24 | Repeated preview cost unbounded | Open: inherited performance-contract reconciliation, budgets/cache/cancellation above. | I 1.6, 3.4 |
| 25 | No schema catalog owner | Freeze script's import-local catalog; no plugin registration or global definition overwrite. | G 3.12; I 2.2 |
| 26 | Equal names imply equivalence | Open: units, calibration, quality and statistics above; frozen catalogs isolate schema ownership. | I 1.7, 3.3 |
| 27 | One call is not atomic transfer | SDK call stages bounded resumable chunks, host verifies checksums and limits, then atomically publishes validated manifest; no live ingestion. | I 2.1, 2.5 |
| 28 | Shared trust allows collisions | Every connected client may operate; handle visibility adds no per-user ACL. Store scope, expected revisions and explicit conflicts prevent accidental replacement. | I 2.3, 4.3 |
| 29 | Columnar parity missing | Reuse plot-frame dokime.v1 message with typed arrays, explicit validity/row count/timestamps and exact int64/uint64/bytes; optional Arrow/dataframes. | I 3.2, 4.3 |
| 30 | Later workflows violate seals | Whole imports only; corrections are new executions linked to predecessors; incremental imports excluded. | I 2.2, 4.3 |
