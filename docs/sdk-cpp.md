# C++ SDK

`sdk/cpp` is a C++20 source SDK. `Dokime::Cpp` provides plugin composition on
Harmos revision `81d32d3bc9ab3e2121746923712ec9508cee3ec7`, generated Dokime
messages and service stubs, a channel-owning client, and the plot/document builders.
Only `.proto` files are authoritative; CMake runs the build's own `protoc` and
`grpc_cpp_plugin`, so generated sources never need to be committed.

## Authoring

The [starter](../sdk/cpp/starter/main.cpp) is the single C++ scaffold source.
Compose `Suite`, `Template`, `Step` and `Criterion` values and pass them by value.
The template's generated materialize operation copies its steps, criteria,
streams and resolved parameters. An optional materialize callback modifies that
case without execution state or host calls. Generated operation identifiers are
Protected; a step's retries become its operation's total attempts.

```cpp
dokime::Template value;
*value.definition.mutable_metadata() = dokime::metadata("sample.check", "Check");
value.steps.push_back({dokime::metadata("measure", "Measure"), [](dokime::Context& ctx) {
    ctx.set(std::make_shared<double>(42));
}});
value.criteria.push_back({dokime::metadata("valid", "Valid"), [](dokime::Context& ctx) {
    return *ctx.get<double>() == 42;
}});
dokime::Plugin plugin("sample", "0.1.0");
plugin.suite({dokime::metadata("sample", "Sample"), {std::move(value)}});
plugin.serve();
```

Execution calls require matching execution scope, template and frozen PluginRun
case. Resources are shared pointers indexed by C++ type, created on the first
execution operation, visible to subsequent operations and observers, and released
after cleanup even when cleanup throws. Resource contents must provide their own
synchronization when steps and observers access them concurrently. Replacing a
resource releases its previous ownership. Failed initialization releases its
resources; process exit releases every remaining resource.

`Stream<Row>` is a Harmos stream; `Observer<Row>` composes a stream, a
`Context&` sampling function and a protobuf duration. The host sends ticks; the SDK
creates no sampling thread or wall-clock timer. Device producers use
`Stream::publish_on` and Harmos `Publisher<Row>`. Observer callbacks may run before
initialize, so they must tolerate unavailable execution resources if the host
opens their stream early.

`Prompt` and `Record` are generated shared declaration values. `record<Row>`
includes the row descriptor through an empty typed Harmos section alongside
`RecordContracts`.
The host must ignore unknown typed sections. `context_operations()` declares the
standard context requirements; explicit `requires_operation` declares narrower
custom dependencies. Context methods include prompt, settings, secrets, record
publication and reads, statistics, progress, logs, artifact file references,
traceability, document parts, and host-clock sleep/now. File capture and effects remain ordinary
typed `ctx.operation` calls. These calls inherit the execution scope. They add no
deadline or retry policy. Operation and transport failures retain Harmos's types.

## Managed and Remote Placement

The starter produces `dokime-plugin.exe` on Windows and `dokime-plugin` on Linux.
With no arguments, Harmos binds `HARMOS_ADDRESS` supplied by the host, or an
ephemeral loopback endpoint, and announces `harmos listening ADDRESS` on stdout.
Stop performs managed shutdown and exits. `--address HOST:PORT` explicitly chooses
remote placement; Stop leaves the remote server running. Bundle the executable
with `dokime plugin bundle --executable PATH --out sample.dokime` once that CLI
flow is integrated.

## Client

```cpp
auto client = dokime::Client::connect("127.0.0.1:7000");
// For each generated service, use client.stub<Service>() to obtain its NewStub.
// Use generated unary/streaming methods with grpc::ClientContext for metadata,
// cancellation and explicit deadline overrides. Client::unary throws RpcError,
// retaining the complete grpc::Status, on a non-OK unary result.
```

Pass a credentialed channel or credentials to `connect` for authenticated TLS
connections. Client/channel/stub ownership uses RAII; no global connection is
created. Generated streaming APIs remain directly available.

CMake generates client services under `dokime::client::v1`, shared messages under
`dokime::v1`, plugin messages under `dokime::plugin::v1`, and host-call messages
under `dokime::plugin::host::v1`. The handler object remains `context`.
Lifecycle bindings retain independent Protected dotted identities and policies.
Artifact publication attaches a library file reference through
`dokime.plugin.host.v1.content.Artifacts/Create`.
The composed descriptor-derived hierarchy exposes `client.executions.progress.get(context, request)` and
`.watch(context, request)` with caller-owned gRPC contexts. Plugin progress uses
separate context Get/Set declarations and whole-value revision checks; declare
both when selecting requirements. Clients have no progress Set.
`dokime/client.hpp` exposes every generated service header. Registry resources are
top-level: `client.suites`, `client.record_schemas`, `client.stream_schemas`,
`client.prompt_definitions` and `client.analyses.list`.
Use `client.stub<dokime::client::v1::Plugins>()` for plugin Add/List and the other plugin
operations. The in-process client story uses the generated Backend and Plugins
services for authentication refusal, identity, complete log stream envelopes,
plugin Add/List and typed server errors. It defines no substitute protocol.
Real-host execution remains separate integration proof.

