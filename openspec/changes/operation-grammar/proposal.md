Status: proposed, not locked

# Operation Grammar

## Why

The client protocol has 132 logical operations and 264 routes, while overlapping
read families and older desktop contracts give the same resources different
names and guarantees. Agree the shared operation grammar before the desktop
moves to the new host, so consumers migrate once.

## What Changes

This is a documentation-only proposal awaiting the owner's lock. No protocol,
source, test, workflow or specification lifecycle activation is authorized here.
“Owner” below means decided by the owner on 2026-11-12; “proposed” means a
recommendation awaiting lock, including recommendations the owner accepted in
discussion. Proposed requirement deltas describe the target, not landed behavior.

| Decision | Status | Direction |
| --- | --- | --- |
| Naming grammar | Owner | Symmetric, predictable `VerbResource`; `<Method>Request` and `<Method>Response`. |
| Response representation | Proposed | Always a response wrapper, never a bare resource. |
| Ordinary verbs | Proposed | Closed set: Get, List, Watch, Set, Create, Delete, Append; Append replaces Create for ordered immutable records, samples and log lines in both repositories. Update, Read, Browse, Put, Edit, Add, Remove, Follow, Scan, Inspect and Register are not ordinary verbs. |
| Set | Proposed | Optional field mask; absent means the whole writable value; expected revision guards resources with revisions. |
| Domain verbs | Proposed | Only stated side effects that ordinary verbs would hide; retain lifecycle distinctions and name the clock wait Sleep. |
| Position-resolved reads | Owner | One path per kind; optional `at`, default latest, reply `resolved_at`; same rule for pages and watches. Running executions resolve to the live frontier, sealed executions to their seal. |
| Earlier positions | Owner | Accept only a position returned by a previous reply for consistency, or the seal; no arbitrary historical selection and no separate checkpoint family. |
| Latest and durability | Proposed | Resolve latest durable without waiting for accepted unsaved data; no accepted-data snapshot. Use a write receipt's durable position for read-your-write. Measure lag before lock. |
| Execution cancellation | Owner | Cancel only running executions; refuse queued cancellation with an error naming the action. Stop finishes at a safe boundary; Cancel interrupts. |
| Execution rename | Owner | Rename one execution, never every attempt of its case. |
| Uploads | Proposed; recommendation accepted by owner | One resumable unary session mechanism for clients and plugins, replacing streaming uploads. |
| Streaming twins | Proposed | Never generate automatically; use unary operations and resource Get/Watch. A managed stream replaces the ordinary route only where a cooperative final result after cancellation is promised. |
| Shared primitives | Proposed | One bounded paging shape, `(history, sequence)` positions, protobuf Timestamp/Duration, Failure, retry policy, and accepted/durable receipts. |
| SDK ownership | Open | Whether the library retains language SDKs or the product SDKs implement the plugin protocol directly; no choice made. |

**BREAKING:** replace the old operation names, requests and reply carriers; remove
all 132 automatic Run routes, duplicate Runtime identity/health, separate
History/checkpoint paths, streaming upload inputs and superseded generated DTOs.
Preserve the capabilities behind these routes: frozen context, seals, bounded
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

These deltas supersede conflicting requirements only after owner lock and
implementation. Other open changes, especially `grpc-plugins`,
`storage-interface` and `imported-executions`, remain unimplemented here; their
owners must reconcile affected contracts rather than introduce parallel rules.

## Impact

Future implementation spans `proto/dokime/v1/`, protocol generation,
`crates/dokime-operations-next/`, `crates/dokime-runtime-next/`,
`crates/dokime-storage/`, all four SDKs, CLI/client, and desktop/native bridge.
Callers must regenerate, adopt the new IDs/fingerprints and wrappers, retain
`resolved_at` across pages, inspect durability receipts, use upload sessions,
and replace old cancellation and case-wide rename assumptions. The desktop's
host cutover and grammar migration are one changeover, followed by deletion of
the old loop and obsolete consumers; no compatibility aliases are proposed.

The sibling library owns generic protocol/storage primitives. Its exact revised
revision must be published and pinned before final combined validation. SDK
placement, retention, managed cancellation coverage and other lock questions are
recorded in design.md. README and product guides describe current behavior and
need no edits in this proposal-only delivery; implementation must update affected
claims and examples with the consumer cutover.
