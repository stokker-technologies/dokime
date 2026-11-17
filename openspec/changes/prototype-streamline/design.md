# Prototype Streamline Design

## Retained Contract

Application preferences are protocol values in
`proto/dokime/host/v1/settings.proto`, persisted with whole-value revision guards.
Bootstrap paths are read-only. The retired operational-knob catalog and retained
memory-accounting machinery are not restored. Existing incompatible roots are
preserved and refused at baseline v1; there is no silent reset or migration.
Desktop selection remembers Local or a saved remote and preserves unavailable
remote selection with explicit connection controls. Paths follow distribution
identity. OS credentials remain protected and root-qualified.

The current implementation lives in `crates/dokime-operations/src/host/settings/`,
the desktop settings/connection owners and runtime composition. Host-free Rust
SDK dependencies are in `sdk/rust/Cargo.toml`. The old domain/server/client and
managed-checkpoint loop have been deleted; old test paths and lease admission
counters are not outstanding work against those deleted mechanisms.

## Ownership After the Execution-Host Replacement

operation-boundary owns remaining storage consumers, native cuts, compound
mutations, Arrow intake, bounded query transforms, archive fidelity and facade/
wrapper adoption. plugin-boundary owns plugin contracts, SDKs, registry/settings
and bundles; plugin-lifecycle owns verbs/hooks. Generated output stays tracked.
Generic aggregation and merge kernels belong in Harmos; product presentation and
FFT behavior stay product-owned. New adapter or extraction layers are not implied.

remote-access-simplification owns pairing and the unresolved transparent-recovery
contract. grpc-plugins owns remaining file-source support, Effects, aggregate/media
handlers, benchmark restoration and application migration. test-stories owns the
surviving test inventory and deterministic consolidation. Duplicate prototype
worklists for those owners are removed rather than checked as implemented.

## Remaining Acceptance

Recheck preference edits/reopening/conflicts, read-only bootstrap paths, decoder
retention and exact wire bounds against final consumers on both native platforms.
Current/sealed report-save and ingest-contention measurements use the restored
benchmark path with identical deterministic fixtures. Preserve real historical
baselines if available; do not invent missing comparisons. Conditional scheduling
or storage tuning requires evidence and unchanged durability/cancellation.

Retained multi-day file/log capture must bound pending work, retain saved offsets
on failure and expose permanent capture failures. Reconcile its segmented output
and fault policy with the current execution-file owner, without restoring retired
plugin-restart recovery. Measure surviving crates before proposing further
layering. Source presence and historical Windows proof do not establish final
native Linux acceptance or authorized delivery.
