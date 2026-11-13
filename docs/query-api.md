# Read and Plot Contracts

The current declarations live in `proto/dokime/v1/`, operations in
`crates/dokime-operations-next/src/client/`, serving in
`crates/dokime-runtime-next/src/services/` and persistence in `crates/dokime-storage/`.
The [Execution Host Guide](execution-host.md) lists served methods and explicit refusals.
The [Operation Grammar](../openspec/changes/operation-grammar/design.md) owns the
locked replacement names and position/revision contracts; those remain distinct
from currently implemented behavior.

## Bounded Reads

Small bounded declaration and plan sets return whole values. Accumulated evidence
and execution/cycle history retain row/byte-bounded pages. Continuations bind their
selection and retained evidence identity; mismatches fail explicitly. Samples retain
source time separately from durable ingestion position. Frozen schemas decode
saved payloads after plugin removal and restart.

## Plots and Documents

Plot queries use storage-owned indexed reads and bounded reducers. Raw windows
refuse excess samples; LTTB and bucket reductions retain required endpoints and
reported resolution. Unsupported expression/reducer combinations return actionable
failures. Watches currently deliver snapshots; incremental deltas remain tracked work.
Documents bind plots and summaries to a fixed execution frontier, with bounded
repeat pages and stable card identities. Report rendering produces temporary HTML;
publication is explicit and binds exact frozen document/evidence bytes.

## Remaining Contract Work

Returned-position retention, durable snapshot/follow semantics, replay receipts,
whole-value revision guards and final nested services are tracked in the combined
operation-grammar tasks. Missing evidence, unavailable cuts and watch gaps must
never become empty success. Performance measurements are separate from correctness
stories, with no wall-clock or resource-sampling assertions.
