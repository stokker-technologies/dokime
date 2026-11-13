Status: locked by the owner on 2026-11-12
Boundary, managers and nested resources locked by the owner on 2026-11-12
Naming rule amended by the owner on 2026-11-12: bare verbs on the service's resource
Separation by audience locked by the owner on 2026-11-12
Three visibility levels locked by the owner on 2026-11-12
Paging, SDK packaging, transfers and durable lag amended by the owner on 2026-11-13

# Operation Grammar

## Why

The client protocol has 132 logical operations and 264 gRPC methods, while overlapping
read families and older desktop contracts give the same resources different
names and guarantees. Agree the shared operation grammar before the desktop
moves to the new host, so consumers migrate once.

## What Changes

The maintained architecture contract is docs/architecture.md. Its missing guards
are explicit tasks here; the data-root-identity delta reconciles obsolete whole-root
digest requirements with the existing product-owned compatibility decision.

This documentation records the owner's lock on 2026-11-12. No protocol,
source, test, workflow or specification lifecycle activation is authorized here.
The audience lock separates the plugin protocol (`harmos.plugin.v1`, with product
catalog operations in `dokime.plugin.v1`), host context (`harmos.context.v1` and
`dokime.context.v1`) and client API (`harmos.v1` and `dokime.v1`). Nested resources
remain below the parent package on the same face. Only the host calls Instance;
clients manage entries through Plugins. Context calls are children of an active
invocation, never client services. Faces do not replace catalog visibility:
Private means plugin-internal local execution, absent from the catalog and every
wire; Protected means declaring plugin and host over the host-to-plugin carrier,
with no client-facing gRPC binding; Public permits other plugins by declared
requirement and clients, always through the host. All four SDKs
require explicit visibility with no default.

This product's target plus its mapped library dependencies totals 64 logical
service namespaces, 166 operations, 132 gRPC entry methods and 166 request/response
pairs (332 wrappers). The client mapping now uses the library-owned Access and
Status declarations; design.md records the per-face inventory. Product-only totals
remain 48 namespaces/116 operations/93 gRPC methods/232 wrappers. Counts cover
mapped dependencies, not the library's whole protocol.
Every decision below is Owner and LOCKED on 2026-11-12. These are target counts,
not landed behavior.

