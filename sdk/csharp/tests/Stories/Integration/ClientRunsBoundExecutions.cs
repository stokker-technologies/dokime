// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Stories.Support;
using Xunit;

using Harmos;

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

  [Theory]
  [InlineData(false)]
  [InlineData(true)]
  public async Task RegisterBindsTemplateAndCloseKeepsTheEntry(bool connectOnWatch)
  {
    // Register and Observe Readiness.
    var backend = new FakeBackend { ConnectOnWatch = connectOnWatch };
    var client = Connect(backend);
    var registered = await client.Plugins.RegisterAsync(new ExamplePlugin());
    Assert.Equal("Ready", registered.State);
    Assert.Equal(1, backend.RegisteredCount);
    Assert.Single(client.Plugins.Bindings);

    // Refresh and List the Bound Entry.
    Assert.Equal(registered.Identifier, (await client.Plugins.GetAsync(registered.Identifier)).Identifier);
    Assert.Same(registered, await registered.RefreshAsync());
    var listed = new List<RegisteredPlugin>();
    await foreach (var entry in client.Plugins.ListAsync())
      listed.Add(entry);
    Assert.Equal(registered.Identifier, Assert.Single(listed).Identifier);

    // Close the Client; the Host Keeps the Entry for the Next Start.
    await client.CloseAsync();
    Assert.Equal(1, backend.RegisteredCount);
    Assert.Empty(client.Plugins.Bindings);

    // Starting Again Reuses the Entry Under the Same Name.
    var restarted = Connect(backend);
    var replacement = await restarted.Plugins.RegisterAsync(new ExamplePlugin());
    Assert.Equal(registered.Identifier, replacement.Identifier);
    var stopped = replacement.WaitAsync(CancellationToken.None);
    Assert.False(stopped.IsCompleted);
    await replacement.DisposeAsync();
    await stopped;
    await replacement.DisposeAsync();
    await restarted.CloseAsync();
    Assert.Equal(1, backend.RegisteredCount);

    // Verify Product Registration Routes Without Teardown.
    var registrations = backend.Requests.OfType<Dokime.Clients.PluginsCreateRequest>().ToArray();
    Assert.Equal(2, registrations.Length);
    Assert.All(registrations, registration =>
    {
      Assert.True(registration.Value.Enabled);
      Assert.False(registration.HasOrigin);
      Assert.Equal(registered.Identifier, registration.Name);
      Assert.True(registration.HasSdkVersion);
      Assert.NotEmpty(registration.SdkVersion);
    });
    Assert.Equal(connectOnWatch ? 2 : 0, backend.Calls.Count(method => method == "/dokime.host.v1.Plugins/Watch"));
    Assert.DoesNotContain("/dokime.host.v1.Plugins/Delete", backend.Calls);
  }

  [Fact]
  public async Task DisposingARegistrationKeepsItsEntryUntilExplicitRemoval()
  {
    var backend = new FakeBackend();
    var client = Connect(backend);
    var registered = await client.Plugins.RegisterAsync(new ExamplePlugin());
    await registered.DisposeAsync();
    Assert.Equal(1, backend.RegisteredCount);
    Assert.DoesNotContain("/dokime.host.v1.Plugins/Delete", backend.Calls);
    await registered.UnregisterAsync();
    Assert.Equal(0, backend.RegisteredCount);
    await client.CloseAsync();
  }

  [Fact]
  public async Task RefusedRegistrationPreservesTheHostMessageAndKeepsExistingEntries()
  {
    var backend = new FakeBackend();
    var first = Connect(backend);
    await first.Plugins.RegisterAsync(new ExamplePlugin());
    await first.CloseAsync();
    backend.RegistrationRefusal = "fixture-sdk-refusal";
    var client = Connect(backend);
    var failure = await Assert.ThrowsAsync<OperationException>(() => client.Plugins.RegisterAsync(new ExamplePlugin()));
    Assert.Contains(backend.RegistrationRefusal, failure.Message);
    Assert.Empty(client.Plugins.Owned);
    Assert.Equal(1, backend.RegisteredCount);
    Assert.DoesNotContain("/dokime.host.v1.Plugins/Delete", backend.Calls);
    await client.CloseAsync();
  }

  private static Client Connect(FakeBackend backend) => new(backend, headers: new Grpc.Core.Metadata { { "authorization", "Bearer test-token" } });
}
