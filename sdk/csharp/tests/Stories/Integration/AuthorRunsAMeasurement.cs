// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Clients;
using Dokime.Clients.EvidenceResources;
using Dokime.Plugins;
using Google.Protobuf;
using Google.Protobuf.Collections;
using Google.Protobuf.WellKnownTypes;
using Harmos;
using Stories.Support;
using Xunit;
using Value = Dokime.Value;
using Stream = Dokime.Plugins.Stream<Dokime.Clients.Statistics>;
using Criterion = Dokime.Plugins.Criterion;
using Step = Dokime.Plugins.Step;
using Suite = Dokime.Plugins.Suite;

namespace Stories.Integration;

public sealed class AuthorRunsAMeasurement
{
  [Fact]
  public async Task MaterializeRunObserveEvaluateAndRelease()
  {
    // Compose a Suite with an Owned Instrument, Record and Host-Sampled Stream.
    var instrument = new Instrument();
    var record = new Record<Statistics>("measurement", "Measurement");
    var stream = new Stream("journey.readings");
    var template = new Template("journey.measure", "Measure")
        .Resource(_ => Task.FromResult<IAsyncDisposable>(instrument))
        .Initialize(context =>
        {
          context.Resource<Instrument>().Initialized = true;
          return Task.CompletedTask;
        })
        .Step(new Step("read", "Read", async context =>
        {
          context.Resource<Instrument>().Reads++;
          await context.RecordAsync(record, new Statistics { Count = 1, Mean = context.Parameters["target"] },
              sourceAt: new Timestamp { Seconds = 120 }, requestId: "measurement-read");
        }, Policy: new OperationPolicy { Retries = new RetryPolicy { TotalAttempts = 3, Initial = new Duration { Nanos = 100_000_000 } } }, ReplaySafe: true))
        .Observer(new Observer<Statistics>(stream, new Duration { Seconds = 1 }, context =>
            Task.FromResult(new Statistics { Count = context.Sequence, Mean = new Value { Number = context.Resource<Instrument>().Reads } })))
        .Criterion(new Criterion("captured", "Captured", async context =>
            (await context.Host.Evidence.Records.List(record.Identifier, [], new Window(), "", new CapturedRecordSelection(), page: new PageRequest { PageSize = 8, MaxBytes = 4096, PageToken = "continuation" },
                at: new ReadCut { Positions = { new Position { History = new History { Store = "evidence", Epoch = "epoch" }, Sequence = 1 } } })).Count == 1))
        .Cleanup(context =>
        {
          context.Resource<Instrument>().Cleaned = true;
          return Task.CompletedTask;
        });
    var plugin = new Plugin("journey", "1.0.0", new Suite("journey", "Journey", template))
        .Record(record).Requires(HostOperation.EvidenceRecordsAppend).Requires(HostOperation.EvidenceRecordsList);
    await using var server = await PluginServer.StartAsync(plugin);

    // Discover the Catalog and Materialize the Same Resolved Case Twice.
    var description = await server.Wire.DescribeAsync();
    var materialize = new TestCasesMaterializeRequest { Template = "journey.measure", Parameters = { ["target"] = new Value { Number = 42 } } };
    var testCase = (await server.Wire.InvokeAsync<TestCasesMaterializeResponse>(description, "journey.measure.materialize", materialize)).TestCase;
    var repeated = (await server.Wire.InvokeAsync<TestCasesMaterializeResponse>(description, "journey.measure.materialize", materialize)).TestCase;
    Assert.Equal(testCase, repeated);
    // The Catalog Declares the Procedure Before Any Case Is Materialized.
    var catalog = Suites.Parser.ParseFrom(description.Declarations.Sections.Single(item => item.Type == Suites.Descriptor.FullName).Payload);
    Assert.Equal(testCase.Steps, Assert.Single(Assert.Single(catalog.Suites_).Templates).Steps);
    Assert.Equal(3u, description.Declarations.Operations.Single(item => item.Id == testCase.Steps[0].Operation).Policy.Retries.TotalAttempts);
    Assert.Equal(0, instrument.Reads);
    var origin = new Origin { PluginRun = new PluginRun { Plugin = "journey", Template = materialize.Template, TestCase = testCase.Clone(), Digest = description.Declarations.Digest, StartedAt = description.StartedAt } };

    // Initialize the Frozen Case Through Its Own Protected Binding.
    var bindings = new[] { "journey.measure.materialize", "journey.measure.initialize", testCase.Steps[0].Operation,
        testCase.Criteria[0].Operation, "journey.measure.cleanup" };
    Assert.All(bindings, binding => Assert.Equal(Visibility.Protected,
        description.Declarations.Operations.Single(item => item.Id == binding).Visibility));
    await server.Wire.InvokeAsync<ExecutionsInitializeResponse>(description, "journey.measure.initialize",
        new ExecutionsInitializeRequest { Execution = "execution-a", TestCase = testCase, Origin = origin }, "execution-a");
    Assert.True(instrument.Initialized);

    // Execute the Step and Acknowledge Its Typed Record on the Parent Call.
    RecordsAppendRequest? stored = null;
    await server.Wire.InvokeAsync<StepsExecuteResponse>(description, testCase.Steps[0].Operation,
        new StepsExecuteRequest { Execution = "execution-a", Step = "read", TestCase = testCase, Origin = origin }, "execution-a", child =>
        {
          Assert.Equal(HostContracts.EvidenceRecordsAppend.Contract.Id, child.Operation);
          stored = RecordsAppendRequest.Parser.ParseFrom(child.Payload);
          return new RecordsAppendResponse { Ordinal = 1 }.ToByteString();
        });
    Assert.Equal("execution-a", stored!.Execution);
    Assert.Equal(new Timestamp { Seconds = 120 }, stored.SourceAt);
    Assert.Equal("measurement-read", stored.RequestId);
    Assert.Equal(42, Statistics.Parser.ParseFrom(stored.Payload).Mean.Number);

    // Grant the Observer One Explicit Host Window of Two Samples; Each Sample Sees Its Own Sequence.
    var columns = await server.Wire.WindowAsync(stream.Identifier, "execution-a", new Timestamp { Seconds = 123 }, new Duration { Seconds = 1 }, 2);
    var counts = new RepeatedField<ulong>();
    var means = new RepeatedField<Value>();
    var input = new CodedInputStream(columns.ToByteArray());
    for (var tag = input.ReadTag(); tag != 0; tag = input.ReadTag())
    {
      switch (WireFormat.GetTagFieldNumber(tag))
      {
        case Statistics.CountFieldNumber:
          counts.AddEntriesFrom(input, FieldCodec.ForUInt64(tag));
          break;
        case Statistics.MeanFieldNumber:
          means.AddEntriesFrom(input, FieldCodec.ForMessage(tag, Value.Parser));
          break;
        default:
          input.SkipLastField();
          break;
      }
    }
    Assert.Equal([1ul, 2ul], counts);
    Assert.Equal([1d, 1d], means.Select(mean => mean.Number));

    // Evaluate Captured Evidence and Release the Instrument After Cleanup.
    var verdict = await server.Wire.InvokeAsync<CriteriaEvaluateResponse>(description, testCase.Criteria[0].Operation,
        new CriteriaEvaluateRequest { Execution = "execution-a", Criterion = "captured", TestCase = testCase, Origin = origin }, "execution-a",
        child =>
        {
          Assert.Equal(HostContracts.EvidenceRecordsList.Contract.Id, child.Operation);
          var request = RecordsListRequest.Parser.ParseFrom(child.Payload);
          Assert.Equal(new PageRequest { PageSize = 8, MaxBytes = 4096, PageToken = "continuation" }, request.Page);
          Assert.Equal(new ReadCut { Positions = { new Position { History = new History { Store = "evidence", Epoch = "epoch" }, Sequence = 1 } } }, request.At);
          return new RecordsListResponse { Count = stored is null ? 0ul : 1ul }.ToByteString();
        });
    Assert.True(verdict.Passed);
    await server.Wire.InvokeAsync<ExecutionsCleanupResponse>(description, "journey.measure.cleanup",
        new ExecutionsCleanupRequest { Execution = "execution-a", TestCase = testCase, Origin = origin, Termination = Termination.Completed }, "execution-a");
    Assert.True(instrument.Cleaned);
    Assert.Equal(1, instrument.Disposals);
  }

