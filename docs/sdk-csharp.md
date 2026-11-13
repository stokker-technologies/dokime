# C# SDK

The development NuGet package is `Dokime` (`0.1.0-dev`, .NET 10). It provides
composed plugin authoring, typed host calls, canonical plot/document builders
and a channel-owning client facade. It builds messages and any declared service
stubs with `Grpc.Tools` directly from `proto/dokime/v1/*.proto`; generated C# is
never committed. The Harmos SDK source is compiled into Dokime without modifications from
revision `26010c03516060deb5f70d17af3fbf0e49d9fbe2`.

## Build and Install

From the repository root:

```sh
mise -C sdk/csharp run build
mise -C sdk/csharp run package
mise -C sdk/csharp run build:starter
mise -C sdk/csharp run build:system
```

Preparation downloads the immutable Harmos source archive. Set `HARMOS_SOURCE`
to a local Harmos Git checkout to read that same commit with `git archive` instead;
the checkout is never written. Python, Git (for the local-source option) and the
SDK's mise-managed .NET 10 toolchain are needed. The package task consumes the
existing Release build and writes `Dokime` to `sdk/csharp/obj/packages/`. The
package contains the pinned Harmos implementation; no separate unpublished Harmos
package is needed. These tasks do not publish to a registry. Packing clears only
the SDK-local consumer package cache so starter proof uses the newly packed bytes.

Restore a consuming project with that feed and nuget.org available, then add:

```xml
<PackageReference Include="Dokime" Version="[0.1.0-dev]" />
```

`starter/` is a package-consuming ordinary executable project. Its own
`mise run build` works with Dokime available in the configured NuGet
feeds. The repository's `build:starter` supplies the local feed explicitly.
The starter has no repository project references and needs no Rust toolchain.
Its apphost executable uses adjacent managed assemblies and the .NET/ASP.NET 10
runtime. A distribution can publish a self-contained single-file executable for
its target before passing that executable to the language-neutral bundler.

## Author a Plugin

Compose `Suite`, `Template`, `Step`, `Criterion`, `Stream<T>`, `Observer<T>` and
`Record<T>` values. Callbacks return `Task`; criterion callbacks return
`Task<bool>`. Use generated protobuf messages for typed data. Declare every host
or instrument call with `Plugin.Requires(operation)` before serving.

```csharp
using Dokime;
using Dokime.V1;

var template = new Template("example.measurement", "Measurement")
    .Step(new Dokime.Step("measure", "Measure", async context =>
        await context.LogAsync(Severity.Info, "Measurement completed")));
var suite = new Dokime.Suite("example", "Example", template);
var plugin = new Dokime.Plugin("example", "1.0.0", suite).Requires(Host.Log);
await plugin.ServeAsync();
```

The complete example is `sdk/csharp/starter/Program.cs`. The managed executable
serves `harmos.plugin.v1.Instance` at `HARMOS_ADDRESS`, which the host supplies. Stop releases
resources and exits. For remote placement, use `--remote`; Stop runs cleanup
while the process remains available. `--address 127.0.0.1:7001` supplies a remote
endpoint when `HARMOS_ADDRESS` is absent. Transport readiness is announced as
`harmos listening HOST:PORT`.

`Template.Resource` creates one `IAsyncDisposable` per execution at its first
operation. `context.Resource<T>()` returns it to subsequent operations and
observer ticks. Cleanup disposes it even when the cleanup callback fails;
server shutdown also releases active resources. Resources are never serialized.
Observers can tick before initialization, so a resource-dependent observer
requires the host to initialize the execution before its first tick.

Materialization is pure: it receives resolved parameters, copies the registered
steps, criteria and streams, and never creates execution resources. Override it
with `Template.Materialize` when parameters select or repeat registered bodies.
Operation names stay stable within the catalog. Steps and criteria must name
registered operations; the host owns expansion, ordering and retries. `Retries`
is the number of retries after the first attempt.

`Context` exposes prompt, record, records, statistics, settings, secret, sleep,
clock, progress and log conveniences. `Host` declares the remaining typed
artifact, file capture, document, traceability, environment and effect operations;
call them with `context.OperationAsync`. All plugin operations are unary even
though Harmos carries their child calls over a duplex transport. Calls inherit
the execution scope; the SDK rejects disagreement between scope, execution,
origin and frozen case.

Record payload types must currently also occur in a stream or another typed
operation in the catalog. Harmos at the pinned revision has no descriptor-only
registration API; `RecordContracts` names a payload type but cannot introduce its
descriptor. The system suite uses its declared stream row for records as well.
A record-only type needs an upstream descriptor registration capability before
that authoring case can be supported without a transport workaround.

## Connect a Client

`Client.Connect(address, options)` owns its channel and implements
`IAsyncDisposable`. `new Client(callInvoker)` borrows an existing transport.
`client.Service(invoker => new GeneratedServiceClient(invoker))` creates a
service client over that transport; call options carry cancellation and metadata.
`GrpcChannelOptions` accepts configured TLS and HTTP handlers.

The build generates all 23 Dokime services, including ordinary and cancellable
`Run<Method>` routes. For example:

```csharp
await using var client = Client.Connect("http://127.0.0.1:8765");
var plugins = client.Service(calls => new Plugins.PluginsClient(calls));
var added = await plugins.AddAsync(new AddRemotePlugin { Url = "http://127.0.0.1:7001" });
var entries = await plugins.ListAsync(new ListPlugins());
```

The in-process fake backend implements generated Backend and Plugins service
bases. Its client journey checks authentication refusal, identity, complete log
stream envelopes, plugin Add/List and typed server errors. `check` verifies that
every declared service has a generated client. Real-host execution remains
separate integration proof.

## Focused Proof

```sh
mise -C sdk/csharp run build:tests
mise -C sdk/csharp run test
mise -C sdk/csharp run test:builders
mise -C sdk/csharp run build:reference
mise -C sdk/csharp run test:conformance
mise -C sdk/csharp run test:starter
mise -C sdk/csharp run format
mise -C sdk/csharp run check
```

The reference task builds only the Harmos conformance binary from the pinned
source in an ignored copy, with one Cargo job. The conformance plugin adapts
Harmos's C# fixture at that revision through `Dokime.Plugin.Build()`. The reference
host checks every transport rule in managed and remote placements; it does not
understand Dokime execution orchestration. Separate journeys directly exercise
the Dokime suite catalog, pure materialization, scoped child calls, explicit
observer ticks, evidence evaluation and resource release. Test setup deadlines
are transport hang guards; no scenario waits for wall-clock time.

## Shared System Story

`tests/SystemPlugin` implements the same thermal suite as Rust, Python and C++.
Build it through `mise -C sdk/csharp run build`, `package` and `build:system`.
The `csharp_plugin_matches_rust_thermal_evidence` host story runs the built
executable through both remote URL and managed placements. It compares the full
frozen case, ordered evidence, source sample times, outcome and counts against
Rust, then reopens each sealed run. The process requires the selected .NET 10
runtime; set `DOTNET_ROOT` to the mise .NET installation when another system
runtime is the default. See [Execution Host](execution-host.md) for named runs.

The named assertion `csharp_frozen_record_schema_decodes_door` passes in both
placements at the pinned Harmos revision. `Record<T>` calls descriptor-only
`Message<T>()`, retaining Door and its dependencies without an artificial
operation, stream or section. The frozen catalog decodes the exact record bytes.

## Composed Client Services

`Dokime.Client` exposes the library `Access`, `Status` and `Storage` resources
over its call invoker. For example, use `client.Access.Listener.Get` and
`client.Access.Pairings.Create` with the matching Harmos requests and call options.
