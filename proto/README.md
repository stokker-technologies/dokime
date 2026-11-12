# Dokime Protocol Contracts

`dokime/v1/` contains the authoritative client services, execution, plugin,
context-operation, catalog and document contracts. `harmos/v1/plugin.proto` is a
read-only Harmos copy. Each `client_<service>.proto` owns one client service and
its request/stream envelopes. `client_types.proto` owns shared projections and
replies so services can share them without cyclic imports. All messages are in
`dokime.v1`; canonical values, documents, catalogs, origins, plugin placements,
appearance, backend diagnostics and context messages retain their existing owners.

The declarations cover the 129 operations previously declared in Rust, plus
plugin `List` and `Add`. Each has its existing ordinary route and a duplex
`Run<Method>` route for managed cancellation: 131 operations and 262 RPC routes.
Service paths remain `/dokime.v1.<Service>/<Method>`. The host must adopt these
declarations with its handlers; publishing the files does not migrate a handler.

| Service | Operations | Service | Operations |
| --- | ---: | --- | ---: |
| Plugins | 9 | Catalog | 6 |
| Planning | 3 | Execution | 28 |
| Observation | 4 | Reports | 3 |
| Health | 1 | Backend | 6 |
| Appearance | 1 | Content | 9 |
| Documents | 2 | Evidence | 3 |
| History | 11 | Media | 5 |
| Notes | 3 | Plots | 3 |
| Prompts | 5 | Query | 6 |
| Remote | 4 | Runtime | 2 |
| Secrets | 7 | Settings | 8 |
| Telemetry | 2 | | |

## Calling Services

Every operation is Public for connected clients and plugins, subject to the
host's authorization and advertised availability. Deadlines, retries, admission
limits and stream buffering remain callee policy. An SDK must not automatically
retry a mutation after an uncertain acknowledgement. Read back its identity or
idempotency key before deciding whether to repeat it.

Unary requests and replies are direct messages. Client-streaming requests send
one `request` envelope first, followed by ordered `item` envelopes, then
half-close. Server-streaming replies carry ordered `item` envelopes followed by
one `response`, even when that response is empty. Accept a download only after
its completion frame, size/digest verification where supplied, and successful
final gRPC status. A transport disconnect or missing terminal response is not
success. Duplex ordinary routes combine these rules.

`Run<Method>` always uses duplex streaming, including for unary operations. Send
exactly one request first. Input items use field 2; `finish` at field 4 closes
operation input without cancelling its output. `cancel` at field 3 requests
operation cancellation. Keep reading until settlement and final status; output
items use field 1 and the final response uses field 2. Finish and cancel are
different controls. Transport cancellation ends observation but does not roll
back a committed mutation. Admitted executions outlive the start call and
runtime watch: stop them explicitly through `Execution.CancelExecution`, which
is valid only while a live manager drives the execution.

Paged calls retain filters, ordering and the captured cursor/frontier across
continuations. An absent or empty continuation ends a page sequence. A changed
epoch, revision or manifest may require restarting the sequence. Ordering is
owned by the operation: planning preserves selection and dimension order;
runtime watches are latest-state invalidations that can coalesce, whereas
record/telemetry cursors refer to evidence frontiers. A watch is not a durable
event log unless its operation explicitly promises replay.

The first client journey adds a remote plugin, reads its catalog, materializes
a selection through Planning, creates a cycle, appends explicit execution/case
bindings, starts those executions and watches Observation before downloading
Reports. Mutation replies distinguish admission from completion.

## Model Cutover

These declarations use the locked target model rather than preserving the
retired native row-generated wire format. New messages have protocol-owned
field numbers; existing canonical message numbers remain unchanged. Regenerate
all clients together with the host's protocol-operation adoption.

- Planned cases and live/historical execution reads carry `Origin`, with the
  full frozen `TestCase` in `PluginRun` or the import manifest. No wire `Run` or
  second execution-definition model is introduced. Executions have only
  open/closed/sealed state. Live manager state is a separate optional projection.
- Outcomes retain evaluated, declared or unevaluated attribution. Cycle result
  summaries count declared and unevaluated members separately; their headline
  folds evaluated outcomes only and cannot pass without an evaluated result.
- Plugin entries reuse managed/remote placement and availability state.
  `Plugins.Remove` now accepts the canonical entry identity and removes either
  placement; managed installation and bundle inspection remain separate calls.
  Saved settings have no applied-versus-saved comparison or unapplied state.
- Values and expressions use the canonical typed model. Named setting and
  prompt value maps use `MapValue`; prompt definitions reuse `Prompt`.
  `QueryAggregate.percentile` retains the rank of native percentile aggregates.
- Runtime identity and health reuse the Backend messages, including the
  canonical timestamp instead of separate seconds/nanoseconds fields.
- `Metadata` additively retains tags, named attributes and creation/update times
  from the served identity model. `PromptCatalog` replaces the catalog payload
  name `Prompts`, which cannot coexist with the required service of that name.

## Generation

Generate Rust with `mise run generate -- --protocol`; verify drift with
`mise run test -- --package dokime-protocol --target integration` (or
`mise run generate -- --protocol --check`). The pinned vendored compiler needs
no installed `protoc`. Normal builds consume checked-in generated Rust.
`dokime-protocol` exposes `dokime_protocol::v1` and re-exports Harmos messages
from the pinned `harmos-protocol` owner; it has no dependency on the host.
Only Rust, reflection, descriptors and embedded sources are committed here.
Each SDK runs its language's protobuf and gRPC generators over
the same complete source set, including `harmos/v1/plugin.proto` and Google
well-known imports. Python uses grpcio generators, C# uses Grpc.Tools, and C++
uses its build's matching protobuf compiler and gRPC plugin. No SDK should
derive its client definitions from Rust declarations or committed document-only
bindings.

Export these sources with `dokime proto export --out <directory>`. The output
contains only `.proto` files and preserves their package directories. Standard
`google/protobuf` imports come from your protobuf toolchain. Plugin-specific
export is not available yet; obtain those sources from the plugin author.

Within `v1`, evolve additively: preserve field names, numbers, types and meanings;
reserve removed names and numbers; ignore unknown fields and enum values. Every
enum has an unspecified zero. Incompatible changes require `v2` alongside `v1`
for one release.
