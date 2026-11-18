# Dokime C# SDK

`Dokime` provides .NET 10 plugin authoring and clients. Authoring types live in
`Dokime.Plugins`; plain operations and their visibility live in `Dokime.Authoring`.

## Author a Suite

```csharp
using Dokime.Plugins;

var plugin = new Plugin("example", "0.1.0", Suite.From<Capture>());
await plugin.ServeAsync();

[SuiteAttribute("Capture")]
public sealed class Capture
{
  public static readonly Record<bool> Confirmation = new("Confirmation");

  [StepAttribute("Confirm")]
  public async Task Confirm(Context context)
  {
    var answer = await context.PromptAsync("Continue?", choices: ["Yes", "No"]);
    await context.RecordAsync(Confirmation, answer == "Yes");
  }

  [CriterionAttribute("Confirmed")]
  public Task<bool> Confirmed(Context context) =>
      context.Records(Confirmation, step: Confirm).LastAsync();
}

```

Put the class in its own file and the entry point in `Program.cs` as in
`examples/Test`. The entry point only starts the sidecar. `examples/Flow` starts
that executable and drives its suite through a running host. `examples/Client`
shows client inspection; `examples/Plugin` exposes a plain string operation:

```csharp
using Dokime.Authoring;
using Dokime.Plugins;

var echo = new Operation<string, string>("example.echo", Visibility.Public);
var plugin = new Plugin("example.echo", "1.0.0")
    .Operation(echo, (_, request) => Task.FromResult(request));
```

## Identifiers and Evidence

`Suite.From<Capture>()` declares the single template `capture`, with members
`capture.confirm`, `capture.confirmed` and `capture.confirmation`. Grouped templates
use `new Suite("accuracy", "Accuracy", Template.From<VoltageTemplate>())`, giving
`accuracy.voltage` and `accuracy.voltage.read_voltage` for `ReadVoltage`.
Names come from types, fields and methods, independently of display labels.
A trailing `Template` or `Suite` is removed from type names; word boundaries become
underscores. An explicit `identifier` or attribute `Identifier` takes precedence.
The plugin binds reusable templates without modifying their local declarations.
Duplicate or empty identifiers are refused.

Declare records, streams and `PromptDeclaration` fields on the template class.
`context.PromptAsync(text, choices)` or `context.PromptAsync(declaration)` returns
the selected label. A `PromptDeclaration` can also own a structured `Dokime.Prompt`.
`context.RecordAsync(declaration, value)` accepts plain values or protobuf rows;
`context.Records(declaration, step: boundMethod).LastAsync()` decodes the last row
and refuses empty evidence. `StatisticsAsync(stream, field, step)`
uses the same qualified declaration selectors.

Plain payloads share one conversion for records and operations: `bool` maps to
BoolValue, `string` to StringValue, `int` and `long` to Int64Value, and `float` and
`double` to DoubleValue. Structured protobuf rows retain their message type.
Use `long` and `double` to match Python's integer and floating values exactly.
C# `float` retains its original single precision before conversion.

The SDK supplies host requirements when none are explicitly declared. Explicit
`Requires` declarations remain available, and the host still enforces them.
Policies and timeouts continue to come from operation declarations and settings.
Template fluent composition remains available for parameters, documents,
materialization, observers and execution-owned resources. Record and prompt
registration belongs to the template; there is no plugin record list or raw
section escape hatch. Generated protocol sources remain tracked.

Qualified identifiers change evidence identities. Select a fresh storage root
for an incompatible development root; preserve the existing data.

## Build and Run

```powershell
$env:DOKIME_CSHARP_CONFIGURATION = 'Debug'
mise -C sdk/csharp run build
mise -C sdk/csharp run test
mise -C sdk/csharp run package
mise -C sdk/csharp run build:examples
mise -C sdk/csharp run test:starter
mise -C sdk/csharp run format
mise -C sdk/csharp run check
mise run build:host-fixtures -- --language csharp
```

All compiler outputs stay under the worktree's `target/cs`. Omit the configuration
environment variable to use the existing Release default. Packages and example
builds consume the selected configuration. The standalone starter's `mise run dev`
serves a remote plugin; managed launches receive the host-assigned address.

With a host already running, drive the built starter with:

```powershell
mise -C sdk/csharp exec -- dotnet ../../target/cs/bin/Flow/debug/dokime-example-flow.dll http://127.0.0.1:7000 ../../target/cs/bin/Test/debug/dokime-example-test.exe
```

Use the corresponding executable path on other platforms. The source guide is
[the C# SDK guide](../../docs/sdk-csharp.md), and the reference suite is
[`examples/Test/Capture.cs`](examples/Test/Capture.cs).
