Status: locked by the owner on 2026-11-12

# Operation Grammar

## Why

The client protocol has 132 logical operations and 264 gRPC methods, while overlapping
read families and older desktop contracts give the same resources different
names and guarantees. Agree the shared operation grammar before the desktop
moves to the new host, so consumers migrate once.

## What Changes

This documentation records the owner's lock on 2026-11-12. No protocol,
source, test, workflow or specification lifecycle activation is authorized here.
Every decision below is Owner. Requirement deltas describe the locked target,
not landed behavior.

| Decision | Status | Direction |
| --- | --- | --- |
| Naming grammar | Owner | Symmetric, predictable `VerbResource`; `<Method>Request` and `<Method>Response`. |
| Response representation | Owner | Always a response wrapper, never a bare resource. |
| Ordinary verbs | Owner | Closed set: Get, List, Watch, Set, Create, Delete, Append; Append replaces Create for ordered immutable records, samples, log lines and upload bytes in both repositories. Update, Read, Browse, Put, Edit, Add, Remove, Follow, Scan, Inspect and Register are not ordinary verbs. |
| Set | Owner | Whole writable value and expected revision from the matching Get; Get returns that same value and revision. Stale revisions conflict with the action: get again and retry. Split large writable parts into separately revisioned Get/Set sub-resources. |
| Domain verbs | Owner | Only stated side effects that ordinary verbs would hide; retain lifecycle distinctions and name the clock wait Sleep. |
| Position-resolved reads | Owner | One path per kind; optional `at`, default latest, reply `resolved_at`; same rule for pages and watches. Running executions resolve to the live frontier, sealed executions to their seal. |
| Earlier positions | Owner | Accept only a position returned by a previous reply for consistency, or the seal; no arbitrary historical selection and no separate checkpoint family. |
| Latest and durability | Owner | Resolve latest durable without waiting; durable lag must meet a payload-size-independent 250 ms p95 bound. Use durable receipts for read-your-write. Benchmark small, multi-megabyte and mixed loads; first reduce flush pieces and flush at a size threshold; accepted-data snapshots are the named fallback if that is insufficient and are not built now. |
| Execution cancellation | Owner | Cancel only running executions; refuse queued cancellation with an error naming the action. Stop finishes at a safe boundary; Cancel interrupts. |
| Execution rename | Owner | Rename one execution, never every attempt of its case. |
| Transfers | Owner | One unary, ranged, resumable mechanism for clients and plugins: CreateUpload, AppendUploadBytes at ordered offsets, GetUpload, PublishUpload and DeleteUpload; bounded GetContent ranges return total length and digest. Identical chunk replay succeeds; different bytes at an accepted offset conflict. Configuration bounds chunks and ranges. Benchmark large-file throughput in Nightly before deleting streaming transfers. |
| Streaming twins | Owner | Never generate automatically; server streaming is reserved for Watch and the plugin protocol retains its own carriers. A managed stream replaces the ordinary gRPC method only where a cooperative final result after cancellation is promised. |
| Shared primitives | Owner | One bounded paging shape, `(history, sequence)` positions, protobuf Timestamp/Duration, Failure, retry policy, and accepted/durable receipts. |
| SDK ownership | Owner | Other products will host plugins on the library: it retains language SDKs, and product SDKs build on them. Protocol, SDKs and conformance remain self-sufficient and free of product vocabulary. Only four-language plugin parity and generic managed-client coverage remain open under SDKs. |

**BREAKING:** replace the old operation names, requests and reply carriers; remove
all 132 automatic Run gRPC methods, duplicate Runtime identity/health, separate
History/checkpoint paths, streaming transfers and superseded generated DTOs.
Preserve the capabilities behind these gRPC methods: frozen context, seals, bounded
evidence, preview versus publication, interactive commands, scoped access and
replay protection. “No named caller found” is not deletion evidence.

## Capabilities

### New Capabilities

- `operation-contracts`: Naming, Set/Append, watches, receipts, cancellation,
  execution rename and shared upload contracts across product interfaces.

### Modified Capabilities

- `query-snapshots`: Resolve latest durable without a read barrier; accept only
  returned positions or seals, with exact retained-cut continuation.
- `query-operations`: Replace the fixed five-operation read family and
  read-triggered flush with the canonical resource read paths.

These deltas supersede conflicting requirements only after implementation.
Other open changes, especially `grpc-plugins`,
`storage-interface` and `imported-executions`, remain unimplemented here; their
owners must reconcile affected contracts rather than introduce parallel rules.

## Impact

Future implementation spans `proto/dokime/v1/`, protocol generation,
`crates/dokime-operations-next/`, `crates/dokime-runtime-next/`,
`crates/dokime-storage/`, all four SDKs, CLI/client, and desktop/native bridge.
Callers must regenerate, adopt the new IDs/fingerprints and wrappers, retain
`resolved_at` across pages, inspect durability receipts, use upload sessions
and ranged downloads,
and replace old cancellation and case-wide rename assumptions. The desktop's
host cutover and grammar migration are one changeover, followed by deletion of
the old loop and obsolete consumers; no compatibility aliases are proposed.

The sibling library owns generic protocol/storage primitives. Its exact revised
revision must be published and pinned before final combined validation. SDK
parity/client coverage, retention/restart, watch semantics, deletion with readers
and managed cancellation coverage are recorded in design.md. README and product
guides describe current behavior and
need no edits in this documentation-only delivery; implementation must update affected
claims and examples with the consumer cutover.
