// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Plugins;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Stories.Support;
using Xunit;

namespace Stories.Integration;

public sealed class AuthorDeclaresQualifiedMembers
{
  [Fact]
  public async Task CodeNamesQualifyEveryDeclarationAndPreserveExplicitIdentifiers()
  {
    var local = Template.From<HTTPAccuracySuite>();
    var first = new Plugin("one", "1.0.0", new Dokime.Plugins.Suite("accuracy", "Ignored", local));
    var second = new Plugin("two", "1.0.0", new Dokime.Plugins.Suite("production", "Ignored", local));
    var template = first.Suites[0].Templates[0];
    Assert.Equal("accuracy.http_accuracy", template.Definition.Metadata.Identifier);
    Assert.Equal("accuracy.http_accuracy.read_voltage", template.Resolve((Func<Context, Task>)new HTTPAccuracySuite().ReadVoltage));
    Assert.Equal("accuracy.http_accuracy.rms", template.Resolve(HTTPAccuracySuite.Reading));
    Assert.Equal("production.http_accuracy.rms", second.Suites[0].Templates[0].Resolve(HTTPAccuracySuite.Reading));
    Assert.Equal("http_accuracy", local.Definition.Metadata.Identifier);
    Assert.Throws<ArgumentException>(() => template.Resolve("accuracy.http_accuracy.rms"));

    await using var server = await PluginServer.StartAsync(first);
    var description = await server.Wire.DescribeAsync();
    var materialized = await server.Wire.InvokeAsync<TestCasesMaterializeResponse>(description,
        "accuracy.http_accuracy.materialize", new TestCasesMaterializeRequest { Template = "accuracy.http_accuracy" });
    Assert.Equal("accuracy.http_accuracy.read_voltage", Assert.Single(materialized.TestCase.Steps).Metadata.Identifier);
    Assert.Equal("accuracy.http_accuracy.read_voltage", Assert.Single(materialized.TestCase.Steps).Operation);
    Assert.Equal("accuracy.http_accuracy.reading_ok", Assert.Single(materialized.TestCase.Criteria).Metadata.Identifier);
    Assert.Equal("accuracy.http_accuracy.samples", Assert.Single(materialized.TestCase.Streams));
    var section = description.Declarations.Sections.Single(item => item.Type == PromptDefinitions.Descriptor.FullName);
    var prompt = Assert.Single(PromptDefinitions.Parser.ParseFrom(section.Payload).Prompts);
    Assert.Equal("accuracy.http_accuracy.question", prompt.Metadata.Identifier);
    Assert.Equal("accuracy", prompt.OwnerSuite);
    Assert.Equal("accuracy.http_accuracy", prompt.Template);
    Assert.True(Assert.Single(prompt.Fields).Required);
    Assert.Equal("save", Assert.Single(prompt.Actions).Identifier);
  }

