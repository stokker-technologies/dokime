# Dokime C# SDK

`Plugin.RunAsync()` registers with the host selected by `DOKIME_REMOTE` in the
process environment or working-directory `.env`. Managed launches take precedence.
Registration sends the SDK version under the plugin's stable identifier.
Closing a client or disposing a registration stops its listener and retains the
host entry. Call `UnregisterAsync()` explicitly to remove it.

`Dokime` provides .NET 10 authoring and bound asynchronous clients.
The public homes are `Dokime`, `Dokime.Operations` and `Dokime.Templates`.

## Operation Lanes

Integration Only: `prepare:harmos` stages the sibling `harmos` main checkout
into this repository's `target/` directory. `HARMOS_SOURCE` can select that checkout.

Both handler contexts support `await ctx.Operation("example.measure", new { channel = 1 })`,
returning a `JsonNode`. The library owns JSON conversion, validation and child admission.

`Dokime.Lane` forwards the library's three lane kinds: `Lane.Serial("instrument")`, `Lane.Concurrent("workers", 4)` and `Lane.Unbounded("independent")`. Register a lane with `plugin.Lane(instrument)` and a handler with `plugin.Operation(new Operation<double, double>("example.measure", lane: instrument), handler)`. The library validates declarations and rejects undeclared lane references. `Describe()` includes lane declarations and each operation's lane reference.

## Author a Plugin

A `Plugin` subclass lists its suite types and annotated static operation delegates.
A `Suite` subclass lists its `TestTemplate` types. Templates declare records,
streams, observers, parameters and methods carrying `Step` or `Criterion` attributes.
Both handler contexts expose the actual author plugin through `Plugin`;
`Dokime.Templates.Context` adds execution evidence and lifecycle operations.
Template `InitializeAsync` and `CleanupAsync` own execution lifetimes. A template that
implements `IAsyncDisposable` is disposed after cleanup, including cleanup failure.
Plugin classes may override `InitializeAsync(Dokime.Operations.Context, CancellationToken)`
and `FinalizeAsync(Dokime.Operations.Context, CancellationToken)`. The SDK encodes
protected `<plugin>.initialize` and `<plugin>.finalize` operations with empty payloads.
Settings groups remain open.

Identifiers strip `Plugin`, `Suite` and `TestTemplate` suffixes and convert word
boundaries to underscores. Templates are always qualified by their suite, and
members by their template. Explicit `Identifier` overrides remain available.
Labels derive independently from type names. Duplicate identifiers are refused.
A short prompt uses `<suite>.<template>.question`; records acknowledge their ordinal.
Retry policies require positive backoff.

## Connect and Run

```csharp
using Dokime;

var client = await Client.ConnectAsync();
var cycle = await client.CycleAsync(label: "Capture");
// Register your plugin, then materialize its declared template type.
await cycle.SealAsync();
var evidence = await cycle.EvidenceAsync();
await client.CloseAsync();
```

`ConnectionSettings` supplies `Remote`, `PluginAddress` and a `TimeSpan`
`ConnectTimeout`. `Remote` resolves from explicit settings, process environment,
then working-directory `.env`. `PluginAddress` overrides a programmatic
registration's listener; otherwise it binds `127.0.0.1:0`. A direct remote
is an HTTP origin with an explicit port. Pairing files retain ordered endpoints,
credentials, pinned HTTPS certificates and host identity.

The starter calls `RunAsync()`: launch credentials select library announcement,
otherwise pairing selects self-registration, and an absent remote selects standalone serving.

`client.Plugins.RegisterAsync(plugin)` serves the plugin, adds its remote placement
and watches until ready. `client.CloseAsync()` retains owned host entries and
closes their endpoints. Transport failures become `SdkError` values retaining the
host failure, cause chain and mutation request identity. Handles also implement `IAsyncDisposable` where applicable.
`cycle.ExecutionAsync(testCase)` binds an execution; `RunAsync()` starts it and
returns progress through the sealed terminal frame. `StartAsync`, `WatchAsync`,
`StopAsync`, `CancelAsync` and `SealAsync` remain separate operations. Cancellation
of observation releases the watch without cancelling a host execution. Seal returns
an `Outcome`; cycle evidence returns verified archive bytes. Execution record and
sample readers retain paging cuts and validate frozen payload schemas before decoding.

The pinned host does not yet expose product managed-bundle upload, plugin note
admission or scoped typed sample subscriptions. Those calls refuse with the required
upstream action, matching the Python reference. Client invocation policy overrides
also require a host contract. They are not claimed as working features.

## Starter

[`examples/Starter`](examples/Starter) is one solution:

```text
Example.slnx
src/Example/                 Shared Plugin, Suites and Operations
src/Example.Plugin/          Plugin Executable
src/Example.Client/          Client Flow
tests/Example.Tests/         Host-Free Description Test
```

The plugin project supplies literal native metadata for `dokime.toml` dynamic
fields. `mise run build` builds Debug outputs and stages plugin runtime companions
under `assets/plugin`. `mise run test` describes the plugin without a host.
`mise run dev` starts a disposable product host. With `DOKIME_REMOTE` configured,
`mise run run` runs the client and saves evidence under `_output/`.

## Develop the SDK

```sh
DOKIME_CSHARP_CONFIGURATION=Debug mise -C sdk/csharp run build
DOKIME_CSHARP_CONFIGURATION=Debug mise -C sdk/csharp run test
mise -C sdk/csharp run format
mise -C sdk/csharp run check
mise -C sdk/csharp run generate -- --write
mise -C sdk/csharp run check:generated
mise -C sdk/csharp run test:created
```

`test:created` requires the prepared command-line binary and host fixtures. It
creates a temporary consumer, points its SDK dependency at this worktree, bundles
and develops it, answers its operator prompt through the CLI and checks sealed evidence.

Generated carriers remain tracked and are projected through the SDK generation
task. They are implementation details; authored examples use product values and
handles. See the [Architecture](../../docs/project/architecture.md) and
[C# SDK Guide](../../docs/sdks/csharp.md) for the owning contracts.
