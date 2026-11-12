// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.V1;
using Google.Protobuf.WellKnownTypes;
using Harmos.Sdk;
using SystemStory;

var decision = new Dokime.Record<Reading>("sdk.journey.decision", "Operator Decision");
var lifecycle = new Dokime.Record<Reading>("sdk.journey.lifecycle", "Lifecycle");
var readings = new Dokime.Stream<Reading>("sdk.journey.readings");
var template = new Template("sdk.journey.measure", "Measure")
    .Parameter(new Parameter
    {
      Metadata = new Metadata { Identifier = "target", Label = "Target" },
      Type = Dokime.V1.ValueType.Number,
      Expression = new ParameterExpression { Literal = new Dokime.V1.Value { Number = 42 } }
    })
    .Resource(context => Task.FromResult<IAsyncDisposable>(new Instrument(context.Parameters["target"].Number)))
    .Initialize(async context => await context.RecordAsync(lifecycle, new Reading { Value = 1 }))
    .Observer(new Observer<Reading>(readings, new Duration { Seconds = 1 }, context =>
        Task.FromResult(new Reading { Value = context.Resource<Instrument>().Target, Tick = context.Sequence })))
    .Step(new Dokime.Step("measure", "Measure", async context =>
    {
      var answer = await context.PromptAsync(new Prompt
      {
        Metadata = new Metadata { Identifier = "sdk.journey.confirm", Label = "Confirm Measurement" },
        Message = "Ready to measure?",
        Archetype = PromptArchetype.Confirmation,
        Actions = { new PromptAction { Identifier = "continue", Label = "Continue" } }
      });
      await context.RecordAsync(decision, new Reading { Value = answer.Accepted ? 1 : 0 });
      await context.SleepAsync(new Duration { Seconds = 3 });
    }, Retries: 1))
    .Criterion(new Dokime.Criterion("evidence", "Evidence Matches", async context =>
    {
      var statistics = await context.StatisticsAsync(readings.Identifier, "value", "measure");
      var records = await context.RecordsAsync(decision.Identifier);
      var target = context.Parameters["target"].Number;
      return statistics.Count == 3 && statistics.Min == target && statistics.Max == target &&
          records.Count == 1 && Reading.Parser.ParseFrom(records.Last.Payload).Value == 1;
    }))
    .Cleanup(async context => await context.RecordAsync(lifecycle, new Reading { Value = 2 }));
var plugin = new Dokime.Plugin("sdk.journey", "1.0.0", new Dokime.Suite("sdk.journey", "SDK Journey", template))
    .Record(decision).Record(lifecycle)
    .Requires(Host.Record).Requires(Host.Records).Requires(Host.Statistics).Requires(Host.Prompt).Requires(Clock.Sleep);
await plugin.ServeAsync(remote: args.Contains("--remote"));

internal sealed class Instrument(double target) : IAsyncDisposable
{
  internal double Target { get; } = target;
  public ValueTask DisposeAsync() => ValueTask.CompletedTask;
}
