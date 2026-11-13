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
source time, host receipt time, schema and durable position. The store retains
Arrow batches alongside exact protobuf evidence and derived indexes. Read pages
apply configured row/byte limits and reject invalid or changed continuations.

Documents evaluate against fixed evidence selections. HTML rendering creates
temporary output; downloads do not publish it. Explicit report publication freezes
the selected document and evidence seal. Unsupported plot/document variants fail
with an action rather than returning incomplete results. See
[Read and Plot Contracts](query-api.md) and [Supported Host Variants](execution-host.md).

## Interfaces

`dokime-next` is the current command-line binary from `crates/dokime-cli-next`.
It hosts Dokime in process, creates projects in all four SDK languages, produces
and inspects bundles, admits plugins and controls execution/cycle workflows.
Exact subcommands are in the [CLI section](execution-host.md#command-line-candidate).
Desktop native integration has a separate restoration checklist in the combined
change; this candidate does not package desktop installers.

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
