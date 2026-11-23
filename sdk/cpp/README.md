# Dokime C++ SDK

The C++20 authoring API uses explicit plugin, suite and template classes, ordinary
operation handlers, and separate operation and template contexts. The
[capture example](examples/starter/src/plugin/main.cpp) serves a plugin without arguments.

Authoring libraries include `dokime/dokime.hpp` or `dokime/plugin.hpp` and link `Dokime::Plugin`,
which compiles no product gRPC stubs. Clients include `dokime/client.hpp` and link
`Dokime::Client`, which adds the client stubs on top of `Dokime::Plugin`.

The starter's `mise run build` builds Release plugin and client binaries and stages
the plugin's runtime files in `assets/plugin` for bundling. The `development`
CMake preset keeps Debug outputs in the build directory.

## Local Integration

Integration Only: source builds select the sibling `harmos` main checkout through
`FETCHCONTENT_SOURCE_DIR_HARMOS` and keep build outputs in this repository's `target/`.

## Operation Calls

Both handler contexts support `ctx.operation("example.measure", dokime::json(R"({"channel":1})"))`,
returning a `google::protobuf::Struct`. The library owns JSON conversion, validation and child admission.

## Operation Lanes

`dokime::Lane` re-exports the library declaration. Create `auto instrument = Lane::serial("instrument");`, assign it with `operation.lane(instrument)`, and register it with `plugin.lane(instrument)`. `Lane::concurrent("workers", 4)` bounds concurrency; `Lane::unbounded("independent")` is unbounded. Plugin descriptions retain lane declarations and operation references; the library rejects undeclared references.

## Author a Plugin

Declare activation resources with `plugin.resource<RemoteApi>()`, where `RemoteApi`
derives from `dokime::operations::Resource` and overrides `open(Context &)` and
`close()`. Handlers call `auto api = ctx.resource<RemoteApi>();` without naming the
plugin class. The SDK opens each type lazily under a lock once per activation,
then closes resources in reverse opening order. An unsuccessful `open` owns cleanup
of its partially initialized state.

Settings use typed classes and synchronous activation snapshots:

```cpp
class ConnectionSettings : public dokime::operations::Settings {
public:
  using Settings::Settings;
  const std::string api_address = setting<std::string>("api_address", "Remote API Address", "127.0.0.1:30059");
  const std::chrono::duration<double> dll_timeout = setting<std::chrono::duration<double>>(
      "dll_timeout", "DLL Host Timeout", std::chrono::seconds(5), std::chrono::seconds(0), std::chrono::seconds(30));
};
```

Register with `plugin.settings<ConnectionSettings>()` and read
`ctx.settings<ConnectionSettings>().dll_timeout`. The inherited constructor loads
saved values before plugin initialization; Retry or restart refreshes the snapshot.
Settings use the library's defaults and inclusive bounds. Supported field types are
`bool`, `std::int64_t`, `double`, `std::string`, and `std::chrono::duration<double>`.
The `setting` declaration accepts `choices` and `required`, enforced by the host. Secret declarations and
`ctx.secret("api_token")` remain independent of resource and settings classes.

```cpp
#include <dokime/plugin.hpp>

class ConfirmationTestTemplate : public dokime::templates::TestTemplate {
public:
  inline static const dokime::templates::Record<bool> confirmation{"confirmation", "Confirmation"};

  void confirm(dokime::templates::Context &ctx) {
    const auto answer = ctx.prompt("Continue?", {"Yes", "No"});
    ctx.record(confirmation, answer == "Yes");
  }
  bool confirmed(dokime::templates::Context &ctx) {
    return ctx.records<&ConfirmationTestTemplate::confirm>(confirmation).last();
  }
  static dokime::templates::Template definition() {
    auto value = dokime::templates::Template::from<ConfirmationTestTemplate>();
    value.record(confirmation);
    value.step(dokime::templates::Step::method<&ConfirmationTestTemplate::confirm>("Confirm"));
    value.criterion(dokime::templates::Criterion::method<&ConfirmationTestTemplate::confirmed>("Confirmed"));
    return value;
  }
};

class CaptureSuite : public dokime::Suite {
public:
  static dokime::Suite definition() {
    auto value = dokime::Suite::from<CaptureSuite>();
    value.template_value(ConfirmationTestTemplate::definition());
    return value;
  }
};

class ExamplePlugin : public dokime::Plugin {
public:
  ExamplePlugin() : dokime::Plugin(dokime::Plugin::from<ExamplePlugin>("0.1.0")) {
    suite(CaptureSuite::definition());
  }
};

int main() {
  ExamplePlugin plugin;
  plugin.serve();
}
```

