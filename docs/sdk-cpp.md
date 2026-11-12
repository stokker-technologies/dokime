# C++ SDK

`sdk/cpp` is a C++20 source SDK. `Dokime::Cpp` provides plugin composition on
Harmos revision `a7583836e721c8dc8b135af40a78860ea5945ec6`, generated Dokime
messages and service stubs, a channel-owning client, and the plot/document builders.
Only `.proto` files are authoritative; CMake runs the build's own `protoc` and
`grpc_cpp_plugin`, so generated sources never need to be committed.

## Authoring

The [starter](../sdk/cpp/starter/main.cpp) is the single C++ scaffold source.
Compose `Suite`, `Template`, `Step` and `Criterion` values and pass them by value.
The template's generated materialize operation copies its steps, criteria,
streams and resolved parameters. An optional materialize callback modifies that
case without execution state or host calls. Generated operation identifiers are
private; a step's retries become its operation's total attempts.

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

`Prompt` and `Record` are generated catalog values. `record<Row>` includes the
row descriptor through an empty typed Harmos section alongside `RecordContracts`.
The host must ignore unknown typed sections. `context_operations()` declares the
standard context requirements; explicit `requires_operation` declares narrower
custom dependencies. Context methods include prompt, settings, secrets, record
publication and reads, statistics, progress, logs, artifact chunks, traceability,
document parts, and host-clock sleep/now. File capture and effects remain ordinary
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

The current authoritative `proto/dokime/v1` files declare messages but **no
services**. CMake generates messages and gRPC output for every file and will pick
up service declarations there. Actual host-service stubs and their end-to-end
client proof require the protocol owner to publish those declarations. The
focused client story uses an explicitly test-owned service over real Dokime
messages and an in-process gRPC fake; it does not claim host compatibility.

## Source Distribution and Starter

`mise -C sdk/cpp run archive -- --version VERSION` creates
`target/sdk/cpp/dokime-cpp-VERSION.tar.gz` and its SHA-256 sidecar. The archive
contains `sdk/cpp` and the authoritative `proto` tree in their repository layout.
It also includes this guide, LICENSE and NOTICE. It contains neither generated
bindings nor dependency binaries. Release tooling
must publish these two files together.

The starter uses CMake `FetchContent`, a versioned `DOKIME_SDK_URL`, its required
`DOKIME_SDK_SHA256`, and `SOURCE_SUBDIR sdk/cpp`. The CLI scaffold must write those
release coordinates into `CMakePresets.json` as `cacheVariables`; no guessed
release URL or digest is embedded here. The starter's vcpkg manifest supplies
gRPC, protobuf and OpenSSL. Set `VCPKG_ROOT` to an existing vcpkg checkout, set the
two archive coordinates in the preset, and run `mise run build`. A manual configure
can instead pass `-DDOKIME_SDK_URL=URL -DDOKIME_SDK_SHA256=HASH`. Windows requires Visual Studio
2022 C++ Build Tools and a Windows SDK. Linux requires a C++20 compiler.

## Focused Contributor Proof

Run all commands through mise; no host build is needed for SDK compilation:

```powershell
mise -C sdk/cpp tasks --all
mise -C sdk/cpp run prepare
mise -C sdk/cpp run format
mise -C sdk/cpp run configure -- --harmos-source C:/path/to/harmos --harmos-cache C:/path/to/harmos/target/cpp/dev
mise -C sdk/cpp run build
mise -C sdk/cpp run test
mise -C sdk/cpp run check
mise -C sdk/cpp run conformance -- --host C:/path/to/harmos-conformance.exe
mise -C sdk/cpp run starter -- --harmos-source C:/path/to/harmos --harmos-cache C:/path/to/harmos/target/cpp/dev
mise -C sdk/cpp run starter -- --build-dir ../../target/sdk/cpp-archive-proof --sdk-archive ../../target/sdk/cpp/dokime-cpp-0.1.0.tar.gz --harmos-source C:/path/to/harmos --harmos-cache C:/path/to/harmos/target/cpp/dev
```

Without `--harmos-cache`, configure uses the CMake package dependencies installed
by vcpkg, and fetches the exact Harmos revision unless its source is overridden.
`--harmos-cache` imports the existing Windows Visual Studio Debug gRPC/protobuf
libraries and code generators read-only. It builds the requested Harmos SDK source
anew, sharing only dependencies. Never clean or reconfigure that other worker's
cache. Native Linux proof uses package dependencies and remains separately required.
`--build-dir` selects a fresh output directory for all actions. Complete logs and
CTest JUnit stay under that directory. Tests consume existing build outputs.

The conformance executable reuses the exact Harmos reference fixture source at
configure time, replacing only plugin composition with `dokime::Plugin`. The
Harmos reference host checks its transport rules, child calls, failure propagation,
observer ticks, device streams and managed shutdown. The SDK's focused stories
separately cover catalog composition, duplicate rejection, client authentication,
stream completion, errors and retained document builders. The managed and remote
starter journeys materialize a case, preserve a full-width unsigned parameter,
execute its step and criterion, reject a mismatched scope, release execution
resources through cleanup and check placement-specific Stop behaviour.

## Shared System Story

[`tests/system/main.cpp`](../sdk/cpp/tests/system/main.cpp) builds
`dokime-system-suite`; it is ready for the Nightly cross-language host story and
is not a locally claimed host proof. The host needs to:

1. Connect to managed `sdk.cpp` (or its explicit remote endpoint), read Suites,
   RecordContracts and the `sdk.temperature` stream declaration.
2. Materialize `sdk.journey`, freeze its full case, plugin/template identifiers,
   digest and process start time in PluginRun, and pass matching origin/case and
   execution scope to every operation.
3. Open the execution stream, initialize, run `capture`, answer prompt `door`
   with accepted action `continue`, acknowledge one Boolean `sdk.door` record,
   then send a tick during the one-second host-clock sleep. Initialize owns a
   42-degree resource; the observer publishes its value without real time.
4. Evaluate `within_limit`: statistics for `sdk.temperature.number` during
   `capture` have count greater than zero and max 42; `sdk.door` has count one
   and its last Boolean is true. Expect the criterion to pass.
5. Run cleanup, retain its info log, close the stream, finalize evidence and stop
   the plugin. Compare the same frozen evidence across SDKs. The host owns
   deterministic tick ordering, statistics, prompt resolution and evidence storage.
