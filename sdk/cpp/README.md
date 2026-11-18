# Dokime C++ SDK

The C++20 SDK provides template declarations, execution context operations and a
composed host client. The [test example](examples/test/main.cpp) is the starter;
its entry point serves the sidecar and drains the event engine at shutdown.
The [client example](examples/client/main.cpp) connects to an existing host.

## Author a Suite

```cpp
#include <dokime/plugin.hpp>

using dokime::plugin::Context;
using dokime::plugin::Criterion;
using dokime::plugin::Plugin;
using dokime::plugin::Record;
using dokime::plugin::Step;
using dokime::plugin::Suite;
using dokime::plugin::Template;

class Capture {
public:
  inline static const Record<bool> confirmation{"confirmation", "Confirmation"};

  void confirm(Context &context) {
    const auto answer = context.prompt("Continue?", {"Yes", "No"});
    context.record(confirmation, answer == "Yes");
  }
  bool confirmed(Context &context) {
    return context.records<&Capture::confirm>(confirmation).last();
  }
  static void declare(Template &value) {
    value.record(confirmation);
    value.steps.push_back(Step::method<&Capture::confirm>("Confirm"));
    value.criteria.push_back(Criterion::method<&Capture::confirmed>("Confirmed"));
  }
};

Plugin plugin("example", "0.1.0", {Suite::from<Capture>("Capture")});
plugin.serve();
```

The template is `capture`; its members are `capture.confirmation`,
`capture.confirm` and `capture.confirmed`. The confirmation record's message is
`google.protobuf.BoolValue`, exactly as in the Python example.

`Suite("accuracy", {Template::from<VoltageTemplate>()})` produces
`accuracy.voltage`; `readVoltage`, `ReadVoltage` and `read_voltage` produce
`accuracy.voltage.read_voltage`. Type names lose a trailing `Template` or `Suite`.
Names come from code, independently of labels. An optional explicit identifier
wins, including on a suite, template or member. Empty identifiers and duplicate
members across declaration kinds are refused before registration.

C++20 has no declaration reflection. `Template::from<T>` and
`Step::method<&T::method>` read compiler signatures on MSVC, GCC and Clang.
Record, prompt and stream declarations supply their field's code name explicitly.
Declarations remain local and reusable across plugins and suites.

Declare records with `value.record(local_record)` inside the template's `declare`
method. Declare prompts with `value.prompts.push_back(local_prompt)` and observers
with `value.observer(observer)`. A `PromptDeclaration` accepts text and choices or
a complete `dokime::Prompt`; `context.prompt(declaration)` returns its chosen label.
Text prompts default to a single OK action. Rejected or unknown answers fail.
`context.records<&T::step>(record).last()` returns the declared value and refuses
an empty result. `context.statistics<&T::step>(stream, "field")` resolves both
local declarations. `initialize(Context&)` and `cleanup(Context&)` methods bind
automatically and share one template instance per execution.

`Template` carries typed `parameters`, `files` and `documents`. A pure
`materialize(Case&, parameters)` callback adds declared `Step` and `Criterion`
values with unique local instance identifiers; `Case` qualifies them through the
same identifier function while preserving the declared operation identity.

## Plain Operations

```cpp
using dokime::plugin::Visibility;
using dokime::plugin::operation;

auto echo = operation<std::string, std::string>(
    "example.echo", Visibility::Public,
    [](Context &, std::string request) { return request; });
plugin.operation(echo);
auto relay = operation<std::string, std::string>(
    "example.relay", Visibility::Public,
    [echo](Context &context, std::string request) {
      return context.operation(echo, request);
    });
plugin.operation(relay);
```

Records and operations share conversions for `std::string`, `bool`, signed
integers and floating values: StringValue, BoolValue, Int64Value and DoubleValue.
Structured protobuf messages retain their type. A narrower signed integer refuses
out-of-range returned values. Visibility is `Visibility::Private`,
`Visibility::Protected` or `Visibility::Public`. Operation policies and replay
safety remain explicit; calls retain the transport's policy and cancellation.

Plugin sections are assembled from typed suites, records, prompts and
`plugin.secrets(SecretsDeclarations)`. Context requirements are assembled by the
SDK. Authors do not maintain a requirement list.

## Build and Test

```sh
mise -C sdk/cpp run prepare
mise -C sdk/cpp run format
mise -C sdk/cpp run check -- --preset tests
mise -C sdk/cpp run build -- --preset tests
mise -C sdk/cpp run test -- --preset tests
```

These commands use this checkout's `target/cp/` and the Debug test preset. The
SDK tests cover generated identifiers, independent qualification, duplicate
refusal, primitive conversion and managed/remote sidecar calls. Build the C++
thermal host fixture with the SDK target `dokime-system-suite`.
Native Windows and Linux validation are independent obligations.

Qualified evidence identities require a fresh development storage root; preserve
existing roots instead of resetting them. See the [SDK guide](../../docs/sdk-cpp.md)
for client, packaging and installation details; its authoring examples need the
owner's corresponding update.
