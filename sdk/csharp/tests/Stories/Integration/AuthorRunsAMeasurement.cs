// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Clients.EvidenceResources;
using Dokime.Plugins;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Stories.Support;
using Xunit;

namespace Stories.Integration;

public sealed class AuthorRunsAMeasurement
{
  [Fact]
  public async Task CapturePromptRecordReadCriterionAndDispose()
  {
    var plugin = new ExamplePlugin();
    await using var server = await PluginServer.StartAsync(plugin);
    var description = await server.Wire.DescribeAsync();
    var catalog = Suites.Parser.ParseFrom(description.Declarations.Sections.Single(item => item.Type == Suites.Descriptor.FullName).Payload);
    var template = Assert.Single(Assert.Single(catalog.Suites_).Templates);
    var testCase = (await server.Wire.InvokeAsync<TestCasesMaterializeResponse>(description, template.Materialize,
      new TestCasesMaterializeRequest { Template = template.Metadata.Identifier })).TestCase;
    var origin = new Origin { PluginRun = new PluginRun { Plugin = description.Id, Template = template.Metadata.Identifier, TestCase = testCase } };
    RecordsAppendRequest? stored = null;
    await server.Wire.InvokeAsync<StepsExecuteResponse>(description, testCase.Steps[0].Operation,
      new StepsExecuteRequest { Execution = "capture-run", Step = testCase.Steps[0].Metadata.Identifier, TestCase = testCase, Origin = origin }, "capture-run", child =>
      {
        if (child.Operation == "harmos.plugin.v1.Time/Get")
          return new Harmos.Clients.TimeGetResponse { Now = new Timestamp { Seconds = 120 } }.ToByteString();
        if (child.Operation == HostContracts.PromptsCreate.Contract.Id)
        {
          var request = Dokime.Clients.PromptsCreateRequest.Parser.ParseFrom(child.Payload);
          Assert.Equal(["Yes", "No"], request.Definition.Actions.Select(action => action.Label));
          Assert.Equal("capture.confirmation.question", request.Prompt);
          return new Dokime.Clients.PromptsCreateResponse { Accepted = true, Action = "0" }.ToByteString();
        }
        Assert.Equal(HostContracts.EvidenceRecordsAppend.Contract.Id, child.Operation);
        stored = RecordsAppendRequest.Parser.ParseFrom(child.Payload);
        return new RecordsAppendResponse { Ordinal = 1 }.ToByteString();
      });
    Assert.Equal("capture.confirmation.confirmation", stored!.Kind);
    Assert.True(BoolValue.Parser.ParseFrom(stored.Payload).Value);
    var outcome = await server.Wire.InvokeAsync<CriteriaEvaluateResponse>(description, testCase.Criteria[0].Operation,
      new CriteriaEvaluateRequest { Execution = "capture-run", Criterion = testCase.Criteria[0].Metadata.Identifier, TestCase = testCase, Origin = origin }, "capture-run", child =>
      {
        var request = RecordsListRequest.Parser.ParseFrom(child.Payload);
        Assert.Equal("capture.confirmation.confirm", request.Step);
        return new RecordsListResponse { Last = new CapturedRecord { Payload = stored.Payload } }.ToByteString();
      });
    Assert.True(outcome.Passed);
    await server.Wire.InvokeAsync<ExecutionsCleanupResponse>(description, template.Cleanup,
      new ExecutionsCleanupRequest { Execution = "capture-run", TestCase = testCase, Origin = origin, Termination = Termination.Completed }, "capture-run");
    Assert.Equal(1, plugin.Reads);
    Assert.Equal(1, plugin.Cleanups);
    Assert.Equal(1, plugin.Disposals);
  }
}
