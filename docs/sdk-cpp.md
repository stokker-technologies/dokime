# C++ SDK

`sdk/cpp` is a C++20 SDK. `Dokime::Cpp` composes the product's descriptor-derived
client tree, plugin authoring, document builders and public payload barrels with
Harmos revision `b15faaf202a005cf5077b3938557980f7f49fa37`.
CMake consumes the pinned library archive's own targets and installable package;
protobuf and gRPC use the same vcpkg graph for both SDKs.

Request aliases include their resource owner, reserved operation words gain an
underscore, and library policy/operation/watch types retain their owner.
Artifacts.Create is shared by both callers and plugin AnalysisSamples is public.

## Public Surface

Include `dokime/dokime.hpp` for the complete SDK, `dokime/client.hpp` for clients,
or `dokime/plugin.hpp` for plugin authoring. `dokime/messages.hpp` supplies shared
values at the root and boundary-specific values under `dokime::client` and
`dokime::plugin`. Authoring `Suite`, `Template`, `Step`, `Criterion`, `Context` and
`Plugin` live in `dokime::plugin`; root `Suite`, `Step` and `Criterion` are model
values. Document and plot builders remain in `dokime::authoring`.

Generated protobuf headers are installed in internal directories. Public
handwritten headers and examples name only public SDK types; application code
uses the resource tree instead of obtaining service stubs.

## Client

```cpp
#include <dokime/client.hpp>

using dokime::Client;
using dokime::client::StatusGetOptions;
using grpc::ClientContext;

// Connect to the Host.
auto client = Client::connect("127.0.0.1:7000");

// Read Through the Composed Resources.
auto status = client.status.get();
auto plugins = client.plugins.list();

// Override This Call's Metadata and Deadline Through Its Owned Context.
StatusGetOptions options;
options.context = std::make_shared<ClientContext>();
options.context->AddMetadata("authorization", "Bearer credential");
auto authenticated = client.status.get(options);
```

Request content is passed as arguments. Empty requests require no placeholder;
optional request fields and transport overrides belong to the generated operation
options. Each call owns its context unless the caller supplies a shared context.
A streaming result owns that context for its lifetime and exposes `next()` and
`cancel()`. Operation policies supply deadlines and retries; a caller may set a
deadline on one context without changing the operation policy. Unary responses
with exactly one field return that payload; other replies retain their typed
fields and presence.

A channel supplied to `Client` may carry TLS credentials and pinned trust. The
composed client includes library `access`, `status`, `plugins`, `invocations` and
`time`, alongside product `executions`, `files` and evidence resources.
`client.plugins.settings.list()` uses the library settings resource, while
`client.plugins.secrets.list()` uses the product's binding metadata resource.
Discovery reflects the product's narrowed callers.

`dokime::client::upload(client, path, configure)` reads a file in the limits
returned by the upload owner, uses the current fenced Upload after every append,
verifies the published length and SHA-256, and abandons failed production. It
returns a library FileReference. `configure` optionally configures each call's
owned context.

Execution details retain frozen schema digests and display identities:

```cpp
using dokime::client::FrozenSchemaCache;

FrozenSchemaCache schemas;
auto types = schemas.get(client, detail);
```

The cache fetches bounded descriptor pages at one issued cut, verifies the
complete SHA-256 digest and reuses an immutable set for equal digests across
executions. The caller owns the cache lifetime and synchronization. An optional
context callback configures every page's metadata or deadline override.

## Plugin Authoring

The runnable examples are the scaffold sources:

- [Client](../sdk/cpp/examples/client/main.cpp) reads the composed client resources.
- [Plugin](../sdk/cpp/examples/plugin/main.cpp) serves a plugin in either placement.
- [Test](../sdk/cpp/examples/test/main.cpp) declares a suite and template with
  execution resources, a step and a criterion.

```cpp
#include <dokime/plugin.hpp>

using dokime::plugin::Context;
using dokime::plugin::Plugin;
using dokime::plugin::Template;
using dokime::plugin::metadata;

Template value;
*value.definition.mutable_metadata() = metadata("sample.check", "Check");
value.steps.push_back({metadata("measure", "Measure"), [](Context& context) {
    context.set(std::make_shared<double>(42));
}});
value.criteria.push_back({metadata("valid", "Valid"), [](Context& context) {
    return *context.get<double>() == 42;
}});

Plugin plugin("sample", "0.1.0");
plugin.suite({metadata("sample", "Sample"), {std::move(value)}});
plugin.serve();
```

Lifecycle bindings retain independent Protected identities and policies. The
materializer copies frozen steps, criteria, streams and resolved parameters;
an optional callback changes the materialized case without execution state or
host calls. Calls require matching scope, template, plugin identity and frozen
origin. Cleanup releases execution resources even when its callback throws;
failed initialization and process shutdown release their resources too.