  [Fact]
  public async Task SharedCaptureSuiteUsesThePythonContractAndRoundTripsPlainEvidence()
  {
    var plugin = Capture.Plugin;
    await using var server = await PluginServer.StartAsync(plugin);
    var description = await server.Wire.DescribeAsync();
    var catalog = Suites.Parser.ParseFrom(description.Declarations.Sections.Single(item => item.Type == Suites.Descriptor.FullName).Payload);
    var suite = Assert.Single(catalog.Suites_);
    Assert.Equal("capture", suite.Metadata.Identifier);
    var template = Assert.Single(suite.Templates);
    Assert.Equal("capture", template.Metadata.Identifier);
    var contracts = RecordContracts.Parser.ParseFrom(description.Declarations.Sections.Single(item => item.Type == RecordContracts.Descriptor.FullName).Payload);
    var contract = Assert.Single(contracts.Records);
    Assert.Equal("capture.confirmation", contract.Metadata.Identifier);
    Assert.Equal("google.protobuf.BoolValue", contract.Message);
    var testCase = (await server.Wire.InvokeAsync<TestCasesMaterializeResponse>(description, template.Materialize,
        new TestCasesMaterializeRequest { Template = template.Metadata.Identifier })).TestCase;
    Assert.Equal("capture.confirm", Assert.Single(testCase.Steps).Metadata.Identifier);
    Assert.Equal("capture.confirmed", Assert.Single(testCase.Criteria).Metadata.Identifier);
    var origin = new Origin { PluginRun = new PluginRun { Plugin = description.Id, Template = template.Metadata.Identifier, TestCase = testCase } };
    Dokime.Clients.EvidenceResources.RecordsAppendRequest? stored = null;
    await server.Wire.InvokeAsync<StepsExecuteResponse>(description, testCase.Steps[0].Operation,
        new StepsExecuteRequest { Execution = "capture-run", Step = testCase.Steps[0].Metadata.Identifier, TestCase = testCase, Origin = origin }, "capture-run", child =>
        {
          if (child.Operation == "harmos.host.v1.Time/Get")
            return new Harmos.Clients.TimeGetResponse { Now = new Timestamp { Seconds = 120 } }.ToByteString();
          if (child.Operation == HostContracts.PromptsCreate.Contract.Id)
          {
            var question = Dokime.Clients.PromptsCreateRequest.Parser.ParseFrom(child.Payload);
            Assert.Equal(["0", "1"], question.Definition.Actions.Select(item => item.Identifier));
            Assert.Empty(question.PromptType);
            Assert.Equal("capture.question", question.Prompt);
            Assert.Equal("capture.question", question.Definition.Metadata.Identifier);
            return new Dokime.Clients.PromptsCreateResponse { Accepted = true, Action = "0" }.ToByteString();
          }
          Assert.Equal(HostContracts.EvidenceRecordsAppend.Contract.Id, child.Operation);
          stored = Dokime.Clients.EvidenceResources.RecordsAppendRequest.Parser.ParseFrom(child.Payload);
          return new Dokime.Clients.EvidenceResources.RecordsAppendResponse { Ordinal = 1 }.ToByteString();
        });
    Assert.Equal("capture.confirmation", stored!.Kind);
    Assert.True(BoolValue.Parser.ParseFrom(stored!.Payload).Value);
    var verdict = await server.Wire.InvokeAsync<CriteriaEvaluateResponse>(description, testCase.Criteria[0].Operation,
        new CriteriaEvaluateRequest { Execution = "capture-run", Criterion = testCase.Criteria[0].Metadata.Identifier, TestCase = testCase, Origin = origin }, "capture-run", child =>
        {
          var request = Dokime.Clients.EvidenceResources.RecordsListRequest.Parser.ParseFrom(child.Payload);
          Assert.Equal("capture.confirmation", request.RecordType);
          Assert.Equal("capture.confirm", request.Step);
          return new Dokime.Clients.EvidenceResources.RecordsListResponse
          {
            Last = new Dokime.Clients.EvidenceResources.CapturedRecord { Payload = stored!.Payload }
          }.ToByteString();
        });
    Assert.True(verdict.Passed);
  }

  [Fact]
  public void DuplicateAndEmptyIdentifiersFail()
  {
    Assert.Throws<ArgumentException>(() => new Plugin("invalid", "1.0.0", Dokime.Plugins.Suite.From<Repeated>()));
    Assert.Throws<ArgumentException>(() => new Plugin("invalid", "1.0.0", Dokime.Plugins.Suite.From<Empty>()));
    Assert.Throws<ArgumentException>(() => new Plugin("invalid", "1.0.0", Dokime.Plugins.Suite.From<CrossKind>()));
    var repeated = Dokime.Plugins.Suite.From<Capture>();
    Assert.Throws<ArgumentException>(() => new Plugin("invalid", "1.0.0", repeated, repeated).Build());
  }

