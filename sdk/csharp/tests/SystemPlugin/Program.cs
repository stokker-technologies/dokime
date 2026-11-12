// SPDX-License-Identifier: Apache-2.0
using Dokime;
using Dokime.V1;
using Google.Protobuf.WellKnownTypes;
using Harmos.Sdk;
using ThermalStory;

var door = new Dokime.Record<Door>("thermal.door", "Door Check");
var temperature = new Dokime.Stream<Temperature>("thermal.temperature");
var template = new Template("thermal.cycle", "Thermal Cycle")
    .Parameter(new Parameter { Metadata = new Metadata { Identifier = "limit", Label = "Limit" }, Type = Dokime.V1.ValueType.Number, Expression = new ParameterExpression { Literal = new Dokime.V1.Value { Number = 85 } } })
    .Resource(context => Task.FromResult<IAsyncDisposable>(new Bench(context.Parameters["limit"].Number)))
    .Initialize(async context => await context.LogAsync(Severity.Info, "Initialized"))
    .Observer(new Observer<Temperature>(temperature, new Duration { Seconds = 1 }, context => Task.FromResult(new Temperature { Celsius = context.Resource<Bench>().Limit - 1 })))
    .Step(new Dokime.Step("ramp", "Ramp to Target", async context => await context.ProgressAsync(1, 2, "Ramp to Target"), Retries: 2))
    .Step(new Dokime.Step("hold", "Hold", async context =>
    {
      var answer = await context.OperationAsync(Host.Prompt, new OperatorQuestion { Prompt = "door", PromptType = "thermal.door" });
      await context.RecordAsync(door, new Door { Closed = answer.Action == "yes" && answer.Accepted });
      await context.SleepAsync(new Duration { Seconds = 2, Nanos = 500000000 });
      await context.ProgressAsync(2, 2, "Hold");
    }))
    .Criterion(new Dokime.Criterion("within_limit", "Temperature Within Limit", async context =>
    {
      var statistics = await context.StatisticsAsync(temperature.Identifier, "celsius", "hold");
      var records = await context.RecordsAsync(door.Identifier, "hold");
      return statistics.Count > 0 && statistics.Max <= context.Resource<Bench>().Limit && records.Last is not null && Door.Parser.ParseFrom(records.Last.Payload).Closed;
    }))
    .Cleanup(async context => await context.LogAsync(Severity.Info, "Cleaned Up"));
var plugin = new Dokime.Plugin("thermal", "1.0.0", new Dokime.Suite("thermal", "Thermal", template))
    .Record(door).Requires(Host.Prompt).Requires(Host.Record).Requires(Host.Records).Requires(Host.Statistics).Requires(Host.Progress).Requires(Host.Log).Requires(Clock.Sleep);
plugin.Build().Section(new PromptCatalog { Prompts = { new Prompt { Metadata = new Metadata { Identifier = "thermal.door", Label = "Door Check" }, Message = "Is the Door Closed?", Archetype = PromptArchetype.Confirmation, Actions = { new PromptAction { Identifier = "yes", Label = "Yes" }, new PromptAction { Identifier = "no", Label = "No" } } } } });
await plugin.ServeAsync(remote: args.Contains("--remote"));

internal sealed class Bench(double limit) : IAsyncDisposable
{
  internal double Limit { get; } = limit;
  public ValueTask DisposeAsync() => ValueTask.CompletedTask;
}
