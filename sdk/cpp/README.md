# Dokime C++ SDK

The C++20 authoring API uses explicit plugin, suite and template classes, ordinary
operation handlers, and separate operation and template contexts. The
[capture example](examples/test/main.cpp) serves a plugin without arguments.

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

## Build and Test

```sh
mise -C sdk/cpp run prepare
mise -C sdk/cpp run generate
mise -C sdk/cpp run format
mise -C sdk/cpp run check -- --preset tests
mise -C sdk/cpp run build -- --preset tests
mise -C sdk/cpp run test -- --preset tests
mise -C sdk/cpp run consumer -- --preset tests
```

These tasks use this checkout's `target/cp/` and Debug test preset. The tests cover
class composition, independent binding, duplicate refusal, primitive conversion,
dotenv parsing and managed/remote calls. The C++ thermal host fixture is the
`dokime-system-suite` target. Native Windows and Linux proof are independent.

Use fresh development storage when changing evidence schemas; preserve existing
roots. The [SDK guide](../../docs/sdk-cpp.md) is maintained by the owner.
