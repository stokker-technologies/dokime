# Long-Execution Finalization

## Agreed Direction

Evidence remains durable as received. Memory must not grow with total execution
length. The target is for end-of-run work to seal only a bounded pending tail and
report work to scale with bounded rendered output. No timeout increases or silent
data deletion.

## Owners And Original Findings

- `domains/execution/run/lifecycle.rs` owns provider cleanup. Originally a
  cancelled/failed attempt could repeat stages; retained ownership now joins them.
- `runtime/storage/captures.rs` originally removed capture requests before awaiting
  completion. The retained barrier now keeps concurrent closers waiting.
- Runtime shutdown awaits owned Stop work and provider monitors before stopping
  plugins; the counted integration story verifies this existing ordering.
- `domains/telemetry/output.rs` originally scanned and decoded each stream for
  every numeric field at completion, calculating ten aggregates independently.
- `report/materialize.rs` formerly scanned raw history and replayed spilled plot
  inputs. Durable envelopes and indexed first/last reads replace that pipeline.
- Telemetry signal and record capture foreign keys reference evidence positions,
  but their original primary keys start with other columns. Metadata retirement
  therefore scans accumulated evidence children. Dedicated position indexes make
  each check and cascade visit only matching children.

## Bounded Report Inputs

Owner decisions locked on 2026-10-30 replace full-history report presentation.
`storage/sqlite/data/telemetry/previews.rs` maintains a configured maximum number
of adaptive time buckets per execution/stream/numeric field. Doubling bucket
intervals merges first/last/min/max/count without losing extrema. Updates share
the raw sample and exact-aggregate transaction; exact retries update none twice.
Report plots fit those envelopes to their configured display-point budget. Raw
stream and record tables seek indexed first/last rows with configured limits.
Reports identify the execution and evidence frontier for retrieving complete data.

Unfiltered numeric summaries and built-in mean analysis read exact moments/ranks,
including arbitrary requested percentiles. History-dependent transformations,
filters, grouping, correlations, repeated sections and non-time plots without an
incremental index are explicitly unavailable, with the retained evidence identified.
Old evidence cuts whose mutable preview has advanced are refused; no raw replay
is a fallback. Presentation settings are construction snapshots owned by SQLite
and report defaults. Fresh v1 roots are required; no migration or reset occurs.

## Background Publication

Provider completion and Stop/Cancel close and drain capture, freeze the terminal
execution revision and durably record its report intent. A retained background job
publishes and seals HTML afterwards. The existing durable intent owns pending or
failed publication; a durable execution seal means ready. Execution detail exposes
that state and failure cause through the shared generated contract. GUI displays
it; CLI and Python snapshots expose it. Downloads and evidence saves join readiness.
Publication failure cannot change the provider outcome or fail a repeated Stop.
Shutdown cancels presentation jobs after evidence settlement and preserves their
intent; startup resumes publication without rerunning providers.

## Incremental Numeric Index

`storage/sqlite/data/telemetry/aggregates.rs` updates rolling moments and exact
numeric ranks in the raw evidence transaction. Eight byte-prefix levels over
sortable IEEE-754 keys keep rank lookup depth fixed; each level visits at most
256 children. Batched prefix deltas retain only one admitted
write batch, never a whole execution. Exact position retries skip both raw writes
and index updates. Completion reads moments and rank lookups without raw replay.
This replaces `domains/telemetry/output.rs`'s repeated stream decoding and ten
separate accumulators per field. Storage stays v1; older indexed roots are refused
without migrations or resets. Summary arithmetic matches Harmos's running moments.
The existing signal metadata also carries each finite numeric projection's exact
IEEE-754 bits. Admission derives it in its existing single payload pass and shares
it across lease scopes; the SQLite index never prepares schemas with a second,
possibly different configuration. Decoding checks the projection against raw bytes.

## Capture And Cleanup Ownership

Capture closure fences admission, starts one retained drain and joins its barrier.
Provider cleanup retains its stage and runs under a detached owner holding the
original mutex; a cancelled waiter cannot restart successful stages. Runtime
shutdown keeps its existing owned-work/provider joins. Counted integration stories
exercise capture seals and effect restoration across Stop, retry and shutdown.
Log resident cleanup opens its source if Stop arrives before the first run poll;
the same acknowledged byte frontier makes subsequent cleanup idempotent.

## Validation Boundary

Deterministic default-cadence host streams plus OBC-like data, records, logs,
traceability and documents at one hour and four times that volume. Compare actual
Stop-to-seal, Cancel-to-settled and report/save measurements separately. Physical
drain counts prove concurrency, re-entry and cancelled-waiter behavior. Full check
and format follow source closure, with focused tests throughout. Commit only.