`plugin.analysis(definition, compute, policy)` publishes an Analysis and binds its
identifier to a Protected compute operation. The callback receives `Context&`,
`const AnalysisSamples&` and `const MapValue&`, and returns
`std::vector<AnalysisValues>`. Samples retain raw source-time axes, values and
units. The test example scales values with a declared gain parameter while
preserving the input axis; its managed and remote stories invoke that binding.

`context.host()` returns the generated plugin-callable resource tree, including
`evidence`, `executions.progress`, `files`, `plugins.settings`, `secrets` and `time`.
Declare a generated dependency with
`plugin.requires_operation(dokime::plugin::HostOperation::EvidenceStatistics)`.
`context_operations()` declares the standard context dependencies.
Custom operations use the library's typed Operation and explicit visibility;
`context.call(operation, request, policy).get()` retains the library's failure,
policy and cancellation behavior. Client-only administration is absent from the
plugin tree after composition narrowing.

Use `plugin.section(SecretsDeclarations)` to publish declared secret names;
reading a secret uses the scoped host resource. The library owns plugin settings
and time. There is no second product plugin record or clock declaration.

`Stream<Row>` composes the library stream; `Observer<Row>` combines a stream,
`Context&` sampling function and protobuf duration. Host ticks drive sampling,
with no SDK sampling thread or wall-clock timer. Observers may run before
initialization, so they must tolerate unavailable execution resources. Shared
resource contents own their synchronization.

With no arguments the examples use the host-provided endpoint, or an ephemeral
loopback endpoint, and announce readiness. Stop exits a managed process;
`--address HOST:PORT` selects remote placement, whose server survives Stop.
Media capture, FTP capture and the effects journal remain unsupported host work.

## Numeric and Document Values

The public Value retains signed integers, unsigned integers, floating numbers
and ExactNumber fractions as distinct variants. ExactNumber preserves its sign,
base-10 numerator and positive denominator without conversion through double.
Statistics, including sum, and plot summaries retain typed fields and presence.
Category totals remain Value entries separate from drawing coordinates.
Document replies preserve the host's typed layout and completion states.
Plugin analysis callbacks use the public AnalysisSamples export and preserve raw
axes and values independently of plot drawing coordinates.

## Setup, Build, Test and Package

Install mise, Git, Python and a C++20 compiler. Windows uses MSVC; Linux uses GCC
or Clang with `pkg-config`, curl, zip, unzip and tar. Run:

```sh
mise -C sdk/cpp install cmake ninja pipx:clang-format
mise -C sdk/cpp run prepare
mise -C sdk/cpp run generate
mise -C sdk/cpp run format -- --preset tests
mise -C sdk/cpp run check -- --preset tests
mise -C sdk/cpp run build -- --preset tests
mise -C sdk/cpp run test -- --preset tests
mise -C sdk/cpp run package -- --preset tests
mise -C sdk/cpp run consumer -- --preset tests
```

`generate` renders the six descriptor-derived C++ files in temporary storage and
compares them byte for byte, without changing the frozen outputs. The formatter
covers handwritten files in this SDK and its shared C++ fixtures. `check` configures CMake and checks C++
formatting and Python lint, format and types. `build` is the compiler owner;
`test`, `package` and `consumer` refuse when Ninja reports pending producer work,
so a failed build cannot silently reuse older executables or libraries.
CMake lists protocol inputs explicitly, as the pinned library does; adding a
protocol file also updates `sdk/cpp/cmake/Library.cmake`. This keeps the Ninja
freshness check independent of forced glob regeneration.

Presets are `development`, `tests` (default) and `release`. Native/generated
outputs live under `target/cp/<preset>`; vcpkg dependencies use `target/cp/v`.
The pinned manifest baseline selects gRPC 1.71.0#1, protobuf 5.29.3 and Catch2
3.8.1. Windows and Linux remain independent required compiler proof.

`package` installs SDK headers, libraries and CMake exports and produces
`target/cp/<preset>/dokime-cpp.tar.gz`. Consumers supply the matching native
dependencies and set the installed prefix in `CMAKE_PREFIX_PATH`:

```cmake
find_package(Dokime CONFIG REQUIRED)
target_link_libraries(my_plugin PRIVATE Dokime::Cpp)
```

`consumer` builds the test example against the installed package.
`consumer -- --source PATH` selects another example or a generated project.
The examples' presets accept the installed SDK archive URL and SHA-256; their
builds use the pinned vcpkg graph and public headers only. The command-line tool
owns embedding the plugin and test examples in new projects.

The focused SDK stories retain backend authentication, streamed logs, library
plugin administration, frozen schema caching, authoring contracts and both
managed and remote execution. The library's conformance example is consumed
from its pinned archive. The separate shared thermal fixture and real-host
stories require the repaired core and their owner-managed fixture sources;
see [Execution Host](execution-host.md) for those prerequisites.
