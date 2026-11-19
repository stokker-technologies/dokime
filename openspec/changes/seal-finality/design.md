# Seal Finality Design

## Context

See proposal.md for the agreed publication and archive contracts.

Cycle state currently persists through `crates/dokime-storage/src/cycles.rs`
in `execution.sqlite3`. Native file productions, prepared-report objects and
compound mutations persist in `storage.sqlite3` through
`crates/dokime-storage/src/backend/`. The cycle materializer currently calls
report preparation, archive materialization and cycle-state saving independently.
A late archive persistence failure therefore retains an earlier report.

## Goals / Non-Goals

The cycle seal owns a commit record for its report and archive. The single
Sealed-state write carries both references; cross-store atomicity is not required. This delivery preserves the existing fatal persistence latch and operation
policies. Other open changes remain outside this worker's ownership.

## Decisions

### Seal Attempt Commit Record

Each admitted attempt owns the exact `cycle-seal/<cycle>` storage scope. Before
admitting a replacement, the cycle owner deletes the previous scope through
native DeleteScope. The scope is reused only after that deletion, while each
attempt's report, archive and mutation identities remain unique. On-demand
reports belong to `cycle/<cycle>` and are never included in attempt deletion.

The existing report preparation and archive uploads materialize complete native
files and the prepared report capture inside the attempt scope. Neither reference
is reachable through the cycle before commitment. One `save_cycle` write persists
Sealed together with both file references; that write is the commit point. A
failed attempt leaves the cycle unsealed with Seal Failed and its cause, and a
retry remains admissible after the existing fatal storage latch is repaired by
reopening when necessary.

On host open, the existing native upload owner abandons incomplete productions
in non-Sealed cycles' attempt scopes before native upload recovery runs. The
cycle owner then deletes those scopes through the same native DeleteScope path
used before a new attempt, before other startup recovery reads can retain the
failed outputs. A Sealed cycle retains its scope. Native tombstones
preserve deletion history; scoped deletion reclaims the failed attempt's file
chunks and prepared capture. No sweeper, timer, registry or parallel recovery
mechanism is added.

### Publication Boundaries

Cycle state lives in `execution.sqlite3`; native files and prepared captures live
in `storage.sqlite3`. The commit record needs no cross-database transaction or
staged-file compound capability. Existing upload publication, scoped deletion,
operation policies and the fatal persistence latch remain the owners of their
respective behavior.

### Archives: Implemented

Storage assembly writes ordered execution folders directly into the cycle ZIP.
The operation supplies frozen template labels through the existing execution
projection owner. Each execution gets its own report capture. Standalone export
uses the same member writer at the root.

No product reader consumed `evidence.pb`. Its encoder and archive wire structs
are removed. JSON projects the original evidence, typed record rows and frozen
declarations. Keys are deterministic; integer and floating-point formatting is
locale independent. Logs, progress, traceability, capture gaps, statistics reads
and document parts have named JSON files; there is no remainder bucket. Native
file references remain beside their retained bytes. Cycle metadata and its notes
and artifact declarations remain in root `data/cycle.json`.

Each manifest hashes every member in its scope except itself. The root manifest
also hashes nested manifests. Python evidence downloads verify the whole-file
digest and these complete member inventories. Export verifies original native
file and Arrow digests before writing them.

Execution seal timestamps come from the existing runtime clock and persist with
the execution. Successful cycle sealing
retains the timestamp frozen at admission so the retained state and archive agree.
ZIP entries carry DOS dates and NTFS timestamps; the latter preserve controlled
clock dates outside the DOS range and subsecond precision. No explicit directory entries are written; absent file or
artifact folders mean none. ZIP compression preserves the original member bytes.

### Prepared Reports: Implemented

Durable prepared-report objects own lookup. The runtime temporary registry and
its retention limit/lifecycle are removed; explicit execution publication calls
the existing durable report store directly.

## Risks / Trade-Offs

- Files may outlive a failed attempt until its owner knows the attempt is dead →
  Native scoped deletion runs before the next attempt and on host open.
- The unused encoded archive format is removed → All in-tree archive readers
  and stories consume the JSON replacement; development baselines remain v1.
- A manifest cannot hash its own bytes → Each manifest excludes itself; the
  parent hashes nested manifests and the download reference hashes the whole ZIP.

## Migration Plan

Use the JSON archive layout as a clean development break. Existing sealed archives
remain immutable retained files; existing storage roots are preserved. Do not
silently reset a root. Full validation and retirement remain open until commit-record
recovery and the required project proof are complete.
