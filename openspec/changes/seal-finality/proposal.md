# Seal Finality

Status: Locked by the owner on 2026-11-18, with publication and archive
decisions extended on 2026-11-19. Implementation and proof are tracked in tasks.md.

## Why

An execution that finishes is already stored as sealed, yet the product says
"ended" and "finished", lets notes and files change afterwards, and creates a
cycle's report only when someone asks. Two downloads of one report have
differed because a note arrived in between. "Sealed" has to mean the evidence
is final, for executions and cycles alike.

## What Changes

- Seal is the one terminal verb and state for executions and cycles. "Ended"
  and "finished" leave messages, phases, capabilities and client surfaces.
- A sealed execution refuses new notes and attached files. **BREAKING** for
  clients that annotate after a run.
- Sealing a cycle seals its content, materializes one report covering every
  execution and one evidence archive that contains it, and retains both.
  Later downloads return the retained bytes.
- A cycle can be sealed while some of its executions were never started; they
  are sealed as not run. Only a running execution blocks the seal.
- The cycle passes through its existing Sealing phase while that work runs
  and reports Seal Failed with the cause when it cannot complete.

## Capabilities

### Modified Capabilities

- `evidence-finalization`: seal vocabulary, finality of sealed executions and
  materialization of the cycle's report and archive at seal.

## Impact

Host operations for cycles, executions, notes, file uploads, reports and the
evidence archive; the execution and cycle projections; desktop and
command-line wording; every client surface that reports a terminal state.
Stored execution states remain unchanged. Development storage and evidence
baselines remain v1.

## Retry and Observation Decisions

`Cycles/Seal` is a unary admission call. Its response contains the state at
admission and a receipt whose completion flag describes that state. Report and
archive materialization run as host-owned work beyond the admission deadline.
`Cycles/Watch` is the sole progress mechanism: its cycle summaries carry Sealing,
Sealed and Seal Failed with the cause. Desktop, command-line and Python seal
helpers await this existing watch after admission; an already-sealed admission
needs no watch. Losing an observer does not cancel sealing.

A materialization error that is not a persistence failure leaves the cycle
unsealed in Seal Failed and can be retried in the same host after resolving its
cause. The deterministic report story obstructs the host's temporary HTML
directory through its process environment, then restores it without reopening.
Not-run executions sealed before the error deliberately stay sealed: their
complete, immutable outcomes are valid evidence and retry reuses them. No
partially sealed execution is retained.

A storage persistence failure remains catastrophic under the existing native
storage policy. The fatal write latch is unchanged. Resolve the storage cause
and reopen the host on the same root before retrying. A saved Seal Failed phase
and cause survive reopening; an interrupted Sealing phase recovers as Seal
Failed with the restart cause. The persistence story fails the first native
report insertion, proves the latch still rejects writes after removal of the
fault, reopens the host and seals successfully. Neither failure story leaves a
retained report or archive, and each retry publishes exactly one of each.

## Atomic Publication Decision

The cycle seal is the publication and recovery owner. Its report and archive are
staged unpublished and become visible with Sealed in one existing native storage
compound/scoped mutation. Reopening discards abandoned staging. A late archive
publication failure retains neither output; reopening preserves Seal Failed and
its cause and a retry retains exactly one report and one archive.

## Archive Layout Decision

Cycle archives contain root report.html and manifest.json and ordered folders
executions/<NN> <Template Label>/. Windows/Linux-invalid filename characters in
labels become dashes; other characters remain. Every folder, including not-run
executions, contains its own report.html, manifest.json and readable evidence in
data/ (steps/attempts, records, criteria, prompts/answers, notes, parameters and
outcome). Finalized Arrow streams use dashed identifiers under telemetry/;
retained bytes live in files/ and artifacts/. Standalone archives use the same
content at their root. The encoded evidence file has no product reader and is
replaced by JSON, including frozen execution metadata and declarations.
Manifests hash every content member; their own bytes are excluded from their own
hash inventory to avoid self-reference. Entry times use the owner's seal time.

The shipped Python starter adds its approved 100 ms temperature observer and
names its cycle and saved ZIP from one datetime captured at cycle creation.
