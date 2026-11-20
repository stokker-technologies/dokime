# Dokime C++ SDK

The C++20 authoring API uses explicit plugin, suite and template classes, ordinary
operation handlers, and separate operation and template contexts. The
[capture example](examples/starter/src/plugin/main.cpp) serves a plugin without arguments.

Plugins include `dokime/dokime.hpp` or `dokime/plugin.hpp` and link `Dokime::Plugin`,
which compiles no product gRPC stubs. Clients include `dokime/client.hpp` and link
`Dokime::Client`, which adds the client stubs on top of `Dokime::Plugin`.

The starter's `mise run build` builds Release plugin and client binaries and stages
the plugin's runtime files in `assets/plugin` for bundling. The `development`
CMake preset keeps Debug outputs in the build directory.

## Operation Lanes

`dokime::Lane` re-exports the library declaration. Create `auto instrument = Lane::serial("instrument");`, assign it with `operation.lane(instrument)`, and register it with `plugin.lane(instrument)`. `Lane::concurrent("workers", 4)` bounds concurrency; `Lane::unbounded("independent")` is unbounded. Plugin descriptions retain lane declarations and operation references; the library rejects undeclared references.

## Author a Plugin

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

Each execution owns a template instance shared by initialize, steps, observers,
criteria and cleanup. Use template fields for execution state. The optional
`initialize(Context&)` and `cleanup(Context&)` methods bind automatically.
Cleanup retains the instance after failed initialization and exposes its
termination through `ctx.termination()`.

Both contexts provide `ctx.plugin<T>()` for the actual author plugin object,
typed operation calls, host-clock sleep and time, cancellation, declared secrets
and saved settings. Only `dokime::templates::Context` provides execution identity,
parameters, prompts, records, statistics, progress, logs, traceability, documents
and effects. Carrier contexts and the untyped host dispatcher are private.
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

`Plugin::serve` resolves its optional address argument, `DOKIME_PLUGIN_ADDRESS`
in the process environment, the working-directory `.env`, then `127.0.0.1:0`.
Managed launch coordinates are private host input and take precedence internally.
An explicitly empty address is an error. The SDK parses UTF-8 dotenv assignments,
quotes and the shared escape grammar without modifying the process environment.

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
handler failures; destructors release resources without throwing. `Error` retains
structured host failure classification, code, cause, details and diagnostics.

## Build and Test

```sh
mise -C sdk/cpp run prepare
mise -C sdk/cpp run generate
mise -C sdk/cpp run format
mise -C sdk/cpp run check -- --preset tests
mise -C sdk/cpp run build -- --preset tests
mise -C sdk/cpp run test -- --preset tests
mise -C sdk/cpp run package -- --preset tests
mise -C sdk/cpp run consumer -- --preset tests
```

`package -- --preset release` writes `target/cp/release/dokime-cpp-<version>-<target>.tar.gz`
and its `.sha256`: headers, the static SDK and library archives, the static gRPC,
protobuf and OpenSSL closure with their code generators, relocatable CMake packages,
`dokime-cpp.json` (build type, compiler, runtime, dependency versions) and
`THIRD-PARTY-NOTICES.txt`. A project sets the extracted root in `CMAKE_PREFIX_PATH`,
or passes `DOKIME_SDK_URL` and `DOKIME_SDK_SHA256` as the starter's presets do, and
builds without a package manager. The Windows package needs Visual Studio 2022 or
later and `CMAKE_MSVC_RUNTIME_LIBRARY=MultiThreadedDLL` before `project()`; the Linux
package needs GCC 11 or later with the C++11 libstdc++ ABI. Other toolchains run
`package` from this checkout to build their own. `consumer -- --source PATH` builds
a project against the package of the selected preset.

Native dependency versions follow the pinned library's vcpkg baseline. They build
once, in Release, as static libraries on the dynamic release runtime
(`x64-windows-static-md-release`, `x64-linux-release`); Debug SDK builds use that
runtime on Windows too. Catch2 installs only when stories are configured. These
tasks use this checkout's `target/cp/` and Debug test preset. The tests cover
class composition, independent binding, duplicate refusal, primitive conversion,
dotenv parsing and managed/remote calls. The C++ thermal host fixture is the
`dokime-system-suite` target. Native Windows and Linux proof are independent.

Use fresh development storage when changing evidence schemas; preserve existing
roots. The [SDK guide](../../docs/sdk-cpp.md) is maintained by the owner.
