// SPDX-License-Identifier: Apache-2.0
using Dokime;
using Dokime.Clients.EvidenceResources;
using Dokime.Plugins;
using Google.Protobuf.WellKnownTypes;
using Harmos;
using ThermalStory;

var door = new Record<Door>("Door Check", identifier: "door");
var temperature = new Dokime.Plugins.Stream<Temperature>("temperature");
var counter = new Dokime.Plugins.Stream<Counter>("counter");
var hold = new Dokime.Plugins.Step("Hold", async context =>
    {
      var answer = await context.PromptAsync("Is the Door Closed?", choices: ["Yes", "No"]);
      var at = await context.Host.Time.Get();
      await context.RecordAsync(door, new Door { Closed = answer == "Yes" }, at);
      await context.Host.Time.Sleep(new Duration { Seconds = 2, Nanos = 500_000_000 });
      var current = await context.Host.Executions.Progress.Get();
      var progress = new ExecutionProgressValue { Completed = 2, Total = 2, Label = "Hold" };
      await context.Host.Executions.Progress.Set(progress, current.Revision);
    }, Identifier: "hold");
var template = new Template("cycle", "Thermal Cycle")
    .Record(door)
    .Parameter(new Parameter { Metadata = new Metadata { Identifier = "limit", Label = "Limit" }, Type = Dokime.ValueType.Number, Expression = new ParameterExpression { Literal = new Dokime.Value { Number = 85 } } })
    .Resource(context => Task.FromResult<IAsyncDisposable>(new Bench(context.Parameters["limit"].Number)))
    .Initialize(async context => await context.LogAsync(Severity.Info, "Initialized"))
    .Observer(new Observer<Temperature>(temperature, new Duration { Seconds = 1 }, context => Task.FromResult(new Temperature { Celsius = context.Resource<Bench>().Limit - 1 })))
    // A 1 kHz Stream Answers Each Host Window at Once.
    .Observer(new WindowObserver<Counter, CounterColumns>(counter, new Duration { Nanos = 1_000_000 }, (context, window) =>
    {
      var columns = new CounterColumns();
      columns.Value.AddRange(Enumerable.Repeat(1_152_921_504_606_847_975UL, checked((int)window.Count)));
      return Task.FromResult(columns);
    }))
    .Step(new Dokime.Plugins.Step("Ramp to Target", async context =>
    {
      var current = await context.Host.Executions.Progress.Get();
      var progress = new ExecutionProgressValue { Completed = 1, Total = 2, Label = "Ramp to Target" };
      await context.Host.Executions.Progress.Set(progress, current.Revision);
    }, Identifier: "ramp", Policy: new OperationPolicy { Retries = new RetryPolicy { TotalAttempts = 3, Initial = new Duration { Nanos = 100_000_000 } } }, ReplaySafe: true))
    .Step(hold)
    .Criterion(new Dokime.Plugins.Criterion("Temperature Within Limit", async context =>
    {
      var reply = await context.StatisticsAsync(temperature, "celsius", step: hold);
      var statistics = reply.Statistics ?? throw new InvalidDataException("Read Statistics: Missing Payload");
      var records = await context.Records(door, step: hold).LastAsync();
      var withinLimit = statistics.Count > 0 && statistics.Max?.KindCase == Dokime.Value.KindOneofCase.Number
          && statistics.Max.Number <= context.Resource<Bench>().Limit;
      var closed = records.Closed;
      return withinLimit && closed;
    }, Identifier: "within_limit"))
    .Cleanup(async context => await context.LogAsync(Severity.Info, "Cleaned Up"));
var plugin = new Plugin("thermal", "1.0.0", new Dokime.Plugins.Suite("thermal", "Thermal", template))
    .Requires(HostOperation.PromptsCreate)
    .Requires(HostOperation.EvidenceRecordsAppend)
    .Requires(HostOperation.EvidenceRecordsList)
    .Requires(HostOperation.EvidenceStatistics)
    .Requires(HostOperation.ExecutionsProgressGet)
    .Requires(HostOperation.ExecutionsProgressSet)
    .Requires(HostOperation.EvidenceLogsAppend)
    .Requires(HostOperation.TimeGet)
    .Requires(HostOperation.TimeSleep);
await plugin.ServeAsync(remote: args.Contains("--remote"));

internal sealed class Bench(double limit) : IAsyncDisposable
{
  internal double Limit { get; } = limit;
  public ValueTask DisposeAsync() => ValueTask.CompletedTask;
}
