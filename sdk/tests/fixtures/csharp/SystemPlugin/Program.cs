// SPDX-License-Identifier: Apache-2.0

using System.Collections.Concurrent;
using Dokime;
using Dokime.Templates;
using ThermalStory;

await new ThermalPlugin().RunAsync();

public sealed class ThermalPlugin : Dokime.Plugin
{
  public static readonly Record<Door> Door = new("Door Check", "door");
  public override string Version => "1.0.0";
  public override IReadOnlyList<IRecord> Records => [Door];
  public override IReadOnlyList<Type> Suites => [typeof(ThermalSuite)];
  public override IReadOnlyList<Delegate> Operations => [(Func<Dokime.Operations.Context, byte[], Task<byte[]>>)Echo];
  public override IReadOnlyList<Type> Settings => [typeof(ThermalSettings)];

  [Dokime.Operations.Operation("thermal.echo", Visibility = Dokime.Operations.Visibility.Public)]
  public static Task<byte[]> Echo(Dokime.Operations.Context context, byte[] request) => Task.FromResult(request);
}

public sealed class ThermalSettings : Dokime.Operations.Settings
{
  public override IReadOnlyList<Harmos.SettingDeclaration> Declarations => [new()
  {
    Name = "thermal.load.interval", Title = "Load Interval",
    Default = new() { Duration = Google.Protobuf.WellKnownTypes.Duration.FromTimeSpan(TimeSpan.FromMilliseconds(1)) }
  }];
}

public sealed class ThermalSuite : Suite
{
  public override string Identifier => "thermal.thermal";
  public override IReadOnlyList<Type> Templates => [typeof(CycleTestTemplate), typeof(LoadTestTemplate)];
}

public sealed class CycleTestTemplate : TestTemplate
{
  private static readonly ConcurrentDictionary<string, double> Chamber = new();
  public static readonly Dokime.Templates.Stream<Temperature> Temperature = new();
  public static readonly Dokime.Templates.Stream<Counter> Counter = new();
  // Approach the Chamber Target on Each Granted Sample.
  public static readonly Observer<Temperature> TemperatureObserver = new(Temperature, TimeSpan.FromSeconds(1), context =>
    Task.FromResult(new Temperature { Celsius = Chamber.AddOrUpdate(context.Execution, 23.0 + (84.0 - 23.0) * 0.25, (_, celsius) => celsius + (84.0 - celsius) * 0.25) }));
  public static readonly WindowObserver<Counter, CounterColumns> CounterObserver = new(Counter, TimeSpan.FromMilliseconds(1), (context, window) =>
  {
    var columns = new CounterColumns();
    columns.Value.AddRange(Enumerable.Repeat(1_152_921_504_606_847_975UL, checked((int)window.Count)));
    return Task.FromResult(columns);
  });
  public override IReadOnlyList<Parameter> Parameters => [Dokime.Templates.Parameters.Number("limit", "Limit").WithDefault(85.0)];

  public override Task InitializeAsync(Context context, CancellationToken cancellationToken = default) => context.LogAsync("Initialized", cancellationToken: cancellationToken);

  [Step("Ramp to Target", TotalAttempts = 3, InitialBackoffSeconds = 0.1, ReplaySafe = true)]
  public Task Ramp(Context context) => context.ProgressAsync(1, 2, "Ramp to Target");

  [Step("Hold")]
  public async Task Hold(Context context)
  {
    var answer = await context.PromptAsync("Is the Door Closed?", ["Yes", "No"]);
    await context.RecordAsync(ThermalPlugin.Door, new Door { Closed = answer == "Yes" });
    await context.SleepAsync(TimeSpan.FromMilliseconds(2500));
    await context.ProgressAsync(2, 2, "Hold");
  }

  [Criterion("Temperature Within Limit")]
  public async Task<bool> WithinLimit(Context context)
  {
    var statistics = await context.StatisticsAsync(Temperature, "celsius", step: Hold);
    var record = await context.Records(ThermalPlugin.Door, step: Hold).LastAsync();
    var withinLimit = statistics.Count > 0 && statistics.Max is double maximum
      && maximum <= (double)context.Parameters["limit"]!;
    return withinLimit && record.Closed;
  }

  public override Task CleanupAsync(Context context, CancellationToken cancellationToken = default)
  {
    Chamber.TryRemove(context.Execution, out _);
    return context.LogAsync("Cleaned Up", cancellationToken: cancellationToken);
  }
}