| Decision | Status | Direction |
| --- | --- | --- |
| Audience faces | Owner | Each service belongs to exactly one face visible in its package: running-plugin protocol, invocation context or host client API. Rename the plugin protocol service to Instance (Get/Invoke/Samples/Stop); Plugins remains client management. Product plugin lifecycle operations are catalog declarations carried by Instance.Invoke. Context is child calls through the plugin context object, never exposed to clients. |
| Visibility and routing | Owner | Visibility is explicit on every operation, including host built-ins: Private is local-only, never catalogued or wired; Protected admits only the declaring plugin and host over their carrier, with no client-facing binding; Public admits declared cross-plugin requirements and clients through the host. Plugin -> host -> plugin uses a declared requirement, one registry, visibility/fingerprint/admitted-provider checks, inherited scope and propagated result/failure/cancellation. Clients use the generic client invocation path; only the host opens Instance calls. All four SDKs require visibility with no default. |
| Per-face declarations and generation | Owner | A capability needed on two faces has separate IDs, requests and responses per face even with a shared handler, for independent evolution and authorization. One SDK package per language contains plugin development and backend-client functionality, with separate face roots; hosts generate all three faces. Package/file moves precede other implementation work. Amended by the owner on 2026-11-13. |
| Naming grammar | Owner | Bare verbs on the service resource; noun-only single-mode reads. Multiple read/change paths form nested resources with bare verbs, replacing rule 3 and flat compound services. Nested services live in a lowercase parent sub-package with one file per service in a folder per parent; wrappers use the local service and method names. |
| Library/product boundary | Owner | The library owns generic machinery under its managers: Plugins, access, transfers and host status, in addition to operations/invocations, time, storage, streams, plugin protocol and SDKs. A product owns its storage definition, objects, operations and authorization policy; it serves the library services directly by composition. Never rebuild generic machinery on the protocol crate alone: that previously duplicated plugin connection, dispatch, streams and process launch. |
| Settings and secrets | Owner | Product-owned stores, schemas, UI declarations, archives and OS credential-store use. The library owns only generic plugin-to-host calls, including context HostValues.Get with a host-supplied resolver, and a plugin catalog's settings-declaration section. The per-plugin settings/secrets question is closed. |
| Managers | Owner | Live ownership determines managers, as stated once in design.md. Library managers own their services. Split the product Host/shared State into execution, prompt and report managers, plus capture only if recordings remain; rendering is pure and storage operations have no manager. Decide execution-versus-small-cycle ownership at the split by the simplest complete option. |
| Host contributions | Owner | The library service manager owns identity, health, availability, logs, trace events and diagnostics archive; narrow registration accepts host-application version and component-health facts. Product-only reads stay with product services. |
| Nested clients and cross-owner resources | Owner | Descriptor-derived client roots expose the same hierarchy in all four SDKs, built by a small generator from a shared package/service table, without hand-written per-operation code. Product Plugins.Settings intentionally lives beside library Plugins on the same endpoint. |
| Progress | Owner | Executions.Progress.Get and Watch observe the current value, also returned by Executions.Get. A plugin produces it through context Progress.Get/Set; no client-facing progress Set. |
| Lost calls | Owner | Resending after a client-side failure is the client's responsibility; the same request identifier returns the original receipt, and a changed payload under it conflicts. |
| Response representation | Owner | Always a response wrapper, never a bare resource. |
| Ordinary verbs | Owner | Closed set: Get, List, Watch, Set, Create, Delete, Append; Append replaces Create for ordered immutable records, samples, log lines and upload bytes in both repositories. Update, Read, Browse, Put, Edit, Add, Remove, Follow, Scan, Inspect and Register are not ordinary verbs. |
| Set | Owner | Whole writable value and expected revision from the matching Get; Get returns that same value and revision. Stale revisions conflict with the action: get again and retry. Split large writable parts into separately revisioned Get/Set sub-resources. |
| Domain verbs | Owner | Only stated side effects that ordinary verbs would hide; retain lifecycle distinctions and name the clock wait Sleep. |
| Position-resolved reads | Owner | One path per kind; optional `at`, default latest, reply `resolved_at`; same rule for pages and watches. Running executions resolve to the live frontier, sealed executions to their seal. |
| Earlier positions | Owner | Accept only a position returned by a previous reply for consistency, or the seal; no arbitrary historical selection and no separate checkpoint family. |
| Latest and durability | Owner | Resolve latest durable without waiting; use durable receipts for read-your-write. The measured lag is accepted: small payloads about 99 ms, large and mixed up to several hundred milliseconds at p95. Report benchmark measurements without a failing threshold; no accepted-data snapshot. Amended by the owner on 2026-11-13. |
| Execution cancellation | Owner | Cancel only running executions; refuse queued cancellation with an error naming the action. Stop finishes at a safe boundary; Cancel interrupts. |
| Execution rename | Owner | Rename one execution, never every attempt of its case. |
| Transfers | Owner | One unary, ranged, resumable mechanism for clients and plugins: Storage.CreateFileProduction, Storage.AppendFileBytes at ordered offsets, Storage.GetFileProduction, Storage.CreateFileWriter for fenced resume, Storage.PublishFileProduction and Storage.DeleteFileProduction; bounded Storage.GetFileBytes ranges return total length and digest, and Storage.WatchFileBytes follows byte growth. Reuse Storage. Identical chunk replay succeeds; different bytes at an accepted offset conflict. Default chunks are 4 MiB and the default message limit is 8 MiB; remove streaming file transfers. Amended by the owner on 2026-11-13. |
| Streaming twins | Owner | Never generate automatically; server streaming is reserved for Watch semantics (including noun-only streamed parts) and the plugin protocol retains its own carriers. A managed stream replaces the ordinary gRPC method only where a cooperative final result after cancellation is promised. |
| Shared primitives | Owner | One bounded paging shape, `(history, sequence)` positions, protobuf Timestamp/Duration, Failure, retry policy, and accepted/durable receipts. |
| Paging eligibility | Owner | Paging is only for data rows that can grow unbounded in size. Small bounded catalog, plugin, pairing, suite and schema sets return whole in one response; executions, records, samples, logs, files and notes remain paged and position-resolved. Amended by the owner on 2026-11-13. |
| SDK ownership | Owner | Other products will host plugins on the library: it retains language SDKs, and product SDKs build on them. Each language has one package for authoring and backend clients: Python uses dokime-sdk and harmos-sdk, without audience-specific builds. A codebase can install itself via main.py; the API should look simple and good. Protocol, SDKs and conformance remain self-sufficient and free of product vocabulary. Amended by the owner on 2026-11-13. |

**BREAKING:** replace the old operation names, requests and reply carriers; remove
all 132 automatic Run gRPC methods, product copies of generic plugin connection,
dispatch, streams, process launch, entry management, listener/pairing, uploads and
host status, duplicate Runtime identity/health, separate
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

Future implementation first moves declarations into `proto/dokime/plugin/v1/`,
`proto/dokime/context/v1/` and client `proto/dokime/v1/`, then spans protocol generation,
`crates/dokime-operations-next/`, `crates/dokime-runtime-next/`,
`crates/dokime-storage/`, all four SDKs, CLI/client, and desktop/native bridge.
Callers must regenerate, adopt the new IDs/fingerprints and wrappers, retain
`resolved_at` across pages, inspect durability receipts, use upload sessions
and ranged downloads,
and replace old cancellation and case-wide rename assumptions. The desktop's
host cutover and grammar migration are one changeover, followed by deletion of
the old loop and obsolete consumers; no compatibility aliases are proposed.

The sibling library owns the generic services and runtime managers listed in
design.md, including access under its gRPC manager and host status under its
service manager. Its exact revised revision must be published and pinned before
final combined validation. Product service methods stay thin: one operation file
calls a manager or storage. Prepare face package layout and generation filters
first, then library services and client-root generation, then product contracts at their final names and manager split, then consumers;
migrate the desktop once after those services are ready. SDK
parity/client coverage, retention/restart, watch semantics, deletion with readers
and managed cancellation coverage are recorded in design.md. README and product
guides describe current behavior and
need no edits in this documentation-only delivery; implementation must update affected
claims and examples with the consumer cutover.
