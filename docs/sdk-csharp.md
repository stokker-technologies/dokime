# C# SDK

The development NuGet package is `Dokime` (`0.1.0-dev`, .NET 10). It provides
composed plugin authoring, typed host calls, canonical plot/document builders
and a channel-owning client facade. It builds messages and any declared service
stubs with `Grpc.Tools` directly from `proto/dokime/**/*.proto`; generated C#
protobuf messages and service stubs are build outputs. Descriptor-derived resource
wrappers in `sdk/csharp/Dokime/Resources.cs` and `ClientResources.cs` are committed
and regenerated through `mise run generate -- --protocol`. The Harmos SDK source
is compiled into Dokime without modifications from revision `81d32d3bc9ab3e2121746923712ec9508cee3ec7`.

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
using Dokime.Protocol.V1;

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
Each materialize, initialize, cleanup, step and criterion binding has its own
Protected operation, dotted identity, handler and policy. Their payloads come
from `Dokime.Protocol.Plugin.V1`; they travel through `Instance.Invoke`, without
independent lifecycle gRPC services. Operation names stay stable within the catalog.
Steps and criteria must name registered operations; the host owns expansion, ordering and retries. `Retries`
is the number of retries after the first attempt.

`Context` exposes prompt, record, records, statistics, settings, secret, sleep,
clock, progress and log conveniences. `Host` declares the remaining typed
artifact, file capture, document, traceability, environment and effect operations;
call them with `context.OperationAsync`. All plugin operations are unary even
though Harmos carries their child calls over a duplex transport. Calls inherit
the execution scope; the SDK rejects disagreement between scope, execution,
origin and frozen case.

`Record<T>` registers the payload descriptor and dependencies through Harmos's
`Message<T>()`. Record-only payloads do not need an artificial operation or stream.

## Connect a Client

`Client.Connect(address, options)` owns its channel and implements
`IAsyncDisposable`. `new Client(callInvoker)` borrows an existing transport.
`client.Service(invoker => new GeneratedServiceClient(invoker))` creates a
service client over that transport; call options carry cancellation and metadata.
`GrpcChannelOptions` accepts configured TLS and HTTP handlers.

The client API uses `Dokime.Protocol.Client.V1` for root resource contracts. Every nested
client package uses a `Resources` suffix in its C# namespace: `BackendResources`,
`ContentResources`, `EvidenceResources`, `ExecutionsResources`,
`MediaResources`, `PlotsResources`, `PluginsResources`, `PromptsResources` and
`StorageResources`, all under `Dokime.Protocol.Client.V1`. This keeps root service types
and nested namespaces distinct. Wire identities retain their face packages, such
as `dokime.client.v1.executions.Progress/Get`.

Registries are top-level resources: `client.Suites`, `client.RecordSchemas`,
`client.StreamSchemas`, `client.PromptDefinitions` and `client.Analyses.List`.
Shared vocabulary stays in `Dokime.Protocol.V1`; plugin lifecycle payloads stay
in `Dokime.Protocol.Plugin.V1`.

Use the descriptor-derived hierarchy with explicit request messages and call
options. For example:

```csharp
await using var client = Client.Connect("http://127.0.0.1:8765");
using var progress = client.Executions.Progress.Get(
    new Dokime.Protocol.Client.V1.ExecutionsResources.ProgressGetRequest
    {
        Name = "execution-id"
    });
var observed = await progress.ResponseAsync;
```

`client.Executions.Progress.Watch` opens a snapshot and follows changes. Reads
resolve latest durable data once; reuse the returned `ResolvedAt` for a consistent
read. Paged resources bind continuation tokens to their selection and cut. Watch
gaps require explicit resynchronization. Downloads use bounded unary responses
and ranged `client.Storage.GetFileBytes` reads; only watches use server streaming.

Plugin child calls use separate `Dokime.Protocol.Plugin.Host.V1` contracts and its
nested `.ContentResources`, `.EvidenceResources`, `.ExecutionsResources` and
`.FilesResources` namespaces. Context
contracts travel through the active invocation, without client gRPC entry points.
`Context.ProgressAsync` requires `Host.ProgressGet` and `Host.ProgressSet` and uses
whole-value revision checks. Clients have no progress Set. Settings and secret
helpers retain their present contracts while awaiting library composition.

The in-process backend journey checks authentication refusal, status identity,
log watch envelopes, case reads and resolved cuts, plugin Add/List and typed server
errors. Plugin management retains its present declaration while awaiting library
composition. `check` verifies generated client coverage for every declared service.
Real-host execution remains separate integration proof.

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
observer ticks, evidence evaluation and resource release. The ordinary `test` task runs three in-process journeys;
`test:builders` runs two document builder checks. Conformance and packaged-starter
stories require the reference host fixture and are excluded from `test`.
Test setup deadlines
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

Traceability entries use distinct context `evidence.Traceability.Get/Set`
contracts. Get selects the execution and entry name and returns its revision;
Set replaces the whole entry with that revision and a caller-retained request ID.
The client root exposes `client.Evidence.Traceability.Get` and `.Watch`, with scoped,
position-resolved history pages and a bounded durable change stream.
