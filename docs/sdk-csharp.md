# C# SDK

`Dokime` is the development NuGet package (`0.1.0-dev`, .NET 10). Public model
values live in `Dokime`, host replies and client-only values in `Dokime.Clients`,
and plugin authoring in `Dokime.Plugins`. Nested resource values retain their
resource namespace under `Dokime.Clients`. Wire requests and service stubs are
internal. Calls take content arguments, own their transport context, and return
the payload directly when the reply has exactly one field.

## Build and Package

```sh
mise -C sdk/csharp install dotnet
mise -C sdk/csharp run clean
mise -C sdk/csharp run restore
mise -C sdk/csharp run generate
mise -C sdk/csharp run build
mise -C sdk/csharp run format
mise -C sdk/csharp run check
mise -C sdk/csharp run test
mise -C sdk/csharp run package
mise -C sdk/csharp run build:examples
mise -C sdk/csharp run test:starter
```

`restore` stages unmodified Harmos sources from revision
`82e394dac656bf6b28a31f7429360d84f1701051`, including their checked-in wire output,
and packs `Harmos` into `target/cs/feed`. Set `HARMOS_SOURCE` to a local library
checkout to read that exact revision with `git archive`; otherwise preparation
uses its immutable source archive. Dokime references the staged `Harmos` project,
so the solution builds the library as its own assembly; the packed `Dokime`
declares the matching `Harmos` package, which exported consumers restore from the feed.
It does not compile library sources into the product assembly.
The conformance project links the same revision's test fixture sources and model
definitions from that staging directory; they remain outside the NuGet assembly.

`generate` regenerates C# into temporary storage and compares the result with the
frozen checked-in `Generated/Client.g.cs`, `Generated/Host.g.cs` and `Generated/Wire`
inventory. It never rewrites that inventory. `build` requires this comparison.
`check` verifies formatting, generator drift, tooling and declared service coverage.
The SDK does not compile the checked-in wire inventory again with `Grpc.Tools`.

Compiler outputs and restored package caches live under `target/cs`. Dependency
locks are tracked beside their owning projects, including the staged library's
lock under tools. `restore --refresh-lock` refreshes them explicitly; ordinary
restore and feed preparation use locked mode. Packages built locally from source
are never content-locked: NuGet's pack output differs between Linux and Windows,
so the pinned revision, not a package hash, fixes the library's content. Locks
therefore record `Harmos` as a project, and exported consumers re-evaluate the
locally built `Dokime` and `Harmos` packages. `package` consumes the existing Release build.
`build:examples` restores the three standalone consumers against that feed.
`build:starter -- --project /path/to/Test.csproj` selects an exported consumer.
Each example pins the NuGet package directly and can be exported independently.

`test:generation` compiles both generated public surfaces and their content calls
in isolation under `target/cs/generation`. `build` compiles the complete solution,
including authored file transfers. `test`, `package`, `build:examples` and
`test:starter` exercise the SDK and its installed NuGet consumers. Real-host proof
also requires the operations/runtime crates and shared host fixtures.

## Author a Test

```csharp
using Dokime;
using Dokime.Plugins;
using Step = Dokime.Plugins.Step;
using Suite = Dokime.Plugins.Suite;

var template = new Template("example.measurement", "Measurement")
    .Step(new Step("measure", "Measure", async context =>
        await context.LogAsync(Severity.Info, "Measurement Completed")));
var suite = new Suite("example", "Example", template);
var plugin = new Plugin("example", "1.0.0", suite)
    .Requires(HostOperation.EvidenceLogsAppend)
    .Requires(HostOperation.TimeGet);
await plugin.ServeAsync();
```

The runnable suite and template are in `sdk/csharp/examples/Test`. The generic
Public operation example is in `examples/Plugin`, and the application client is
in `examples/Client`. These examples are also the command-line tool's starter
sources. Each uses public namespaces.

`Template` composes ordered steps, criteria, streams, window-sampled observers,
parameters, documents and execution-owned resources. Materialization is pure.
Initialization, cleanup, step and criterion bindings are independently Protected;
clients do not receive them in their resource tree. Cleanup releases owned
resources even if the authored cleanup fails. Shutdown drains remaining resources.

A step's `OperationPolicy` owns retry counts and intervals; set `ReplaySafe` when
retries are allowed. Lifecycle callbacks accept independent policy declarations.
Observer intervals come from their declaration or named setting. The host grants
each interval stream one window at a time (`count` samples from `Start`, one
`Period` apart). `Observer<Row>` is called once per sample of a window;
`WindowObserver<Row, Columns>` answers the whole window with a columns message that
repeats every row field, which suits high-rate streams:

```csharp
template.Observer(new WindowObserver<Counter, CounterColumns>(counter, new Duration { Nanos = 1_000_000 }, (context, window) =>
{
  var columns = new CounterColumns();
  columns.Value.AddRange(Enumerable.Repeat(1UL, checked((int)window.Count)));
  return Task.FromResult(columns);
}, windowSetting: "counter_window"));
```