public sealed class LoadTestTemplate : TestTemplate
{
  public override string Label => "Thermal Load";
  public override IReadOnlyList<Parameter> Parameters =>
      [Dokime.Templates.Parameters.Flag("burst", "Burst").WithDefault(false)];
  public static readonly Dokime.Templates.Stream<Counter> Channel000 = Channel("channel_000");
  public static readonly Dokime.Templates.Stream<Counter> Channel001 = Channel("channel_001");
  public static readonly Dokime.Templates.Stream<Counter> Channel002 = Channel("channel_002");
  public static readonly Dokime.Templates.Stream<Counter> Channel003 = Channel("channel_003");
  public static readonly Dokime.Templates.Stream<Counter> Channel004 = Channel("channel_004");
  public static readonly Dokime.Templates.Stream<Counter> Channel005 = Channel("channel_005");
  public static readonly Dokime.Templates.Stream<Counter> Channel006 = Channel("channel_006");
  public static readonly Dokime.Templates.Stream<Counter> Channel007 = Channel("channel_007");
  public static readonly Dokime.Templates.Stream<Counter> Channel008 = Channel("channel_008");
  public static readonly Dokime.Templates.Stream<Counter> Channel009 = Channel("channel_009");
  public static readonly Dokime.Templates.Stream<Counter> Channel010 = Channel("channel_010");
  public static readonly Dokime.Templates.Stream<Counter> Channel011 = Channel("channel_011");
  public static readonly Dokime.Templates.Stream<Counter> Channel012 = Channel("channel_012");
  public static readonly Dokime.Templates.Stream<Counter> Channel013 = Channel("channel_013");
  public static readonly Dokime.Templates.Stream<Counter> Channel014 = Channel("channel_014");
  public static readonly Dokime.Templates.Stream<Counter> Channel015 = Channel("channel_015");
  public static readonly Dokime.Templates.Stream<Counter> Channel016 = Channel("channel_016");
  public static readonly Dokime.Templates.Stream<Counter> Channel017 = Channel("channel_017");
  public static readonly Dokime.Templates.Stream<Counter> Channel018 = Channel("channel_018");
  public static readonly Dokime.Templates.Stream<Counter> Channel019 = Channel("channel_019");
  public static readonly Dokime.Templates.Stream<Counter> Channel020 = Channel("channel_020");
  public static readonly Dokime.Templates.Stream<Counter> Channel021 = Channel("channel_021");
  public static readonly Dokime.Templates.Stream<Counter> Channel022 = Channel("channel_022");
  public static readonly Dokime.Templates.Stream<Counter> Channel023 = Channel("channel_023");
  public static readonly Dokime.Templates.Stream<Counter> Channel024 = Channel("channel_024");
  public static readonly Dokime.Templates.Stream<Counter> Channel025 = Channel("channel_025");
  public static readonly Dokime.Templates.Stream<Counter> Channel026 = Channel("channel_026");
  public static readonly Dokime.Templates.Stream<Counter> Channel027 = Channel("channel_027");
  public static readonly Dokime.Templates.Stream<Counter> Channel028 = Channel("channel_028");
  public static readonly Dokime.Templates.Stream<Counter> Channel029 = Channel("channel_029");
  public static readonly Dokime.Templates.Stream<Counter> Channel030 = Channel("channel_030");
  public static readonly Dokime.Templates.Stream<Counter> Channel031 = Channel("channel_031");
  public static readonly Dokime.Templates.Stream<Counter> Channel032 = Channel("channel_032");
  public static readonly Dokime.Templates.Stream<Counter> Channel033 = Channel("channel_033");
  public static readonly Dokime.Templates.Stream<Counter> Channel034 = Channel("channel_034");
  public static readonly Dokime.Templates.Stream<Counter> Channel035 = Channel("channel_035");
  public static readonly Dokime.Templates.Stream<Counter> Channel036 = Channel("channel_036");
  public static readonly Dokime.Templates.Stream<Counter> Channel037 = Channel("channel_037");
  public static readonly Dokime.Templates.Stream<Counter> Channel038 = Channel("channel_038");
  public static readonly Dokime.Templates.Stream<Counter> Channel039 = Channel("channel_039");
  public static readonly Dokime.Templates.Stream<Counter> Channel040 = Channel("channel_040");
  public static readonly Dokime.Templates.Stream<Counter> Channel041 = Channel("channel_041");
  public static readonly Dokime.Templates.Stream<Counter> Channel042 = Channel("channel_042");
  public static readonly Dokime.Templates.Stream<Counter> Channel043 = Channel("channel_043");
  public static readonly Dokime.Templates.Stream<Counter> Channel044 = Channel("channel_044");
  public static readonly Dokime.Templates.Stream<Counter> Channel045 = Channel("channel_045");
  public static readonly Dokime.Templates.Stream<Counter> Channel046 = Channel("channel_046");
  public static readonly Dokime.Templates.Stream<Counter> Channel047 = Channel("channel_047");
  public static readonly Dokime.Templates.Stream<Counter> Channel048 = Channel("channel_048");
  public static readonly Dokime.Templates.Stream<Counter> Channel049 = Channel("channel_049");
  public static readonly Dokime.Templates.Stream<Counter> Channel050 = Channel("channel_050");
  public static readonly Dokime.Templates.Stream<Counter> Channel051 = Channel("channel_051");
  public static readonly Dokime.Templates.Stream<Counter> Channel052 = Channel("channel_052");
  public static readonly Dokime.Templates.Stream<Counter> Channel053 = Channel("channel_053");
  public static readonly Dokime.Templates.Stream<Counter> Channel054 = Channel("channel_054");
  public static readonly Dokime.Templates.Stream<Counter> Channel055 = Channel("channel_055");
  public static readonly Dokime.Templates.Stream<Counter> Channel056 = Channel("channel_056");
  public static readonly Dokime.Templates.Stream<Counter> Channel057 = Channel("channel_057");
  public static readonly Dokime.Templates.Stream<Counter> Channel058 = Channel("channel_058");
  public static readonly Dokime.Templates.Stream<Counter> Channel059 = Channel("channel_059");
  public static readonly Dokime.Templates.Stream<Counter> Channel060 = Channel("channel_060");
  public static readonly Dokime.Templates.Stream<Counter> Channel061 = Channel("channel_061");
  public static readonly Dokime.Templates.Stream<Counter> Channel062 = Channel("channel_062");
  public static readonly Dokime.Templates.Stream<Counter> Channel063 = Channel("channel_063");
  public static readonly Dokime.Templates.Stream<Counter> Channel064 = Channel("channel_064");
  public static readonly Dokime.Templates.Stream<Counter> Channel065 = Channel("channel_065");
  public static readonly Dokime.Templates.Stream<Counter> Channel066 = Channel("channel_066");
  public static readonly Dokime.Templates.Stream<Counter> Channel067 = Channel("channel_067");
  public static readonly Dokime.Templates.Stream<Counter> Channel068 = Channel("channel_068");
  public static readonly Dokime.Templates.Stream<Counter> Channel069 = Channel("channel_069");
  public static readonly Dokime.Templates.Stream<Counter> Channel070 = Channel("channel_070");
  public static readonly Dokime.Templates.Stream<Counter> Channel071 = Channel("channel_071");
  public static readonly Dokime.Templates.Stream<Counter> Channel072 = Channel("channel_072");
  public static readonly Dokime.Templates.Stream<Counter> Channel073 = Channel("channel_073");
  public static readonly Dokime.Templates.Stream<Counter> Channel074 = Channel("channel_074");
  public static readonly Dokime.Templates.Stream<Counter> Channel075 = Channel("channel_075");
  public static readonly Dokime.Templates.Stream<Counter> Channel076 = Channel("channel_076");
  public static readonly Dokime.Templates.Stream<Counter> Channel077 = Channel("channel_077");
  public static readonly Dokime.Templates.Stream<Counter> Channel078 = Channel("channel_078");
  public static readonly Dokime.Templates.Stream<Counter> Channel079 = Channel("channel_079");
  public static readonly Dokime.Templates.Stream<Counter> Channel080 = Channel("channel_080");
  public static readonly Dokime.Templates.Stream<Counter> Channel081 = Channel("channel_081");
  public static readonly Dokime.Templates.Stream<Counter> Channel082 = Channel("channel_082");
  public static readonly Dokime.Templates.Stream<Counter> Channel083 = Channel("channel_083");
  public static readonly Dokime.Templates.Stream<Counter> Channel084 = Channel("channel_084");
  public static readonly Dokime.Templates.Stream<Counter> Channel085 = Channel("channel_085");
  public static readonly Dokime.Templates.Stream<Counter> Channel086 = Channel("channel_086");
  public static readonly Dokime.Templates.Stream<Counter> Channel087 = Channel("channel_087");
  public static readonly Dokime.Templates.Stream<Counter> Channel088 = Channel("channel_088");
  public static readonly Dokime.Templates.Stream<Counter> Channel089 = Channel("channel_089");
  public static readonly Dokime.Templates.Stream<Counter> Channel090 = Channel("channel_090");
  public static readonly Dokime.Templates.Stream<Counter> Channel091 = Channel("channel_091");
  public static readonly Dokime.Templates.Stream<Counter> Channel092 = Channel("channel_092");
  public static readonly Dokime.Templates.Stream<Counter> Channel093 = Channel("channel_093");
  public static readonly Dokime.Templates.Stream<Counter> Channel094 = Channel("channel_094");
  public static readonly Dokime.Templates.Stream<Counter> Channel095 = Channel("channel_095");
  public static readonly Dokime.Templates.Stream<Counter> Channel096 = Channel("channel_096");
  public static readonly Dokime.Templates.Stream<Counter> Channel097 = Channel("channel_097");
  public static readonly Dokime.Templates.Stream<Counter> Channel098 = Channel("channel_098");
  public static readonly Dokime.Templates.Stream<Counter> Channel099 = Channel("channel_099");
  public static readonly Dokime.Templates.Stream<Counter> Channel100 = Channel("channel_100");
  public static readonly Dokime.Templates.Stream<Counter> Channel101 = Channel("channel_101");
  public static readonly Dokime.Templates.Stream<Counter> Channel102 = Channel("channel_102");
  public static readonly Dokime.Templates.Stream<Counter> Channel103 = Channel("channel_103");
  public static readonly Dokime.Templates.Stream<Counter> Channel104 = Channel("channel_104");
  public static readonly Dokime.Templates.Stream<Counter> Channel105 = Channel("channel_105");
  public static readonly Dokime.Templates.Stream<Counter> Channel106 = Channel("channel_106");
  public static readonly Dokime.Templates.Stream<Counter> Channel107 = Channel("channel_107");
  public static readonly Dokime.Templates.Stream<Counter> Channel108 = Channel("channel_108");
  public static readonly Dokime.Templates.Stream<Counter> Channel109 = Channel("channel_109");
  public static readonly Dokime.Templates.Stream<Counter> Channel110 = Channel("channel_110");
  public static readonly Dokime.Templates.Stream<Counter> Channel111 = Channel("channel_111");
  public static readonly Dokime.Templates.Stream<Counter> Channel112 = Channel("channel_112");
  public static readonly Dokime.Templates.Stream<Counter> Channel113 = Channel("channel_113");
  public static readonly Dokime.Templates.Stream<Counter> Channel114 = Channel("channel_114");
  public static readonly Dokime.Templates.Stream<Counter> Channel115 = Channel("channel_115");
  public static readonly Dokime.Templates.Stream<Counter> Channel116 = Channel("channel_116");
  public static readonly Dokime.Templates.Stream<Counter> Channel117 = Channel("channel_117");
  public static readonly Dokime.Templates.Stream<Counter> Channel118 = Channel("channel_118");
  public static readonly Dokime.Templates.Stream<Counter> Channel119 = Channel("channel_119");
  public static readonly Dokime.Templates.Stream<Counter> Channel120 = Channel("channel_120");
  public static readonly Dokime.Templates.Stream<Counter> Channel121 = Channel("channel_121");
  public static readonly Dokime.Templates.Stream<Counter> Channel122 = Channel("channel_122");
  public static readonly Dokime.Templates.Stream<Counter> Channel123 = Channel("channel_123");
  public static readonly Dokime.Templates.Stream<Counter> Channel124 = Channel("channel_124");
  public static readonly Dokime.Templates.Stream<Counter> Channel125 = Channel("channel_125");
  public static readonly Dokime.Templates.Stream<Counter> Channel126 = Channel("channel_126");
  public static readonly Dokime.Templates.Stream<Counter> Channel127 = Channel("channel_127");
  public static readonly Dokime.Templates.Stream<Counter> Burst = Channel("burst", "thermal.load.interval");

  private static Dokime.Templates.Stream<Counter> Channel(string identifier, string intervalSetting = "")
  {
    var stream = new Dokime.Templates.Stream<Counter>(identifier);
    _ = new WindowObserver<Counter, CounterColumns>(stream, TimeSpan.FromMilliseconds(1), (context, window) =>
    {
      var columns = new CounterColumns();
      columns.Value.AddRange(Enumerable.Repeat(1_152_921_504_606_847_975UL, checked((int)window.Count)));
      return Task.FromResult(columns);
    }, intervalSetting: intervalSetting);
    return stream;
  }

  public override TestCase Materialize(TestCase testCase)
  {
    var burst = testCase.Parameters["burst"].Boolean;
    var selected = testCase.Streams.Where(stream => stream.EndsWith(".burst", StringComparison.Ordinal) == burst).ToArray();
    testCase.Streams.Clear();
    testCase.Streams.AddRange(selected);
    return testCase;
  }

  public override Task InitializeAsync(Context context, CancellationToken cancellationToken = default) =>
      context.ProgressAsync(0, 1, "Hold", cancellationToken: cancellationToken);

  [Step("Hold")]
  public Task Hold(Context context) => context.CancelledAsync();
}
