# Product Guide

Dokime's host executes plugin-defined test cases and retains their evidence.
The [Execution Host Guide](execution-host.md) describes the current executable,
client operations, configuration and supported variants. [Architecture](architecture.md)
distinguishes implemented contracts from locked work still being completed.

## Execution Model

A plugin catalog declares suites and templates. Materialization resolves input
parameters into a case without touching instruments. The host creates execution
evidence, initializes execution-local resources, runs steps and criteria, performs
required cleanup, closes streams and publishes reports explicitly.

Executions are open, closed or sealed evidence containers. Live run state belongs
to the execution manager. Frozen origins and schemas keep sealed evidence readable
when a plugin is removed or the host restarts. Cycles group executions; only
evaluated outcomes determine their headline verdict. Declared or unevaluated
results cannot become an evaluated pass.

Stop completes at a scheduling boundary. Cancel interrupts active work and retains
required cleanup; cancelling queued work is refused. Losing a client observer does
not cancel admitted work. The host refuses an occupied storage root before opening
SQLite, and refuses incompatible development roots with backup/fresh-root guidance.
It never silently resets data. Database and evidence baselines remain v1.

## Plugins

Managed entries install verified executable `.dokime` bundles and own their child
processes. Remote entries connect to a user-owned URL and do not stop that process
when removed. Both placements use the same materialization and execution contracts.
The [Plugin Guide](plugins.md) links the supported SDKs and packaging commands.

Settings are saved product values read on demand; secrets use protected operating
system credential slots. Secret export/import retains the bounded Base64 JSON
format with explicit conflict choices; it is not encryption. Linux requires an
available credential store for secret writes and TLS private-key persistence.

## Evidence and Reports

Records are acknowledged scoped host operations. Samples retain their declared
source time, host receipt time, schema and durable position. Samples have one durable representation: typed, ZSTD-compressed Arrow IPC
files. SQLite catalogs their schema, batch offsets, digests, time/position ranges,
incremental summaries and previews; records and other product objects remain in
SQLite. Read pages
apply configured row/byte limits and reject invalid or changed continuations.

Documents evaluate against fixed evidence selections. HTML rendering creates
temporary output; downloads do not publish it. Explicit report publication freezes
the selected document and evidence seal. Unsupported plot/document variants fail
with an action rather than returning incomplete results. See
[Read and Plot Contracts](query-api.md) and [Supported Host Variants](execution-host.md).

## Storage Roots and Archives

The evidence portion of a root has this layout:

```text
backend-id
local.lock
execution.sqlite3
execution.sqlite3-wal
execution.sqlite3-shm
telemetry/<owner-stream-schema-sha256>.arrow
```

Each file belongs to one execution, stream and frozen protobuf schema. Contract
fields become typed Arrow columns under `values`; `_storage` retains position,
source/receipt nanoseconds and stream sequence. Finalization appends only the Arrow
footer. Restart truncates an interrupted append to the saved checkpoint before
continuing; it does not rewrite saved batches. The diagnostics copy includes these
files and the SQLite files.

Rows are accepted into a bounded live tail. The default flush policy saves at
65,536 rows or five seconds, under pressure at 64 MiB, and at evidence barriers and
shutdown. Saved positions advance only after file synchronization and the catalog
commit. Following reads merge the accepted tail with saved batches; finite reads
retain their selected saved position. Queries default to a 64 MiB working bound.

The CLI configuration file accepts these storage settings:

```toml
telemetry_rows = 65536
telemetry_interval_ms = 5000
telemetry_pending_bytes = 67108864
telemetry_query_bytes = 67108864
```

Embedding hosts receive the same policy through `HostSettings.storage`, including
adaptive preview bucket settings. Changing settings cannot reconstruct evidence
that a previous preview discarded; raw files remain available for exact queries.

An execution ZIP contains `evidence.pb`, the original finalized stream files as
`telemetry/<stream>.arrow`, and `manifest.json` with each stream member's path,
size and SHA-256. Stream members are copied byte for byte. Evidence seals and
content hashes are checked during export. A cycle ZIP contains the execution ZIPs
under `executions/<execution>.zip` and its own size/digest manifest. Deleting sealed
evidence commits file retirement in SQLite before removing its stream files;
startup completes interrupted retirement.

