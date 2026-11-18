# Seal Finality

Status: locked by the owner on 2026-11-18. Implementation waits behind the
administration scan repair (operation-boundary task 4.4w).

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
Storage formats are unchanged: the execution's terminal state is already
`Sealed`.