The window length is the host's `interval_window_ms`; `windowSetting` names a
plugin Duration setting that replaces it for one stream. Interval rows hold one
value per field; repeated, map, oneof and `optional` fields are refused. Time and waits
use `context.Host.Time.Get()` and `.Sleep(duration)`, with the host's controlled
clock during stories. No SDK convenience supplies a retry interval or deadline.

## Call the Host

`context.Host` is the generated plugin-facing resource tree. For example:

```csharp
var settings = await context.Host.Plugins.Settings.List();
var credential = await context.Host.Secrets.Get("instrument");
var progress = await context.Host.Executions.Progress.Get();
```

Declare the corresponding `HostOperation` requirements before serving.
`Plugin.Setting` uses the library's typed setting declarations and defaults.
`Plugin.Secret(name, label, required)` contributes a declared secret name; the
client's `Plugins.Secrets` resource binds that name to a protected host secret.
Plugins can read only their declared names. Secret replies retain protected bytes;
administration metadata belongs to clients.

`Context.RecordAsync(record, value, sourceAt)` serializes a typed record through
`Host.Evidence.Records.Append`. Records.List and Evidence.Statistics use their
harmonized selectors and rich replies. Plugin selectors may be omitted and remain
restricted to the trusted execution. `Context.CallAsync` and `.WatchAsync` also
support explicitly declared instrument operations, preserving cancellation and
scope through the pinned library.

`Plugin.Analysis` registers one Protected computation with raw `AnalysisSamples`
axes, values and units. It returns typed `AnalysisValues`, including full raw axes
and independent display indices. No analysis convenience converts those axes to
plot drawing coordinates.

## Use the Client

```csharp
using Dokime;
using Grpc.Core;
using Metadata = Grpc.Core.Metadata;

using var client = Client.Connect("http://127.0.0.1:7000");
var headers = new Metadata { { "authorization", "Bearer token" } };
using var cancellation = new CancellationTokenSource();
var context = new CallOptions(headers, cancellationToken: cancellation.Token);
var status = await client.Status.Get(context: context);
var entry = await client.Plugins.Get("example", context: context);
var execution = await client.Executions.Get("execution-a", context: context);
```

A `Client.Connect` result owns its channel and implements `IDisposable`.
`new Client(channel.CreateCallInvoker())` borrows a caller-owned transport.
Every operation accepts optional `CallOptions` for one call's metadata, deadline
or cancellation. Watches also accept an enumeration cancellation token.
Operation failures preserve their gRPC status and trailers and identify the action.

Library resources such as Access, Plugins, plugin Settings, Status, Time and
Invocations compose beside product resources such as Executions and Evidence.
Discovery is client-only after composition narrowing. Public plugin calls use
`client.Invocations.Create`; plugin identity does not grant client administration.

`await client.Files.UploadAsync(path, context)` uses the returned `Upload` writer
fence and transfer bounds, checks accepted offsets, and verifies final length and
SHA-256. Failure attempts to abandon the upload and retains both causes if cleanup also
fails. Cleanup uses its callee policy independently of the canceled transfer. A matching extension serves `context.Host.Files` for plugins.
Published replies retain their receipt and file reference in the generated API.

`FrozenSchemaCache` takes explicit page limits, retains the issued descriptor
position across pages, verifies SHA-256 and reuses the complete descriptor set by
digest across executions. Returned descriptor sets are defensive copies.

## Exact Evidence and Documents

`Value` retains signed integers, unsigned integers and `ExactNumber` rational
values. `Statistics` carries typed minimum, maximum, mean, median, percentiles,
first, last and sum. Keep those values typed when calculating or displaying them;
only the standard deviation is inherently floating point. The client example
prints the typed values directly, preserving rational numerators and denominators.

Plot summaries use typed statistics and exact category totals independently of
drawing coordinates. Document binding, summary state, card-completion and retained
evidence-cut frames belong to `Dokime.Clients`; consumers retain their order and
read boundaries. Plot and document conveniences under `Dokime.Plugins` construct
canonical values and clone their inputs.

## Focused Stories

`test` runs SDK-local client and plugin journeys plus document builders.
They cover authentication and per-call context, retained descriptor pages,
materialization, Protected lifecycle bindings, scoped records, explicit observer
windows, evidence evaluation and resource release. Internal carrier descriptors
stay in the test harness. Scenario time uses host windows and awaited conditions.

`test:starter` selects the installed Test example and exercises managed shutdown.
Set `DOKIME_STARTER` to another prepared example executable when needed.
`build:reference` builds the pinned library conformance host with one Cargo job
in the shared `target` directory and installs it at `target/reference-host/bin`.
`HARMOS_CONFORMANCE_HOST` can name another prepared exact-revision host;
`test:conformance` runs its managed and remote fixture journeys. These require
prepared artifacts and never compile inside a test.

`build:system` and `publish:system` prepare the shared C# thermal fixture under
`sdk/tests/fixtures/csharp`. The product story
`csharp_plugin_matches_rust_thermal_evidence` compares its evidence with Rust in
managed and remote placements. Host-backed proof requires the combined core and
shared fixtures to compile. Source migration alone does not establish that proof.
