// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.V1;
using Dokime.Protocol.Plugin.V1;
using Harmos.Sdk;
using Stories.Support;
using Xunit;

namespace Stories.Integration;

public sealed class AuthorRejectsMismatchedExecution
{
  [Fact]
  public async Task RefuseWrongScopeAndReleaseAfterCleanupFailure()
  {
    // Compose an Instrument That Must Be Released Even If Cleanup Fails.
    var instrument = new Instrument();
    var template = new Template("guard.case", "Guard")
        .Resource(_ => Task.FromResult<IAsyncDisposable>(instrument))
        .Step(new Dokime.Step("read", "Read", _ => { instrument.Reads++; return Task.CompletedTask; }))
        .Cleanup(_ => throw new InvalidOperationException("Fixture cleanup failed"));
    var plugin = new Dokime.Plugin("guard", "1.0.0", new Dokime.Suite("guards", "Guards", template));
    await using var server = await PluginServer.StartAsync(plugin);
    var description = await server.Wire.DescribeAsync();
    var testCase = (await server.Wire.InvokeAsync<TestCasesMaterializeResponse>(description, "guard.case.materialize", new TestCasesMaterializeRequest { Template = "guard.case" })).TestCase;
    var origin = new Origin { PluginRun = new PluginRun { Plugin = "guard", Template = "guard.case", TestCase = testCase } };
    var request = new StepsExecuteRequest { Execution = "execution-a", Step = "read", TestCase = testCase, Origin = origin };

    // Reject a Payload That Selects a Different Execution Than the Parent Scope.
    await Assert.ThrowsAsync<PluginFailureException>(() => server.Wire.InvokeAsync<StepsExecuteResponse>(description,
        testCase.Steps[0].Operation, request, "execution-b"));
    Assert.Equal(0, instrument.Reads);

    // Run the Matching Case and Surface Cleanup Failure After Releasing Ownership.
    await server.Wire.InvokeAsync<StepsExecuteResponse>(description, testCase.Steps[0].Operation, request, "execution-a");
    Assert.Equal(1, instrument.Reads);
    await Assert.ThrowsAsync<PluginFailureException>(() => server.Wire.InvokeAsync<ExecutionsCleanupResponse>(description, "guard.case.cleanup",
        new ExecutionsCleanupRequest { Execution = "execution-a", TestCase = testCase, Origin = origin }, "execution-a"));
    Assert.Equal(1, instrument.Disposals);
  }

  private sealed class Instrument : IAsyncDisposable
  {
    internal int Reads;
    internal int Disposals;
    public ValueTask DisposeAsync() { Disposals++; return ValueTask.CompletedTask; }
  }
}
