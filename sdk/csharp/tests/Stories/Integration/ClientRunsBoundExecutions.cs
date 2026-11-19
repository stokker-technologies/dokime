// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Stories.Support;
using Xunit;

namespace Stories.Integration;

public sealed class ClientRunsBoundExecutions
{
  [Fact]
  public async Task RunObservesAutomaticSealAndSealReturnsStableOutcome()
  {
    var backend = new FakeBackend();
    var client = Connect(backend);
    var cycle = await client.CycleAsync("Capture");
    var testCase = await client.Cases.GetAsync("case");
    var execution = await cycle.ExecutionAsync(testCase, requestId: "retained");
    var progress = new List<Progress>();
    await foreach (var frame in execution.RunAsync())
      progress.Add(frame);
    Assert.Equal(2, progress.Count);
    Assert.False(progress[0].Sealed);
    Assert.True(progress[1].Sealed);
    Assert.Equal(ResultStatus.Passed, (await execution.SealAsync()).Status);
    Assert.Equal(["/dokime.host.v1.Executions/Start", "/dokime.host.v1.executions.Progress/Watch", "/dokime.host.v1.Executions/Seal"], backend.Calls.TakeLast(3));
    await cycle.ExecutionAsync(testCase, requestId: "retained");
    Assert.Equal(backend.Bindings[0], backend.Bindings[1]);
    await client.CloseAsync();
  }

  [Fact]
  public async Task MissingSealedFrameAndChangedBindingAreRefused()
  {
    var backend = new FakeBackend { EndBeforeSeal = true };
    var client = Connect(backend);
    var cycle = await client.CycleAsync("Capture");
    var testCase = await client.Cases.GetAsync("case");
    var execution = await cycle.ExecutionAsync(testCase);
    await Assert.ThrowsAsync<InvalidDataException>(async () =>
    {
      await foreach (var frame in execution.RunAsync())
        _ = frame;
    });
    backend.ChangeBindingOnStart = true;
    await Assert.ThrowsAsync<InvalidDataException>(() => execution.StartAsync());
    await client.CloseAsync();
  }

  [Fact]
  public async Task RegisterBindsTemplateAndCloseUnregistersOwnedEndpoint()
  {
    var backend = new FakeBackend();
    var client = Connect(backend);
    var registered = await client.Plugins.RegisterAsync(new ExamplePlugin());
    Assert.Equal("Ready", registered.State);
    Assert.Equal(1, backend.RegisteredCount);
    Assert.Single(client.Plugins.Bindings);
    await client.CloseAsync();
    Assert.Equal(0, backend.RegisteredCount);
    Assert.Empty(client.Plugins.Bindings);
  }

  private static Client Connect(FakeBackend backend) => new(backend, headers: new Grpc.Core.Metadata { { "authorization", "Bearer test-token" } });
}
