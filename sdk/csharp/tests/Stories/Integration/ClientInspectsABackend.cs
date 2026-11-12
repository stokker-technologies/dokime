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
    var calls = client.Service(invoker => invoker);

    // Observe the Transport's Authentication Failure Before Supplying a Token.
    using var denied = calls.AsyncUnaryCall(FakeBackend.Identity, null, new CallOptions(), new GetBackendIdentity());
    var failure = await Assert.ThrowsAsync<RpcException>(() => denied.ResponseAsync);
    Assert.Equal(StatusCode.Unauthenticated, failure.StatusCode);

    // Inspect the Backend Identity with Explicit Per-Call Credentials.
    var headers = new Grpc.Core.Metadata { { "authorization", "Bearer test-token" } };
    using var identity = calls.AsyncUnaryCall(FakeBackend.Identity, null, new CallOptions(headers), new GetBackendIdentity());
    var observed = await identity.ResponseAsync;
    Assert.Equal("bench", observed.BackendId);
    Assert.Equal("boot-a", observed.Epoch);

    // Drain a Finite Server Stream and Observe Successful Completion.
    using var logs = calls.AsyncServerStreamingCall(FakeBackend.Logs, null, new CallOptions(headers), new WatchBackendLogs());
    List<LogFrame> frames = [];
    await foreach (var frame in logs.ResponseStream.ReadAllAsync()) frames.Add(frame);
    Assert.Equal(new[] { LogLevel.Info, LogLevel.Error }, frames.Select(frame => frame.Level));
    Assert.Equal(StatusCode.OK, logs.GetStatus().StatusCode);
  }
}
