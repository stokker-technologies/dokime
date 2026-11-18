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

namespace Stories.Integration;

public sealed class AuthorRunsAMeasurement
{
  [Fact]
  public async Task MaterializeRunObserveEvaluateAndRelease()
  {
    // Compose a Suite with an Owned Instrument, Record and Host-Sampled Stream.
    var instrument = new Instrument();
    var record = new Record<Statistics>("Measurement", identifier: "measurement");
    var stream = new Dokime.Plugins.Stream<Statistics>("readings");
    var template = new Template("measure", "Measure")
        .Record(record)
        .Resource(_ => Task.FromResult<IAsyncDisposable>(instrument))
        .Initialize(context =>
        {
          context.Resource<Instrument>().Initialized = true;
          return Task.CompletedTask;
        })
        .Step(new Dokime.Plugins.Step("Read", async context =>
        {
          context.Resource<Instrument>().Reads++;
          await context.RecordAsync(record, new Statistics { Count = 1, Mean = context.Parameters["target"] },
              sourceAt: new Timestamp { Seconds = 120 }, requestId: "measurement-read");
        }, Identifier: "read", Policy: new OperationPolicy { Retries = new RetryPolicy { TotalAttempts = 3, Initial = new Duration { Nanos = 100_000_000 } } }, ReplaySafe: true))
        .Observer(new Observer<Statistics>(stream, new Duration { Seconds = 1 }, context =>
            Task.FromResult(new Statistics { Count = context.Sequence, Mean = new Dokime.Value { Number = context.Resource<Instrument>().Reads } })))
        .Criterion(new Dokime.Plugins.Criterion("Captured", async context =>
            (await context.Host.Evidence.Records.List(context.Resolve(record), [], new Window(), "", new CapturedRecordSelection(), page: new PageRequest { PageSize = 8, MaxBytes = 4096, PageToken = "continuation" },
                at: new ReadCut { Positions = { new Position { History = new History { Store = "evidence", Epoch = "epoch" }, Sequence = 1 } } })).Count == 1, Identifier: "captured"))
        .Cleanup(context =>
        {
          context.Resource<Instrument>().Cleaned = true;
          return Task.CompletedTask;
        });
    var plugin = new Plugin("journey", "1.0.0", new Dokime.Plugins.Suite("journey", "Journey", template))
        .Requires(HostOperation.EvidenceRecordsAppend).Requires(HostOperation.EvidenceRecordsList);
    await using var server = await PluginServer.StartAsync(plugin);

    // Discover the Catalog and Materialize the Same Resolved Case Twice.
    var description = await server.Wire.DescribeAsync();
    var materialize = new TestCasesMaterializeRequest { Template = "journey.measure", Parameters = { ["target"] = new Dokime.Value { Number = 42 } } };
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
        new StepsExecuteRequest { Execution = "execution-a", Step = "journey.measure.read", TestCase = testCase, Origin = origin }, "execution-a", child =>
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
    var columns = await server.Wire.WindowAsync("journey.measure.readings", "execution-a", new Timestamp { Seconds = 123 }, new Duration { Seconds = 1 }, 2);
    var counts = new RepeatedField<ulong>();
    var means = new RepeatedField<Dokime.Value>();
    var input = new CodedInputStream(columns.ToByteArray());
    for (var tag = input.ReadTag(); tag != 0; tag = input.ReadTag())
    {
      switch (WireFormat.GetTagFieldNumber(tag))
      {
        case Statistics.CountFieldNumber:
          counts.AddEntriesFrom(input, FieldCodec.ForUInt64(tag));
          break;
        case Statistics.MeanFieldNumber:
          means.AddEntriesFrom(input, FieldCodec.ForMessage(tag, Dokime.Value.Parser));
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
        new CriteriaEvaluateRequest { Execution = "execution-a", Criterion = "journey.measure.captured", TestCase = testCase, Origin = origin }, "execution-a",
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
  public async Task DeclareSettingsAndSecrets()
  {
    // Declare a Typed Setting and a Required Secret Without Any Operations.
    var plugin = new Plugin("declarations", "1.0.0")
        .Setting("enabled", new SettingValue { Flag = true })
        .Secret("instrument", "Instrument");
    await using var server = await PluginServer.StartAsync(plugin);
    var description = await server.Wire.DescribeAsync();
    Assert.Empty(description.Declarations.Operations);
    var secrets = SecretsDeclarations.Parser.ParseFrom(description.Declarations.Sections
        .Single(section => section.Type == SecretsDeclarations.Descriptor.FullName).Payload);
    Assert.Equal("instrument", Assert.Single(secrets.Secrets).Metadata.Identifier);
    var settings = SettingsDeclarations.Parser.ParseFrom(description.Declarations.Sections
        .Single(section => section.Type == SettingsDeclarations.Descriptor.FullName).Payload);
    Assert.True(Assert.Single(settings.Values).Default.Flag);
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
