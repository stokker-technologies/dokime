// SPDX-License-Identifier: Apache-2.0
using Dokime;
using Dokime.Clients.EvidenceResources;
using Dokime.Plugins;
using Google.Protobuf.WellKnownTypes;
using Harmos;
using ThermalStory;
using Criterion = Dokime.Plugins.Criterion;
using Step = Dokime.Plugins.Step;
using Suite = Dokime.Plugins.Suite;
using Value = Dokime.Value;
using ValueType = Dokime.ValueType;

var door = new Record<Door>("thermal.door", "Door Check");
var temperature = new Stream<Temperature>("thermal.temperature");
var counter = new Stream<Counter>("thermal.counter");
var template = new Template("thermal.cycle", "Thermal Cycle")
    .Parameter(new Parameter { Metadata = new Metadata { Identifier = "limit", Label = "Limit" }, Type = ValueType.Number, Expression = new ParameterExpression { Literal = new Value { Number = 85 } } })
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
    .Step(new Step("ramp", "Ramp to Target", async context =>
    {
      var current = await context.Host.Executions.Progress.Get();
      var progress = new ExecutionProgressValue { Completed = 1, Total = 2, Label = "Ramp to Target" };
      await context.Host.Executions.Progress.Set(progress, current.Revision);
    }, Policy: new OperationPolicy { Retries = new RetryPolicy { TotalAttempts = 3, Initial = new Duration { Nanos = 100_000_000 } } }, ReplaySafe: true))
    .Step(new Step("hold", "Hold", async context =>
    {
      var answer = await context.Host.Prompts.Create("door", "thermal.door", new Dictionary<string, Value>());
      var at = await context.Host.Time.Get();
      await context.RecordAsync(door, new Door { Closed = answer.Action == "yes" && answer.Accepted }, at);
      await context.Host.Time.Sleep(new Duration { Seconds = 2, Nanos = 500_000_000 });
      var current = await context.Host.Executions.Progress.Get();
      var progress = new ExecutionProgressValue { Completed = 2, Total = 2, Label = "Hold" };
      await context.Host.Executions.Progress.Set(progress, current.Revision);
    }))
    .Criterion(new Criterion("within_limit", "Temperature Within Limit", async context =>
    {
      var reply = await context.Host.Evidence.Statistics(temperature.Identifier, [], new Window(), [], "celsius", "hold");
      var statistics = reply.Statistics ?? throw new InvalidDataException("Read Statistics: Missing Payload");
      var records = await context.Host.Evidence.Records.List(door.Identifier, [], new Window(), "hold", new CapturedRecordSelection());
      var withinLimit = statistics.Count > 0 && statistics.Max?.KindCase == Value.KindOneofCase.Number
          && statistics.Max.Number <= context.Resource<Bench>().Limit;
      var closed = records.Last is not null && Door.Parser.ParseFrom(records.Last.Payload).Closed;
      return withinLimit && closed;
    }))
    .Cleanup(async context => await context.LogAsync(Severity.Info, "Cleaned Up"));
var plugin = new Plugin("thermal", "1.0.0", new Suite("thermal", "Thermal", template))
    .Record(door)
    .Requires(HostOperation.PromptsCreate)
    .Requires(HostOperation.EvidenceRecordsAppend)
    .Requires(HostOperation.EvidenceRecordsList)
    .Requires(HostOperation.EvidenceStatistics)
    .Requires(HostOperation.ExecutionsProgressGet)
    .Requires(HostOperation.ExecutionsProgressSet)
    .Requires(HostOperation.EvidenceLogsAppend)
    .Requires(HostOperation.TimeGet)
    .Requires(HostOperation.TimeSleep);
plugin.Build().Section(new PromptDefinitions { Prompts = { new Prompt { Metadata = new Metadata { Identifier = "thermal.door", Label = "Door Check" }, Message = "Is the Door Closed?", Archetype = PromptArchetype.Confirmation, Actions = { new PromptAction { Identifier = "yes", Label = "Yes" }, new PromptAction { Identifier = "no", Label = "No" } } } } });
await plugin.ServeAsync(remote: args.Contains("--remote"));

internal sealed class Bench(double limit) : IAsyncDisposable
{
  internal double Limit { get; } = limit;
  public ValueTask DisposeAsync() => ValueTask.CompletedTask;
}