The suite is `capture`, the template is `capture.confirmation`, and its record is
`capture.confirmation.confirmation`. Class names lose their `Plugin`, `Suite` or
`TestTemplate` suffix. Identifiers use snake case, while labels use Title Case.
Explicit identifiers and labels override the derived values. Empty identifiers,
duplicate declarations and lifecycle-name collisions fail during composition.

`Plugin::from<T>`, `Suite::from<T>` and `Template::from<T>` use the existing
compiler-signature facility on MSVC, GCC and Clang. Declaration functions return
complete values; suites explicitly list their templates. `plugin.describe()`
returns product description values without starting a listener.

## State and Contexts

Plugin author classes may override `initialize(dokime::operations::Context&)` and
`finalize(dokime::operations::Context&)`. The SDK encodes them as protected
`<plugin>.initialize` and `<plugin>.finalize` operations with empty payloads.
Clock reads and waits use the library context. The product generator excludes
duplicate clock resources while retaining their operation dependencies.

Each execution owns a template instance shared by initialize, steps, observers,
criteria and cleanup. Use template fields for execution state. The optional
`initialize(Context&)` and `cleanup(Context&)` methods bind automatically.
Cleanup retains the instance after failed initialization and exposes its
termination through `ctx.termination()`.

Both contexts provide `ctx.plugin<T>()` for the actual author plugin object,
typed operation calls, host-clock sleep and time, cancellation, declared secrets
and saved settings. Only `dokime::templates::Context` provides execution identity,
parameters, prompts, records, statistics, progress, logs, traceability and
documents. Carrier contexts and the untyped host dispatcher are private.
Template contexts also capture files, manage continuous capture, publish artifacts
and read the host process environment. Artifact uploads share the client's fenced
transfer implementation.

Template records use primitive C++ values or generated message values.
`ctx.record` returns the acknowledged ordinal. `ctx.records(record).last()`
refuses an empty selection; `count()` reports the population. Method-qualified
queries select one declared step. Statistics return the statistics value.

## Plain Operations

```cpp
std::string echo(dokime::operations::Context &, std::string request) {
  return request;
}

plugin.operation(dokime::operations::operation<std::string, std::string>(
    "example.echo", dokime::operations::Visibility::Public, echo));
```

`ctx.operation(echo_operation)(request)` returns the typed response. Retry
policies require positive backoff. Authors declare no host requirements.

## Configuration

`Plugin::serve()` selects library announcement from launch credentials,
self-registration from `DOKIME_REMOTE`, or standalone serving when neither is set.
Its executable links `Dokime::Client`; authoring-only consumers link `Dokime::Plugin`.
Programmatic registration accepts an address and otherwise binds `127.0.0.1:0`.
Closing the client closes its listeners and retains the host entries.

`connect(ConnectionSettings)` resolves `remote`, `DOKIME_REMOTE`, then the
working-directory `.env`. It accepts a direct HTTP origin with an explicit port or
a version 1 `.pair` file. Pairing endpoints are attempted in order, each within the
connection budget, retaining certificate pinning, credentials and host identity.

## Bound Client Handles

```cpp
#include <dokime/client.hpp>

auto client = dokime::connect();
client.plugins().register_plugin(ExamplePlugin{});
auto cases = client.cases().materialize("capture.confirmation", "example", "capture");
auto cycle = client.cycle("Capture");
for (const auto &test_case : cases) {
  auto execution = cycle.execution(test_case);
  auto progress = execution.run();
  while (const auto update = progress.next()) {
    // Observe Until the Host Seals the Execution.
  }
  const auto outcome = execution.seal();
}
cycle.seal();
const auto archive = cycle.evidence();
client.close();
```

