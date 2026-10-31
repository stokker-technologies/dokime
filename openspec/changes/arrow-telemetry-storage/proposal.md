# Arrow IPC Telemetry Storage

## Why

Telemetry currently writes an opaque sample plus per-sample and per-signal SQLite indexes. Queries decode payloads and evidence export rebuilds partitioned stream tables; Arrow IPC should reduce write amplification and eliminate export conversion.

## What Changes

- Experimental owner-locked design dated 2026-10-30; implementation and merge require the recorded contract and the owner's authority respectively.
- Store telemetry as ZSTD-compressed Arrow IPC files in Dokime's backend, with configurable batch flushing, durable acknowledgements, recovery and indexed live reads.
- **BREAKING:** Replace telemetry payload/index tables with SQLite file/stream catalogs and retained incremental numeric summaries; distinguish artifacts from other files.
- **BREAKING:** Export original Arrow files alongside relational evidence and manifests; remove stream partitions, column splitting and `stream_columns`.
- Keep storage and evidence baselines at v1; require fresh incompatible roots without resetting existing data.
- Commit locally in small Conventional Commits; do not push, publish, create installers or run umbrella lanes. Do not edit Harmos or hilti-hcl.

## Capabilities

### New Capabilities

- `telemetry-storage`: Durable typed telemetry, indexed live reads, recovery and portable evidence archives.

### Modified Capabilities

None. Existing `host-telemetry` requirements describe producers and remain unchanged.

## Impact

Runtime storage, configuration, saved cursors, telemetry readers, file ownership, execution/session shutdown and evidence exports; runtime tests/benchmarks, Python export tests and maintained documentation. GUI source changes are conditional on actual consumer needs.

## Approved Footer Amendment

The owner's 2026-10-30 amendment defines decision 3's O(1) as no telemetry payload scan, decode or rewrite at finalize. Metadata-only footer emission proportional to batch count is approved, using the in-memory/catalog block list. Recovery scans batch headers and skips payload bodies. All remaining locked decisions stand.
