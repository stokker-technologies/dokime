// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Templates;
using ThermalStory;

await new ThermalPlugin().RunAsync();

public sealed class ThermalPlugin : Dokime.Plugin
{
  public override string Version => "1.0.0";
  public override IReadOnlyList<Type> Suites => [typeof(ThermalSuite)];
}

public sealed class ThermalSuite : Suite
{
  public override IReadOnlyList<Type> Templates => [typeof(CycleTestTemplate)];
}

public sealed class CycleTestTemplate : TestTemplate
{
  public static readonly Record<Door> Door = new("Door Check");
  public static readonly Dokime.Templates.Stream<Temperature> Temperature = new();
  public static readonly Dokime.Templates.Stream<Counter> Counter = new();
  public static readonly Observer<Temperature> TemperatureObserver = new(Temperature, TimeSpan.FromSeconds(1), context =>
    Task.FromResult(new Temperature { Celsius = (double)context.Parameters["limit"]! - 1 }));
  public static readonly WindowObserver<Counter, CounterColumns> CounterObserver = new(Counter, TimeSpan.FromMilliseconds(1), (context, window) =>
  {
    var columns = new CounterColumns();
    columns.Value.AddRange(Enumerable.Repeat(1_152_921_504_606_847_975UL, checked((int)window.Count)));
    return Task.FromResult(columns);
  });
  public override IReadOnlyList<Parameter> Parameters => [Dokime.Templates.Parameters.Number("limit", "Limit").WithDefault(85.0)];

  public override Task SetupAsync(Context context, CancellationToken cancellationToken = default) => context.LogAsync("Initialized", cancellationToken: cancellationToken);

  [Step("Ramp to Target", TotalAttempts = 3, InitialBackoffSeconds = 0.1, ReplaySafe = true)]
  public Task Ramp(Context context) => context.ProgressAsync(1, 2, "Ramp to Target");

  [Step("Hold")]
  public async Task Hold(Context context)
  {
    var answer = await context.PromptAsync("Is the Door Closed?", ["Yes", "No"]);
    await context.RecordAsync(Door, new Door { Closed = answer == "Yes" });
    await context.SleepAsync(TimeSpan.FromMilliseconds(2500));
    await context.ProgressAsync(2, 2, "Hold");
  }

  [Criterion("Temperature Within Limit")]
  public async Task<bool> WithinLimit(Context context)
  {
    var statistics = await context.StatisticsAsync(Temperature, "celsius", step: Hold);
    var record = await context.Records(Door, step: Hold).LastAsync();
    var withinLimit = statistics.Count > 0 && statistics.Max is double maximum
      && maximum <= (double)context.Parameters["limit"]!;
    return withinLimit && record.Closed;
  }

  public override Task CleanupAsync(Context context, CancellationToken cancellationToken = default) => context.LogAsync("Cleaned Up", cancellationToken: cancellationToken);
}
