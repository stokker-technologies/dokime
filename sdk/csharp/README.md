# Dokime C# SDK

Author plugins with `Dokime.Plugin` and connect to a running host with
`Dokime.Client`. The [starter](examples/Starter) includes a plugin, client and
inert description tests. Its plugin owns records shared by its templates.

## Plugin Declarations

```csharp
using Dokime.Templates;

public sealed class ExamplePlugin : Dokime.Plugin
{
  public static readonly Record<bool> Confirmation = new("Confirmation", identifier: "confirmation");
  public override IReadOnlyList<IRecord> Records => [Confirmation];
  public override IReadOnlyList<Type> Suites => [typeof(CaptureSuite)];
}

public sealed class CaptureSuite : Dokime.Suite
{
  public override IReadOnlyList<Type> Templates => [typeof(ConfirmationTestTemplate)];
}

public sealed class ConfirmationTestTemplate : TestTemplate
{
  [Step("Confirm")]
  public async Task Confirm(Context context)
  {
    var answer = await context.PromptAsync("Continue?", choices: ["Yes", "No"]);
    await context.RecordAsync(ExamplePlugin.Confirmation, answer == "Yes");
  }

  [Criterion("Confirmed")]
  public Task<bool> Confirmed(Context context) =>
    context.Records(ExamplePlugin.Confirmation, step: Confirm).LastAsync();
}
```

Plugin lifecycle overrides `InitializeAsync` and `FinalizeAsync` receive
`Dokime.Operations.Context`. The SDK registers protected plugin initialize and
finalize operations with empty payloads. Template lifecycle methods retain their
execution context. Shared records are declared through the plugin's `Records`.

## Operations and Failures

Use `await context.Operation("example.measure", new { channel = 1 })` for a JSON
operation. The host validates protobuf field names, schedules every call and
returns replies with default fields present. Catch `Harmos.OperationException`
for `Kind`, `Code`, `Message`, `Action`, `Operation` and `Retry` evidence.
Client calls and plugin handlers carry the same library exception, preserving the
full failure payload, causes and request identity in `Failure.Details`.
Override the plugin's `Failure` property with a mapper returning an
`OperationException` or `null` for library default handling.

Declare ordinary operation methods with `[Dokime.Operations.Operation(...)]`
and list their delegates in the plugin's `Operations`. Resources and settings
are classes listed in `Resources` and `Settings`; handlers read them through
`context.Resource<T>()` and `context.Settings<T>()`. The host enforces each
setting declaration's required flag, choices and bounds.

`NowAsync` and `SleepAsync` use the library clock. Generated product resources
exclude the duplicate clock facade and retain the library dependencies.

## Build and Test

From this SDK directory:

```powershell
mise run build
mise run test
```

`mise run generate -- --write` regenerates the frozen C# projection. Generated
output remains tracked. Use `mise run format` to format authored sources.
The [documentation](../../docs/) is maintained by the owner.