Roots from the rebuilt sample-table layout are incompatible. Select a fresh
storage root; the old folder is preserved. There is no reset or migration.

## Interfaces

`dokime-next` is the current command-line binary from `crates/dokime-cli-next`.
It hosts Dokime in process, creates projects in all four SDK languages, produces
and inspects bundles, admits plugins and controls execution/cycle workflows.
Exact subcommands are in the [CLI section](execution-host.md#command-line-candidate).
The CLI uses the Rust SDK's client roots. `catalog list/show` reads the top-level
Suites resource; `operation list/show/call` discovers contracts through the library's
CatalogEntries.List. Product identities use `dokime.client.v1`; library operations
retain `harmos.v1`. Product operations are currently absent from host discovery,
so raw product calls remain blocked pending host registration. Typed product
commands and library raw calls use these forms:

```sh
dokime-next --endpoint http://127.0.0.1:7000 operation call harmos.v1.Status/Get --json '{}'
dokime-next --endpoint http://127.0.0.1:7000 execution list --cycle CYCLE --json
dokime-next --endpoint http://127.0.0.1:7000 execution show EXECUTION --schemas --json
```

Structured output follows the final response wrappers: suites are in `values`,
cycle creation returns `cycle`, and execution reads return `execution`. Execution
list prints one response per page and continues with that page's retained cut;
`--page-size` bounds each response.
`execution show` accepts several execution IDs and `--schemas` fetches frozen
descriptors through Executions.Schemas.Get once per digest within the command.
Frozen record/stream metadata and every descriptor page carry each identity's fully qualified `message_name` binding for payload decoding.

`plan preview/register --input-file` accepts canonical CasesCreateRequest JSON:
`{"plan":{"selections":[{"suite_id":"SUITE","template_id":"TEMPLATE"}]},"request_id":"selection-1"}`.
Preview forces `validate_only`; registration preserves the caller's request identity.
`cycle append --request` reads encoded `dokime.client.v1.ExecutionsCreateRequest`
with `parent`, `executions` and `request_id`. Lifecycle commands and prompt answers
accept `--request-id`; retain it to retry an unchanged mutation. Omitted identities
are generated for a single command invocation. Whole-value metadata changes use
the final Set request with Get's complete writable value, `expected_revision`
and a retained `request_id`; raw delivery awaits the discovery registration above.

Report commands prepare with Reports.Create, read bounded product Files.Read
ranges at the returned cut, verify length and SHA-256, and save without replacing an
existing file. Rendering and downloading do not publish a report. Product content
byte delivery verifies the file length, SHA-256 and retained read cut;
the product serves those resources over its current content store.

The desktop embeds the same host and also attaches to a pinned remote host.
Product packaging includes the portable desktop distribution and native Windows
NSIS or Linux DEB/RPM installers. The remaining document-capture limitation and
unavailable host controls are listed in [GUI Guidance](gui.md#host-connection).

The host composes the library Access and Status services directly. Access manages
the authenticated TLS listener, pairings, revocation and certificate rotation;
Status serves identity, health, logs, trace events and bounded diagnostics.
Other operation grammar and library composition slices remain locked work.

## Examples

- [Rust Thermal Suite](../sdk/rust/examples/thermal.rs) and [Rust SDK Guide](rust-sdk.md).
- [Python Starter](../sdk/python/examples/starter/) and [Python SDK](../sdk/python/README.md).
- [C# Starter](../sdk/csharp/starter/) and [C# SDK Guide](sdk-csharp.md).
- [C++ Starter](../sdk/cpp/starter/) and [C++ SDK Guide](sdk-cpp.md).
- [Remote Access](remote/overview.md), [Tooling](tooling.md) and [Plugin Security](plugin-security.md).

All SDKs provide plugin authoring and backend clients. SDK product dependencies
stop at protocol; embedding the host uses `dokime-runtime-next`.
