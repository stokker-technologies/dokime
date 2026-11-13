// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.V1;
using Grpc.Core;
using Grpc.Net.Client;
using Stories.Support;
using Xunit;

namespace Stories.Integration;

public sealed class ClientInspectsABackend
{
  [Fact]
  public async Task AuthenticateInspectAndReadFiniteLogs()
  {
    // Connect to an In-Process Backend Through the SDK's Shared Call Invoker.
    var server = await FakeBackend.StartAsync();
    await using var app = server.App;
    await using var client = Client.Connect("http://localhost", new GrpcChannelOptions { HttpHandler = server.Handler });
    var plugins = client.Service(invoker => new Plugins.PluginsClient(invoker));

    // Observe the Transport's Authentication Failure Before Supplying a Token.
    using var denied = client.Status.Get(new Harmos.V1.StatusGetRequest());
    var failure = await Assert.ThrowsAsync<RpcException>(() => denied.ResponseAsync);
    Assert.Equal(StatusCode.Unauthenticated, failure.StatusCode);

    // Inspect the Backend Identity with Explicit Per-Call Credentials.
    var headers = new Grpc.Core.Metadata { { "authorization", "Bearer test-token" } };
    using var identity = client.Status.Get(new Harmos.V1.StatusGetRequest(), new CallOptions(headers));
    var observed = await identity.ResponseAsync;
    Assert.Equal("bench", observed.Status.Identity.HostId);
    Assert.True(observed.Status.Available);

    // Drain a Finite Server Stream and Observe Successful Completion.
    using var logs = client.Status.Logs.Watch(new Harmos.V1.StatusResources.LogsWatchRequest { MaxBytes = 4096, MaxEvents = 8 }, new CallOptions(headers));
    List<Harmos.V1.StatusResources.LogsWatchResponse> frames = [];
    await foreach (var frame in logs.ResponseStream.ReadAllAsync()) frames.Add(frame);
    Assert.Empty(frames[0].Events);
    Assert.Equal(new[] { "INFO", "ERROR" }, frames.SelectMany(frame => frame.Events).Select(item => item.Level));
    Assert.Equal(StatusCode.OK, logs.GetStatus().StatusCode);

    // Add a Remote Entry and Read It Back Through the Generated Plugin Client.
    using var add = plugins.AddAsync(new AddRemotePlugin { Url = "http://127.0.0.1:7001" }, headers);
    var added = await add.ResponseAsync;
    Assert.Equal("http://127.0.0.1:7001", added.Remote.Url);
    Assert.Equal(PluginState.Connecting, added.State);
    using var list = plugins.ListAsync(new ListPlugins(), headers);
    var listed = await list.ResponseAsync;
    Assert.Equal(added, Assert.Single(listed.Plugins));

    // Preserve the Server's Refusal Through a Generated Unary Call.
    using var invalid = plugins.AddAsync(new AddRemotePlugin { Url = "invalid" }, headers);
    var rejected = await Assert.ThrowsAsync<RpcException>(() => invalid.ResponseAsync);
    Assert.Equal(StatusCode.InvalidArgument, rejected.StatusCode);
  }
}