  [Fact]
  public async Task ComputeWithRawAxesAndDeclareSettingsAndSecrets()
  {
    // Register a Protected Analysis With Raw Axes and a Typed Parameter.
    var definition = new Analysis
    {
      Metadata = new Metadata { Identifier = "analysis.scale" },
      Version = 1,
      MaxSamples = 8,
      MaxOutputValues = 8
    };
    var plugin = new Plugin("analysis", "1.0.0")
        .Setting("enabled", new SettingValue { Flag = true })
        .Secret("instrument", "Instrument")
        .Analysis(definition, (_, samples, parameters) =>
        {
          var output = new AnalysisValues { Key = "scaled", AxisUnit = samples.AxisUnit, Unit = samples.Unit };
          output.Axis.AddRange(samples.Axis);
          output.Values.AddRange(samples.Values.Select(value => value * parameters.Values["factor"].Number));
          return Task.FromResult<IReadOnlyList<AnalysisValues>>([output]);
        });
    await using var server = await PluginServer.StartAsync(plugin);
    var description = await server.Wire.DescribeAsync();
    var binding = Assert.Single(description.Declarations.Operations);
    Assert.Equal(Visibility.Protected, binding.Visibility);
    var secrets = SecretsDeclarations.Parser.ParseFrom(description.Declarations.Sections
        .Single(section => section.Type == SecretsDeclarations.Descriptor.FullName).Payload);
    Assert.Equal("instrument", Assert.Single(secrets.Secrets).Metadata.Identifier);
    var settings = SettingsDeclarations.Parser.ParseFrom(description.Declarations.Sections
        .Single(section => section.Type == SettingsDeclarations.Descriptor.FullName).Payload);
    Assert.True(Assert.Single(settings.Values).Default.Flag);

    // Invoke the Analysis Without Converting Its Axis to Drawing Coordinates.
    var input = new AnalysisSamples { Axis = { 0.25, 0.75 }, Values = { 4, 9 }, AxisUnit = "s", Unit = "V" };
    var reply = await server.Wire.InvokeAsync<AnalysesComputeResponse>(description, binding.Id,
        new AnalysesComputeRequest
        {
          Samples = input,
          Parameters = new MapValue { Values = { ["factor"] = new Value { Number = 2 } } }
        });
    var result = Assert.Single(reply.Outputs);
    Assert.Equal(input.Axis, result.Axis);
    Assert.Equal(input.Values.Select(value => value * 2), result.Values);
    Assert.Equal(input.AxisUnit, result.AxisUnit);
    Assert.Equal(input.Unit, result.Unit);
  }

  private sealed class Instrument : IAsyncDisposable
  {
    internal bool Initialized;
    internal int Reads;
    internal bool Cleaned;
    internal int Disposals;
    public ValueTask DisposeAsync()
    {
      Disposals++;
      return ValueTask.CompletedTask;
    }
  }
}
