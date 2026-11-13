// SPDX-License-Identifier: Apache-2.0

using System.Diagnostics;
using Dokime.V1;
using Google.Protobuf;
using Grpc.Core;
using Grpc.Net.Client;
using Harmos.V1;
using Harmos.Plugin.V1;
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
    var start = new ProcessStartInfo(ReferenceHost.Artifact("DOKIME_STARTER"))
    {
      RedirectStandardOutput = true,
      RedirectStandardError = true
    };
    start.Environment["HARMOS_ADDRESS"] = "127.0.0.1:0";
    using var process = Process.Start(start)!;
    var errors = process.StandardError.ReadToEndAsync();
    try
    {
      using var setup = new CancellationTokenSource(TimeSpan.FromSeconds(30));
      var readiness = await process.StandardOutput.ReadLineAsync(setup.Token);
      Assert.StartsWith("harmos listening ", readiness);
      var output = process.StandardOutput.ReadToEndAsync();
      using var channel = GrpcChannel.ForAddress("http://" + readiness!["harmos listening ".Length..]);
      var wire = new Wire(channel.CreateCallInvoker());

      // Discover and Materialize the Starter's Authored Template.
      var description = await wire.DescribeAsync();
      var section = Suites.Parser.ParseFrom(description.Catalog.Sections.Single(item => item.Type == "dokime.v1.Suites").Payload);
      var template = Assert.Single(Assert.Single(section.Suites_).Templates);
      var testCase = await wire.InvokeAsync<TestCase>(description, template.Materialize,
          new MaterializeRequest { Template = template.Metadata.Identifier });
      var step = Assert.Single(testCase.Steps);
      var origin = new Origin { PluginRun = new PluginRun { Plugin = description.Id, Template = template.Metadata.Identifier, TestCase = testCase } };

      // Execute the Measurement and Acknowledge Its Host Log Call.
      global::Dokime.Protocol.Context.V1.Evidence.LogsAppendRequest? log = null;
      await wire.InvokeAsync<StepResponse>(description, step.Operation,
          new StepRequest { Execution = "starter-execution", Step = step.Metadata.Identifier, TestCase = testCase, Origin = origin },
          "starter-execution", child =>
          {
            Assert.Equal(Dokime.Host.Log.Contract.Id, child.Operation);
            log = global::Dokime.Protocol.Context.V1.Evidence.LogsAppendRequest.Parser.ParseFrom(child.Payload);
            return new global::Dokime.Protocol.Context.V1.Evidence.LogsAppendResponse().ToByteString();
          });
      Assert.Equal("starter-execution", log!.Execution);
      Assert.Equal(Severity.Info, log.Severity);

      // Clean Up and Send Stop; Managed Placement Must Exit Successfully.
      await wire.InvokeAsync<CleanupResponse>(description, template.Cleanup,
          new CleanupRequest { Execution = "starter-execution", TestCase = testCase, Origin = origin }, "starter-execution");
      using var stopped = channel.CreateCallInvoker().AsyncUnaryCall(
          Wire.Method<InstanceStopRequest, InstanceStopResponse>(MethodType.Unary, "harmos.plugin.v1.Instance", "Stop"), null, new CallOptions(), new InstanceStopRequest());
      await stopped.ResponseAsync;
      await process.WaitForExitAsync(setup.Token);
      Assert.True(process.ExitCode == 0, await errors);
      await output;
    }
    finally
    {
      if (!process.HasExited) process.Kill(entireProcessTree: true);
      await process.WaitForExitAsync();
      await errors;
    }
  }
}