Execution details retain `schemas().digest()` and display identities. Fetch frozen
descriptors only on a digest cache miss:

```cpp
dokime::FrozenSchemaCache schemas;
auto types = schemas.get(client, detail);
```

The caller owns the cache lifetime and synchronizes access when sharing it between
threads. `get` fetches one file per `executions.schemas.get` page at one cut and
verifies the complete SHA-256 digest before retaining an immutable shared set.
Equal digests reuse it across executions. An optional context configuration callback
forwards metadata, cancellation setup and deadline overrides to every page;
operation policies supply deadlines otherwise. Assemble all files before resolving
descriptor dependencies when using raw calls.

## Setup, Build, Test and Package

Install mise, Git, Python, and a native C++20 compiler (MSVC on Windows;
GCC or Clang, `pkg-config`, curl, zip, unzip and tar on Linux). Then run:

```sh
mise -C sdk/cpp install cmake ninja pipx:clang-format
mise -C sdk/cpp run prepare
mise -C sdk/cpp run format
mise -C sdk/cpp run check
mise -C sdk/cpp run build
mise -C sdk/cpp run test
mise -C sdk/cpp run package
mise -C sdk/cpp run consumer
```

The repository owns its build tooling; Harmos tooling is never invoked. The
manifest baseline `ef7dbf94b9198bc58f45951adcf1f041fcbc5ea0` selects gRPC
1.71.0#1, protobuf 5.29.3 and Catch2 3.8.1. This avoids the gRPC 1.76 fused-filter
code that fails under newer MSVC with `MapResult` / `RaceAsyncCompletion::Run`
([upstream compiler issue](https://github.com/grpc/grpc/issues/41970)).
Local MSVC compilation and hosted Windows Nightly are separate compiler proofs;
only Nightly can validate its current Visual Studio image. No gRPC source patch
or project compiler-cache setting is used.

`CMakePresets.json` provides `development`, `tests` (the default) and `release`.
Pass `-- --preset release` to build, test and package that configuration.
`target/cp/` keeps native and generated paths short on Windows; vcpkg's immutable
tool checkout is prepared in the mise cache and dependency installs live in
`target/cp/v`. `check` configures the graph; `build` compiles; `test` consumes
those outputs. Catch2 3 runs the document, authoring and backend stories, while
the plugin journey awaits readiness and exercises managed and remote placement.

`package` installs and exports a relocatable CMake package to
`target/cp/tests/installed` and archives it as `target/cp/tests/dokime-cpp.tar.gz`.
The archive contains headers, static SDK libraries and CMake targets. Consumers
provide the manifest's matching native dependencies and configure with the
installed prefix in `CMAKE_PREFIX_PATH`:

```cmake
find_package(Dokime CONFIG REQUIRED)
target_link_libraries(my_plugin PRIVATE Dokime::Cpp)
```

`consumer` builds `sdk/cpp/starter` through this installed package, with no
SDK source-tree access. `consumer -- --source /path/to/generated-project` builds
a project produced by `dokime-next new` against the same package. Generated
projects accept the installed archive URL and SHA-256 through their CMake preset;
set `VCPKG_ROOT` to the pinned vcpkg checkout for standalone builds.
The current Harmos pin predates its own installable
package, so the dependency recipe compiles its unmodified source and exports
those targets together with Dokime. The next library pin replaces that recipe
with the upstream package and regenerates client roots and protocol bindings.

## Shared System Story

[`main.cpp`](../sdk/tests/fixtures/cpp/thermal/main.cpp) implements the shared
`thermal.cycle` suite. CMake generates its Temperature and Door payload types
from `sdk/tests/fixtures/cpp/thermal/thermal.proto` and builds `dokime-system-suite` through the
SDK `build` task. The `cpp_plugin_matches_rust_thermal_evidence` host story runs
the executable in remote and managed placements, compares the frozen case and
all ordered evidence/sample fields with Rust, and reopens the sealed evidence.
See [Execution Host](execution-host.md) for prerequisites and focused runs.

## Composed Client Services

`dokime::Client` exposes the library `access`, `status` and `storage` resource
roots over its channel, such as `client.access.listener.get(context, request)`.
Caller-owned gRPC contexts supply authentication metadata, cancellation and
explicit deadline overrides for unary and streaming calls.
