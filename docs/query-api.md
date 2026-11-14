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

`Documents.Get` and `Documents.Watch` accept `max_frames` alongside `max_bytes`.
Zero uses the host's item bound. Pages contain complete frames; their joined
sequence retains the same structure, evidence cut and completion counters. A
document read returns plot declarations without executing those plots. The desktop
then requests its visible selections in independent batches of at most 32 plots.
Within one batch, identical selections share a read and frozen schema preparation;
their plot identities, fingerprints and fixed cuts remain independent.

Frequency plots declare `Plot.input_samples`, projected into
`PlotRequest.fft.input_samples`. The selected input is the latest N finite samples
in source-time/position order within its filters and requested source window.
Omitting N uses the host's bounded transform input. A count below two or above that
bound fails with an action; a multi-day source alone is never a refusal. `fft.window`
still names the taper, and `max_points` still controls display resolution.

Grid previews retain the latest complete frames fitting the rows, columns and
display resolution. Numeric frame expressions use source-time seconds and return
timestamp instants. Source counts include the examined history, so a bounded
preview does not present a truncated source as complete evidence. A grid whose
single frame exceeds the resolution still requires fewer rows or columns.

## Remaining Contract Work

Returned-position retention, durable snapshot/follow semantics, replay receipts,
whole-value revision guards and final nested services are tracked in the combined
operation-grammar tasks. Missing evidence, unavailable cuts and watch gaps must
never become empty success. Performance measurements are separate from correctness
stories, with no wall-clock or resource-sampling assertions.
