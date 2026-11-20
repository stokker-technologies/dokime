// SPDX-License-Identifier: Apache-2.0

using System.Diagnostics;
using Dokime;
using Dokime.Plugins;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Dokime.Clients.EvidenceResources;
using Grpc.Net.Client;
using Harmos;
using Harmos.Clients;
using Stories.Support;
using Xunit;

namespace Stories.Integration;

public sealed class AuthorRunsThePackagedStarter
{
  [Fact]
  [Trait("Layer", "Starter")]
  public async Task StartAtTheHostEndpointRunAndStop()
  {
    // Start the Package-Consuming Executable at the Host-Assigned Endpoint.
    var start = ReferenceHost.PluginStart(ReferenceHost.Artifact("DOKIME_STARTER"));
    using var process = Process.Start(start)!;
    var errors = process.StandardError.ReadToEndAsync();
    try
    {
      using var setup = new CancellationTokenSource(TimeSpan.FromSeconds(30));
      var readiness = await process.StandardOutput.ReadLineAsync(setup.Token);
      Assert.NotNull(readiness);
      var output = process.StandardOutput.ReadToEndAsync();
      using var channel = GrpcChannel.ForAddress(readiness!);
      var wire = new Wire(channel.CreateCallInvoker());

      // Discover and Materialize the Starter's Authored Template.
      var description = await wire.DescribeAsync();
      var section = Suites.Parser.ParseFrom(description.Declarations.Sections.Single(item => item.Type == Suites.Descriptor.FullName).Payload);
      var template = Assert.Single(Assert.Single(section.Suites_).Templates);
      var testCase = (await wire.InvokeAsync<TestCasesMaterializeResponse>(description, template.Materialize,
          new TestCasesMaterializeRequest { Template = template.Metadata.Identifier })).TestCase;
      var step = Assert.Single(testCase.Steps);
      var origin = new Origin { PluginRun = new PluginRun { Plugin = description.Id, Template = template.Metadata.Identifier, TestCase = testCase } };

      // Answer the Operator and Acknowledge the Plain Boolean Record.
      RecordsAppendRequest? stored = null;
      await wire.InvokeAsync<StepsExecuteResponse>(description, step.Operation,
          new StepsExecuteRequest { Execution = "starter-execution", Step = step.Metadata.Identifier, TestCase = testCase, Origin = origin },
          "starter-execution", child =>
          {
            if (child.Operation == "harmos.plugin.v1.Time/Get")
              return new TimeGetResponse { Now = new Timestamp { Seconds = 120 } }.ToByteString();
            if (child.Operation == HostContracts.PromptsCreate.Contract.Id)
              return new Dokime.Clients.PromptsCreateResponse { Accepted = true, Action = "0" }.ToByteString();
            Assert.Equal(HostContracts.EvidenceRecordsAppend.Contract.Id, child.Operation);
            stored = RecordsAppendRequest.Parser.ParseFrom(child.Payload);
            return new RecordsAppendResponse { Ordinal = 1 }.ToByteString();
          });
      Assert.Equal("starter-execution", stored!.Execution);
      Assert.Equal("capture.confirmation.confirmation", stored.Kind);
      Assert.True(BoolValue.Parser.ParseFrom(stored.Payload).Value);

      // Evaluate the Captured Evidence Through Its Qualified Step Selector.
      var criterion = Assert.Single(testCase.Criteria);
      var verdict = await wire.InvokeAsync<CriteriaEvaluateResponse>(description, criterion.Operation,
          new CriteriaEvaluateRequest { Execution = "starter-execution", Criterion = criterion.Metadata.Identifier, TestCase = testCase, Origin = origin },
          "starter-execution", child =>
          {
            Assert.Equal(HostContracts.EvidenceRecordsList.Contract.Id, child.Operation);
            var request = RecordsListRequest.Parser.ParseFrom(child.Payload);
            Assert.Equal("capture.confirmation.confirm", request.Step);
            return new RecordsListResponse { Last = new CapturedRecord { Payload = stored.Payload } }.ToByteString();
          });
      Assert.True(verdict.Passed);

      // Clean Up and Send Stop; Managed Placement Must Exit Successfully.
      await wire.InvokeAsync<ExecutionsCleanupResponse>(description, template.Cleanup,
          new ExecutionsCleanupRequest { Execution = "starter-execution", TestCase = testCase, Origin = origin }, "starter-execution");
      await wire.StopAsync();
      process.StandardInput.Close();
      await process.WaitForExitAsync(setup.Token);
      Assert.True(process.ExitCode == 0, await errors);
      await output;
    }
    finally
    {
      if (!process.HasExited)
        process.Kill(entireProcessTree: true);
      await process.WaitForExitAsync();
      await errors;
    }
  }
}