  [Fact]
  public void ExplicitSuiteIdentifiersDoNotRenameGroupedTemplates()
  {
    var single = new Plugin("one", "1.0.0", Dokime.Plugins.Suite.From<RenamedSuite>()).Suites[0].Templates[0];
    var grouped = new Plugin("two", "1.0.0", new Dokime.Plugins.Suite("accuracy", "Accuracy", Template.From<RenamedSuite>())).Suites[0].Templates[0];
    Assert.Equal("chosen", single.Definition.Metadata.Identifier);
    Assert.Equal("chosen.read_value", single.Resolve((Func<Context, Task>)new RenamedSuite().ReadValue));
    Assert.Equal("accuracy.renamed", grouped.Definition.Metadata.Identifier);
  }

  [Fact]
  public async Task EmptyEvidenceAndRefusedPromptAnswersFail()
  {
    var plugin = Capture.Plugin;
    await using var server = await PluginServer.StartAsync(plugin);
    var description = await server.Wire.DescribeAsync();
    var template = plugin.Suites[0].Templates[0];
    var testCase = (await server.Wire.InvokeAsync<TestCasesMaterializeResponse>(description, "capture.materialize",
        new TestCasesMaterializeRequest { Template = "capture" })).TestCase;
    var origin = new Origin { PluginRun = new PluginRun { Plugin = description.Id, Template = "capture", TestCase = testCase } };
    var criterion = new CriteriaEvaluateRequest { Execution = "capture-run", Criterion = "capture.confirmed", TestCase = testCase, Origin = origin };
    await Assert.ThrowsAsync<Harmos.PluginFailureException>(() => server.Wire.InvokeAsync<CriteriaEvaluateResponse>(description,
        "capture.confirmed", criterion, "capture-run", _ => new Dokime.Clients.EvidenceResources.RecordsListResponse().ToByteString()));
    var step = new StepsExecuteRequest { Execution = "capture-run", Step = "capture.confirm", TestCase = testCase, Origin = origin };
    await Assert.ThrowsAsync<Harmos.PluginFailureException>(() => server.Wire.InvokeAsync<StepsExecuteResponse>(description,
        "capture.confirm", step, "capture-run", _ => new Dokime.Clients.PromptsCreateResponse { Accepted = false }.ToByteString()));
  }

  [Theory]
  [InlineData("ReadVoltage")]
  [InlineData("readVoltage")]
  [InlineData("read_voltage")]
  public void MemberWordBoundariesAgree(string name) => Assert.Equal("accuracy.voltage.read_voltage", Identifiers.Build(name, prefix: "accuracy.voltage"));

  [Fact]
  public void PlainAndStructuredPayloadsRoundTrip()
  {
    RoundTrip(true);
    RoundTrip(false);
    RoundTrip("plain value");
    RoundTrip(long.MinValue);
    RoundTrip(long.MaxValue);
    RoundTrip(int.MinValue);
    RoundTrip(2.125);
    RoundTrip(2.125f);
    RoundTrip(new BoolValue { Value = true });
    Assert.Throws<ArgumentException>(() => Payload<byte[]>.Pack([]));
  }

  [Fact]
  public async Task PlainOperationsUseTheSameWrappersAndLocalDispatch()
  {
    var text = new Dokime.Authoring.Operation<string, string>("values.text", Dokime.Authoring.Visibility.Public);
    var flag = new Dokime.Authoring.Operation<bool, bool>("values.flag", Dokime.Authoring.Visibility.Public);
    var integer = new Dokime.Authoring.Operation<long, long>("values.integer", Dokime.Authoring.Visibility.Public);
    var number = new Dokime.Authoring.Operation<double, double>("values.number", Dokime.Authoring.Visibility.Public);
    var local = new Dokime.Authoring.Operation<string, string>("values.local", Dokime.Authoring.Visibility.Private);
    var plugin = new Plugin("values", "1.0.0")
        .Operation(local, (_, value) => Task.FromResult(value + "!"))
        .Operation(text, (context, value) => context.CallAsync(local, value))
        .Operation(flag, (_, value) => Task.FromResult(!value))
        .Operation(integer, (_, value) => Task.FromResult(value))
        .Operation(number, (_, value) => Task.FromResult(value));
    await using var server = await PluginServer.StartAsync(plugin);
    var description = await server.Wire.DescribeAsync();
    Assert.DoesNotContain(description.Declarations.Operations, item => item.Id == "values.local");
    Assert.Equal("plain!", (await server.Wire.InvokeAsync<StringValue>(description, "values.text", new StringValue { Value = "plain" })).Value);
    Assert.True((await server.Wire.InvokeAsync<BoolValue>(description, "values.flag", new BoolValue { Value = false })).Value);
    Assert.Equal(long.MinValue, (await server.Wire.InvokeAsync<Int64Value>(description, "values.integer", new Int64Value { Value = long.MinValue })).Value);
    Assert.Equal(2.125, (await server.Wire.InvokeAsync<DoubleValue>(description, "values.number", new DoubleValue { Value = 2.125 })).Value);
  }

