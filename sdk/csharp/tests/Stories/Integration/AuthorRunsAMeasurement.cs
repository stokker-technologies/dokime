// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.V1;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Harmos.V1;
using Harmos.Plugin.V1;
using Stories.Support;
using Xunit;
using Value = Dokime.V1.Value;

namespace Stories.Integration;

public sealed class AuthorRunsAMeasurement
{
  [Fact]
  public async Task MaterializeRunObserveEvaluateAndRelease()
  {
    // Compose a Suite with an Owned Instrument, Record and Host-Ticked Stream.
    var instrument = new Instrument();
    var record = new Dokime.Record<Statistics>("measurement", "Measurement");
    var stream = new Dokime.Stream<Statistics>("journey.readings");
    var template = new Template("journey.measure", "Measure")
        .Resource(_ => Task.FromResult<IAsyncDisposable>(instrument))
        .Step(new Dokime.Step("read", "Read", async context =>
        {
          context.Resource<Instrument>().Reads++;
          await context.RecordAsync(record, new Statistics { Count = 1, Mean = context.Parameters["target"].Number });
        }, Retries: 2))
        .Observer(new Observer<Statistics>(stream, new Duration { Seconds = 1 }, context =>
            Task.FromResult(new Statistics { Count = context.Sequence, Mean = context.Resource<Instrument>().Reads })))
        .Criterion(new Dokime.Criterion("captured", "Captured", async context =>
            (await context.RecordsAsync(record.Identifier)).Count == 1))
        .Cleanup(context => { context.Resource<Instrument>().Cleaned = true; return Task.CompletedTask; });
    var plugin = new Dokime.Plugin("journey", "1.0.0", new Dokime.Suite("journey", "Journey", template))
        .Record(record).Requires(Host.Record).Requires(Host.Records);
    await using var server = await PluginServer.StartAsync(plugin);

    // Discover the Catalog and Materialize the Same Resolved Case Twice.
    var description = await server.Wire.DescribeAsync();
    var materialize = new MaterializeRequest { Template = "journey.measure", Parameters = { ["target"] = new Value { Number = 42 } } };
    var testCase = await server.Wire.InvokeAsync<TestCase>(description, "journey.measure.materialize", materialize);
    var repeated = await server.Wire.InvokeAsync<TestCase>(description, "journey.measure.materialize", materialize);
    Assert.Equal(testCase, repeated);
    Assert.Equal(3u, description.Catalog.Operations.Single(item => item.Id == testCase.Steps[0].Operation).Policy.Retries.TotalAttempts);
    Assert.Equal(0, instrument.Reads);
    var origin = new Origin { PluginRun = new PluginRun { Plugin = "journey", Template = materialize.Template, TestCase = testCase.Clone(), Digest = description.Catalog.Digest, StartedAt = description.StartedAt } };

    // Execute the Step and Acknowledge Its Typed Record on the Parent Call.
    PublishRecord? stored = null;
    await server.Wire.InvokeAsync<StepResponse>(description, testCase.Steps[0].Operation,
        new StepRequest { Execution = "execution-a", Step = "read", TestCase = testCase, Origin = origin }, "execution-a", child =>
        {
          Assert.Equal(Host.Record.Contract.Id, child.Operation);
          stored = PublishRecord.Parser.ParseFrom(child.Payload);
          return new RecordReceipt { Ordinal = 1 }.ToByteString();
        });
    Assert.Equal("execution-a", stored!.Execution);
    Assert.Equal(42, Statistics.Parser.ParseFrom(stored.Payload).Mean);

    // Advance the Observer with One Explicit Host Tick.
    using var publication = server.Wire.Publish();
    await publication.RequestStream.WriteAsync(new InstanceSamplesRequest { Open = new SamplesSelection { Stream = stream.Identifier, Scope = "execution-a" } });
    var at = new Timestamp { Seconds = 123 };
    await publication.RequestStream.WriteAsync(new InstanceSamplesRequest { Tick = new Tick { At = at } });
    Assert.True(await publication.ResponseStream.MoveNext(default));
    var sample = Assert.Single(publication.ResponseStream.Current.Samples);
    Assert.Equal(at, sample.At);
    Assert.Equal(1ul, Statistics.Parser.ParseFrom(sample.Row).Count);
    Assert.Equal(1, Statistics.Parser.ParseFrom(sample.Row).Mean);
    await publication.RequestStream.CompleteAsync();

    // Evaluate Captured Evidence and Release the Instrument After Cleanup.
    var verdict = await server.Wire.InvokeAsync<CriterionResponse>(description, testCase.Criteria[0].Operation,
        new CriterionRequest { Execution = "execution-a", Criterion = "captured", TestCase = testCase, Origin = origin }, "execution-a",
        child => new Records { Count = stored is null ? 0ul : 1ul }.ToByteString());
    Assert.True(verdict.Passed);
    await server.Wire.InvokeAsync<CleanupResponse>(description, "journey.measure.cleanup",
        new CleanupRequest { Execution = "execution-a", TestCase = testCase, Origin = origin, Termination = Termination.Completed }, "execution-a");
    Assert.True(instrument.Cleaned);
    Assert.Equal(1, instrument.Disposals);
  }

  private sealed class Instrument : IAsyncDisposable
  {
    internal int Reads;
    internal bool Cleaned;
    internal int Disposals;
    public ValueTask DisposeAsync() { Disposals++; return ValueTask.CompletedTask; }
  }
}
