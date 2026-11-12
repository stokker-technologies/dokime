// SPDX-License-Identifier: Apache-2.0

using Dokime.V1;
using Grpc.Core;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.TestHost;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace Stories.Support;

[BindServiceMethod(typeof(FakeBackend), nameof(BindService))]
public class FakeBackend
{
  internal static readonly Method<GetBackendIdentity, BackendIdentity> Identity =
      Wire.Method<GetBackendIdentity, BackendIdentity>(MethodType.Unary, "dokime.v1.Backend", "GetIdentity");
  internal static readonly Method<WatchBackendLogs, LogFrame> Logs =
      Wire.Method<WatchBackendLogs, LogFrame>(MethodType.ServerStreaming, "dokime.v1.Backend", "WatchLogs");

  public static void BindService(ServiceBinderBase binder, FakeBackend service)
  {
    binder.AddMethod(Identity, service is null ? null : service.GetIdentity);
    binder.AddMethod(Logs, service is null ? null : service.WatchLogs);
  }

  public virtual Task<BackendIdentity> GetIdentity(GetBackendIdentity request, ServerCallContext context)
  {
    if (context.RequestHeaders.GetValue("authorization") != "Bearer test-token")
      throw new RpcException(new Status(StatusCode.Unauthenticated, "Missing token"));
    return Task.FromResult(new BackendIdentity { BackendId = "bench", Epoch = "boot-a", ProductVersion = "1.0.0" });
  }

  public virtual async Task WatchLogs(WatchBackendLogs request, IServerStreamWriter<LogFrame> response, ServerCallContext context)
  {
    await response.WriteAsync(new LogFrame { Level = Dokime.V1.LogLevel.Info, Message = "ready" });
    await response.WriteAsync(new LogFrame { Level = Dokime.V1.LogLevel.Error, Message = "fixture failure" });
  }

  internal static async Task<(WebApplication App, HttpMessageHandler Handler)> StartAsync()
  {
    var builder = WebApplication.CreateSlimBuilder();
    builder.Logging.ClearProviders();
    builder.WebHost.UseTestServer();
    builder.Services.AddGrpc();
    var app = builder.Build();
    app.MapGrpcService<FakeBackend>();
    await app.StartAsync();
    var handler = app.GetTestServer().CreateHandler();
    return (app, handler);
  }
}
