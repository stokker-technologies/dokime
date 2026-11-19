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

The cycle seal owns one publication boundary for its staged report, staged
archive and Sealed state, with abandoned staging recovered at the native file
owner. This delivery preserves the existing fatal persistence latch and operation
policies. Other open changes remain outside this worker's ownership.

## Decisions

### Atomic Publication: Agreed, Pending Its Storage Capability

The owner's decision requires the existing native compound/scoped mutation.
The report and archive stay unpublished until that mutation publishes their
references and the cycle's Sealed state. Reopening discards abandoned staging.
The late-failure story must retain neither output and retry must publish one of
each. Task 3.7 owns this outstanding implementation.

### Upstream Capability Request

- Consumer evidence: `cycle_seal_retires_report_after_archive_persistence_failure`
  in `crates/dokime-runtime/tests/system/cycle_seal_retains_evidence.rs` reopens
  with one retained report after the archive insertion fault.
- Attempted supported API: native `MutateRequest` dispatches to
  `crates/dokime-storage/src/backend/compound.rs`. Its pinned `CompoundWrite`
  supports Object, Delete, Record and Scope; no item publishes a staged file.
  `PublishFileProduction` is an independent native publication.
- Smallest required capability: publish staged native file references through
  the existing compound owner together with the owning cycle-state object,
  and discard unpublished productions through its existing recovery owner.
  Cycle state must participate in that same authoritative native history.
- Disposition: the atomicity contract is agreed; the required library facility
  and native cycle-state mapping are unavailable in this worktree. The upstream
  owner must supply a compatible capability and its validation handoff before
  this worker can close task 3.7. No local override or upstream edits are made.
  The existing independent-publication path and failing regression remain visible.

A product-side publication mirror, cross-database commit assumption or independent
cleanup registry would introduce the competing ownership the decision removes.

### Archives: Implemented

Storage assembly writes ordered execution folders directly into the cycle ZIP.
The operation supplies frozen template labels through the existing execution
projection owner. Each execution gets its own report capture. Standalone export
uses the same member writer at the root.

No product reader consumed `evidence.pb`. Its encoder and archive wire structs
are removed. JSON projects the original evidence, typed record rows and frozen
declarations. Keys are deterministic; integer and floating-point formatting is
locale independent. Native file references remain beside their retained bytes.

Each manifest hashes every member in its scope except itself. The root manifest
also hashes nested manifests. Python evidence downloads verify the whole-file
digest and these complete member inventories. Export verifies original native
file and Arrow digests before writing them.

Execution seal timestamps come from the existing runtime clock and persist with
the execution. Successful cycle sealing
retains the timestamp frozen at admission so the retained state and archive agree.
ZIP entries carry DOS dates and NTFS timestamps; the latter preserve controlled
clock dates outside the DOS range and subsecond precision. Directories carry
explicit timestamps too. ZIP compression preserves the original member bytes.

### Prepared Reports: Implemented

Durable prepared-report objects own lookup. The runtime temporary registry and
its retention limit/lifecycle are removed; explicit execution publication calls
the existing durable report store directly.

## Risks / Trade-Offs

- Independent cycle publication currently retains a report after a late archive
  failure → Close task 3.7 with one storage boundary and pass all three failure stories.
- The unused encoded archive format is removed → All in-tree archive readers
  and stories consume the JSON replacement; development baselines remain v1.
- A manifest cannot hash its own bytes → Each manifest excludes itself; the
  parent hashes nested manifests and the download reference hashes the whole ZIP.

## Migration Plan

Use the JSON archive layout as a clean development break. Existing sealed archives
remain immutable retained files; existing storage roots are preserved. Do not
silently reset a root. Full validation and retirement remain open until atomic
publication and the required project proof are complete.
