// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Protocol.V1;
using Dokime.Protocol.Plugin.V1;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Harmos.V1;
using Harmos.Plugin.V1;
using Stories.Support;
using Xunit;
using Value = Dokime.Protocol.V1.Value;

namespace Stories.Integration;

public sealed class AuthorRunsAMeasurement
{
  [Fact]
  public async Task MaterializeRunObserveEvaluateAndRelease()
  {
    // Compose a Suite with an Owned Instrument, Record and Host-Ticked Stream.
    var instrument = new Instrument();
    var record = new Dokime.Record<global::Dokime.Protocol.Plugin.Host.V1.EvidenceStatisticsResponse>("measurement", "Measurement");
    var stream = new Dokime.Stream<global::Dokime.Protocol.Plugin.Host.V1.EvidenceStatisticsResponse>("journey.readings");
    var template = new Template("journey.measure", "Measure")
        .Resource(_ => Task.FromResult<IAsyncDisposable>(instrument))
        .Initialize(context => { context.Resource<Instrument>().Initialized = true; return Task.CompletedTask; })
        .Step(new Dokime.Step("read", "Read", async context =>
        {
          context.Resource<Instrument>().Reads++;
          await context.RecordAsync(record, new global::Dokime.Protocol.Plugin.Host.V1.EvidenceStatisticsResponse { Count = 1, Mean = context.Parameters["target"] },
              sourceAt: new Timestamp { Seconds = 120 }, requestId: "measurement-read");
        }, Retries: 2))
        .Observer(new Observer<global::Dokime.Protocol.Plugin.Host.V1.EvidenceStatisticsResponse>(stream, new Duration { Seconds = 1 }, context =>
            Task.FromResult(new global::Dokime.Protocol.Plugin.Host.V1.EvidenceStatisticsResponse { Count = context.Sequence, Mean = new Value { Number = context.Resource<Instrument>().Reads } })))
        .Criterion(new Dokime.Criterion("captured", "Captured", async context =>
            (await context.RecordsAsync(record.Identifier, page: new PageRequest { PageSize = 8, MaxBytes = 4096, PageToken = "continuation" },
                at: new Position { History = new History { Store = "evidence", Epoch = "epoch" }, Sequence = 1 })).Count == 1))
        .Cleanup(context => { context.Resource<Instrument>().Cleaned = true; return Task.CompletedTask; });
    var plugin = new Dokime.Plugin("journey", "1.0.0", new Dokime.Suite("journey", "Journey", template))
        .Record(record).Requires(Host.Record).Requires(Host.Records);
    await using var server = await PluginServer.StartAsync(plugin);

    // Discover the Catalog and Materialize the Same Resolved Case Twice.
    var description = await server.Wire.DescribeAsync();
    var materialize = new TestCasesMaterializeRequest { Template = "journey.measure", Parameters = { ["target"] = new Value { Number = 42 } } };
    var testCase = (await server.Wire.InvokeAsync<TestCasesMaterializeResponse>(description, "journey.measure.materialize", materialize)).TestCase;
    var repeated = (await server.Wire.InvokeAsync<TestCasesMaterializeResponse>(description, "journey.measure.materialize", materialize)).TestCase;
    Assert.Equal(testCase, repeated);
    Assert.Equal(3u, description.Catalog.Operations.Single(item => item.Id == testCase.Steps[0].Operation).Policy.Retries.TotalAttempts);
    Assert.Equal(0, instrument.Reads);
    var origin = new Origin { PluginRun = new PluginRun { Plugin = "journey", Template = materialize.Template, TestCase = testCase.Clone(), Digest = description.Catalog.Digest, StartedAt = description.StartedAt } };

    // Initialize the Frozen Case Through Its Own Protected Binding.
    var bindings = new[] { "journey.measure.materialize", "journey.measure.initialize", testCase.Steps[0].Operation,
        testCase.Criteria[0].Operation, "journey.measure.cleanup" };
    Assert.All(bindings, binding => Assert.Equal(Visibility.Protected,
        description.Catalog.Operations.Single(item => item.Id == binding).Visibility));
    await server.Wire.InvokeAsync<ExecutionsInitializeResponse>(description, "journey.measure.initialize",
        new ExecutionsInitializeRequest { Execution = "execution-a", TestCase = testCase, Origin = origin }, "execution-a");
    Assert.True(instrument.Initialized);

    // Execute the Step and Acknowledge Its Typed Record on the Parent Call.
    global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.RecordsAppendRequest? stored = null;
    await server.Wire.InvokeAsync<StepsExecuteResponse>(description, testCase.Steps[0].Operation,
        new StepsExecuteRequest { Execution = "execution-a", Step = "read", TestCase = testCase, Origin = origin }, "execution-a", child =>
        {
          Assert.Equal(Host.Record.Contract.Id, child.Operation);
          stored = global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.RecordsAppendRequest.Parser.ParseFrom(child.Payload);
          return new global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.RecordsAppendResponse { Ordinal = 1 }.ToByteString();
        });
    Assert.Equal("execution-a", stored!.Execution);
    Assert.Equal(new Timestamp { Seconds = 120 }, stored.SourceAt);
    Assert.Equal("measurement-read", stored.RequestId);
    Assert.Equal(42, global::Dokime.Protocol.Plugin.Host.V1.EvidenceStatisticsResponse.Parser.ParseFrom(stored.Payload).Mean.Number);

    // Advance the Observer with One Explicit Host Tick.
    using var publication = server.Wire.Publish();
    await publication.RequestStream.WriteAsync(new InstanceSamplesRequest { Open = new SamplesSelection { Stream = stream.Identifier, Scope = "execution-a" } });
    var at = new Timestamp { Seconds = 123 };
    await publication.RequestStream.WriteAsync(new InstanceSamplesRequest { Tick = new Tick { At = at } });
    Assert.True(await publication.ResponseStream.MoveNext(default));
    var sample = Assert.Single(publication.ResponseStream.Current.Samples);
    Assert.Equal(at, sample.At);
    Assert.Equal(1ul, global::Dokime.Protocol.Plugin.Host.V1.EvidenceStatisticsResponse.Parser.ParseFrom(sample.Row).Count);
    Assert.Equal(1, global::Dokime.Protocol.Plugin.Host.V1.EvidenceStatisticsResponse.Parser.ParseFrom(sample.Row).Mean.Number);
    await publication.RequestStream.CompleteAsync();

    // Evaluate Captured Evidence and Release the Instrument After Cleanup.
    var verdict = await server.Wire.InvokeAsync<CriteriaEvaluateResponse>(description, testCase.Criteria[0].Operation,
        new CriteriaEvaluateRequest { Execution = "execution-a", Criterion = "captured", TestCase = testCase, Origin = origin }, "execution-a",
        child =>
        {
          Assert.Equal(Host.Records.Contract.Id, child.Operation);
          var request = global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.RecordsListRequest.Parser.ParseFrom(child.Payload);
          Assert.Equal(new PageRequest { PageSize = 8, MaxBytes = 4096, PageToken = "continuation" }, request.Page);
          Assert.Equal(new Position { History = new History { Store = "evidence", Epoch = "epoch" }, Sequence = 1 }, request.At);
          return new global::Dokime.Protocol.Plugin.Host.V1.EvidenceResources.RecordsListResponse { Count = stored is null ? 0ul : 1ul }.ToByteString();
        });
    Assert.True(verdict.Passed);
    await server.Wire.InvokeAsync<ExecutionsCleanupResponse>(description, "journey.measure.cleanup",
        new ExecutionsCleanupRequest { Execution = "execution-a", TestCase = testCase, Origin = origin, Termination = Termination.Completed }, "execution-a");
    Assert.True(instrument.Cleaned);
    Assert.Equal(1, instrument.Disposals);
  }

  private sealed class Instrument : IAsyncDisposable
  {
    internal bool Initialized;
    internal int Reads;
    internal bool Cleaned;
    internal int Disposals;
    public ValueTask DisposeAsync() { Disposals++; return ValueTask.CompletedTask; }
  }
}
