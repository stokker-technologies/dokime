// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Templates;

namespace Stories.Support;

public sealed class ExamplePlugin : Dokime.Plugin
{
  public override string Version => "0.1.0";
  public override IReadOnlyList<Type> Suites => [typeof(CaptureSuite)];
  public int Reads
  {
    get; set;
  }
  public int Cleanups
  {
    get; set;
  }
  public int Disposals
  {
    get; set;
  }
  public bool FailCleanup
  {
    get; set;
  }
}

public sealed class CaptureSuite : Suite
{
  public override string Identifier => "example.capture";
  public override IReadOnlyList<Type> Templates => [typeof(ConfirmationTestTemplate)];
}

public sealed class ConfirmationTestTemplate : TestTemplate, IAsyncDisposable
{
  private ExamplePlugin? plugin;
  public static readonly Record<bool> Confirmation = new("Confirmation");

  [Step("Confirm")]
  public async Task Confirm(Context context)
  {
    plugin = (ExamplePlugin)context.Plugin;
    plugin.Reads++;
    var answer = await context.PromptAsync("Continue?", ["Yes", "No"]);
    await context.RecordAsync(Confirmation, answer == "Yes");
  }

  [Criterion("Confirmed")]
  public Task<bool> Confirmed(Context context) => context.Records(Confirmation, step: Confirm).LastAsync();

  public override Task CleanupAsync(Context context, CancellationToken cancellationToken = default)
  {
    plugin = (ExamplePlugin)context.Plugin;
    plugin.Cleanups++;
    if (plugin.FailCleanup)
      throw new InvalidOperationException("Fixture Cleanup Failure");
    return Task.CompletedTask;
  }

  public ValueTask DisposeAsync()
  {
    if (plugin is not null)
      plugin.Disposals++;
    return ValueTask.CompletedTask;
  }
}