  [Fact]
  public async Task MaterializedInstancesUseQualifiedIdentifiersAndRegisteredHandlers()
  {
    var local = Template.From<HTTPAccuracySuite>().Materialize((testCase, _) =>
    {
      var first = testCase.Steps[0].Clone();
      first.Metadata.Identifier = "channel_1";
      var second = first.Clone();
      second.Metadata.Identifier = "channel_2";
      testCase.Steps.Clear();
      testCase.Steps.Add(first);
      testCase.Steps.Add(second);
      testCase.Criteria[0].Metadata.Identifier = "accept_1";
      return testCase;
    });
    var plugin = new Plugin("instances", "1.0.0", new Dokime.Plugins.Suite("accuracy", "Accuracy", local));
    await using var server = await PluginServer.StartAsync(plugin);
    var description = await server.Wire.DescribeAsync();
    var testCase = (await server.Wire.InvokeAsync<TestCasesMaterializeResponse>(description, "accuracy.http_accuracy.materialize",
        new TestCasesMaterializeRequest { Template = "accuracy.http_accuracy" })).TestCase;
    Assert.Equal(["accuracy.http_accuracy.channel_1", "accuracy.http_accuracy.channel_2"], testCase.Steps.Select(item => item.Metadata.Identifier));
    Assert.All(testCase.Steps, item => Assert.Equal("accuracy.http_accuracy.read_voltage", item.Operation));
    Assert.Equal("accuracy.http_accuracy.accept_1", testCase.Criteria[0].Metadata.Identifier);
  }

  private static void RoundTrip<T>(T value) => Assert.Equal(value, Payload<T>.Unpack(Payload<T>.Pack(value).ToByteString()));

  public sealed class HTTPAccuracySuite
  {
    public static readonly Record<long> Reading = new("Unrelated", identifier: "rms");
    public static readonly PromptDeclaration Question = new(new Prompt
    {
      Fields = { new PromptField { Metadata = new Metadata { Identifier = "reading" }, Required = true } },
      Actions = { new PromptAction { Identifier = "save", Label = "Save" } }
    });
    public static readonly Dokime.Plugins.Stream<BoolValue> Samples = new();
    public static readonly Observer<BoolValue> Observer = new(Samples, new Duration { Seconds = 1 }, _ => Task.FromResult(new BoolValue { Value = true }));

    [StepAttribute("Unrelated")]
    public Task ReadVoltage(Context context) => Task.CompletedTask;

    [CriterionAttribute("Unrelated")]
    public bool ReadingOK(Context context) => true;
  }

  [SuiteAttribute(Identifier = "chosen")]
  public sealed class RenamedSuite
  {
    [StepAttribute("Read")]
    public Task ReadValue(Context context) => Task.CompletedTask;
  }

  [SuiteAttribute]
  public sealed class Repeated
  {
    public static readonly Record<bool> First = new(identifier: "same");
    public static readonly Record<string> Second = new(identifier: "same");
  }

  [SuiteAttribute]
  public sealed class Empty
  {
    public static readonly Record<long> Reading = new(identifier: "");
  }

  [SuiteAttribute]
  public sealed class CrossKind
  {
    public static readonly Record<bool> Reading = new(identifier: "measure");

    [StepAttribute("Measure")]
    public Task Measure(Context context) => Task.CompletedTask;
  }
}
