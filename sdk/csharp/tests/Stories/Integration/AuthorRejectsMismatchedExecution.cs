// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Plugins;
using Harmos;
using Stories.Support;
using Xunit;

namespace Stories.Integration;

public sealed class AuthorRejectsMismatchedExecution
{
  [Fact]
  public async Task RefuseWrongScopeAndReleaseAfterCleanupFailure()
  {
    var plugin = new ExamplePlugin { FailCleanup = true };
    await using var server = await PluginServer.StartAsync(plugin);
    var description = await server.Wire.DescribeAsync();
    var materialize = new TestCasesMaterializeRequest { Template = "example.capture.confirmation" };
    var testCase = (await server.Wire.InvokeAsync<TestCasesMaterializeResponse>(description, "example.capture.confirmation.materialize", materialize)).TestCase;
    var origin = new Origin { PluginRun = new PluginRun { Plugin = description.Id, Template = materialize.Template, TestCase = testCase } };
    var step = new StepsExecuteRequest { Execution = "execution-a", Step = testCase.Steps[0].Metadata.Identifier, TestCase = testCase, Origin = origin };
    await Assert.ThrowsAsync<OperationException>(() => server.Wire.InvokeAsync<StepsExecuteResponse>(description, testCase.Steps[0].Operation, step, "execution-b"));
    Assert.Equal(0, plugin.Reads);
    await server.Wire.InvokeAsync<ExecutionsInitializeResponse>(description, "example.capture.confirmation.initialize",
      new ExecutionsInitializeRequest { Execution = "execution-a", TestCase = testCase, Origin = origin }, "execution-a");
    await Assert.ThrowsAsync<OperationException>(() => server.Wire.InvokeAsync<ExecutionsCleanupResponse>(description, "example.capture.confirmation.cleanup",
      new ExecutionsCleanupRequest { Execution = "execution-a", TestCase = testCase, Origin = origin }, "execution-a"));
    Assert.Equal(1, plugin.Cleanups);
    Assert.Equal(1, plugin.Disposals);
  }
}