`client.plugins().register_plugin(ExamplePlugin{})` binds the library-owned listener,
waits for host readiness and binds the declared template types. `client.close()`
unregisters its plugins and closes their listeners. Managed bundle installation, typed stream subscription,
plugin notes and client invocation policy overrides require their upstream host
or carrier contracts.

`run()` starts on the first `next()`. `watch()` reattaches without starting work;
closing or destroying either progress stream releases only observation. Explicit
`stop()` and `cancel()` retain their host meanings. `seal()` returns the stable
product outcome. Lookups preserve cycle/case bindings and refuse foreign cases.
Finite resource lists and typed record/sample readers retain their first read cut.
Record queries provide `read`, `pages`, `count` and `last`; sample queries provide
`collect` and on-demand `pages`, with field and time selections. Frozen schema bindings are
verified and shared by the client.

Cycle/execution notes and artifacts return product values. Evidence downloads
verify retained length and SHA-256. `on_prompt` installs one optional answer
handler; close releases its observation and the channel. Explicit close reports
handler failures; destructors release resources without throwing. `harmos::OperationError` retains
the library failure in `.value`, including classification, code, action, cause,
details, diagnostics and host retry evidence.

## Build and Test

```sh
mise -C sdk/cpp run prepare
mise -C sdk/cpp run generate
mise -C sdk/cpp run format
mise -C sdk/cpp run check -- --preset tests
mise -C sdk/cpp run build -- --preset tests
mise -C sdk/cpp run test -- --preset tests
mise -C sdk/cpp run build -- --preset release --target dokime_client
mise -C sdk/cpp run package
mise -C sdk/cpp run consumer
```

`package -- --preset release` writes `target/cp/release/dokime-cpp-<version>-<target>.tar.gz`
and its `.sha256`: headers, the static SDK and library archives, the static gRPC,
protobuf and OpenSSL closure with their code generators, relocatable CMake packages,
`dokime-cpp.json` (build type, compiler, runtime, dependency versions) and
`THIRD-PARTY-NOTICES`. Test-enabled packages also carry Catch2 for SDK test consumers.
Packaging consumes the matching build and never compiles. A project sets the extracted root in `CMAKE_PREFIX_PATH`,
or passes `DOKIME_SDK_URL` and `DOKIME_SDK_SHA256` as the starter's presets do, and
builds without a package manager. Bundles require CMake 4.2 or newer (the starter
pins 4.2.8); earlier CMake finders do not recognize the native dependency layout.
Windows consumers use the MSVC toolset recorded
in the manifest and `CMAKE_MSVC_RUNTIME_LIBRARY=MultiThreadedDLL` before `project()`.
Linux consumers use the recorded compiler major version, the C++11 libstdc++ ABI,
and glibc no older than the recorded version. Other compiler, standard-library or
runtime combinations require a source build of the SDK and its dependencies;
cross-toolchain ABI compatibility is not promised. `consumer -- --source PATH`
builds a project against the package of the selected preset. SDK tests accept
`build -- --sdk-package PATH` and `test -- --sdk-package PATH`, where PATH is an
extracted SDK or its archive; they compile only the tests and examples against it.

Native dependency versions follow the pinned library's vcpkg baseline. They build
once, in Release, as static libraries on the dynamic release runtime
(`x64-windows-static-md-release`, `x64-linux-release`); Debug SDK builds use that
runtime on Windows too. Catch2 installs only when stories are configured. These
tasks use this checkout's `target/cp/` and Debug test preset. The tests cover
class composition, independent binding, duplicate refusal, primitive conversion,
dotenv parsing and managed/remote calls. The C++ thermal host fixture is the
`dokime-system-suite` target. Native Windows and Linux proof are independent.

Use fresh development storage when changing evidence schemas; preserve existing
roots. The [SDK guide](../../docs/sdks/cpp.md) is maintained by the owner.

## Operation Failures and Shared Records

Operation calls throw the library's `harmos::OperationError`, retaining kind, code,
message, action, operation and retry evidence. `plugin.failure(mapper)` accepts an
exception mapper returning `std::optional<harmos::OperationError>`; unmapped exceptions
become Permanent with their text and handler identity.

Declare shared records with `plugin.record(record)`. Template and operation contexts
write them with `ctx.record(record, value)`. Observer writes reach every execution
in the host call's capture set and no other execution.
