// SPDX-License-Identifier: Apache-2.0

using Dokime.V1;
using Grpc.Core;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.TestHost;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace Stories.Support;

public class FakeBackend : Backend.BackendBase
{
  public override Task<BackendIdentity> GetIdentity(GetBackendIdentity request, ServerCallContext context)
  {
    if (context.RequestHeaders.GetValue("authorization") != "Bearer test-token")
      throw new RpcException(new Status(StatusCode.Unauthenticated, "Missing token"));
    return Task.FromResult(new BackendIdentity { BackendId = "bench", Epoch = "boot-a", ProductVersion = "1.0.0" });
  }

  public override async Task WatchLogs(WatchBackendLogs request, IServerStreamWriter<BackendWatchLogsOutput> response, ServerCallContext context)
  {
    await response.WriteAsync(new BackendWatchLogsOutput { Item = new LogFrame { Level = Dokime.V1.LogLevel.Info, Message = "ready" } });
    await response.WriteAsync(new BackendWatchLogsOutput { Item = new LogFrame { Level = Dokime.V1.LogLevel.Error, Message = "fixture failure" } });
    await response.WriteAsync(new BackendWatchLogsOutput { Response = new BackendWatchLogsCompletion() });
  }

  internal static async Task<(WebApplication App, HttpMessageHandler Handler)> StartAsync()
  {
    var builder = WebApplication.CreateSlimBuilder();
    builder.Logging.ClearProviders();
    builder.WebHost.UseTestServer();
    builder.Services.AddGrpc();
    builder.Services.AddSingleton<FakePlugins>();
    var app = builder.Build();
    app.MapGrpcService<FakeBackend>();
    app.MapGrpcService<FakePlugins>();
    await app.StartAsync();
    var handler = app.GetTestServer().CreateHandler();
    return (app, handler);
  }
}

public sealed class FakePlugins : Plugins.PluginsBase
{
  private readonly System.Collections.Concurrent.ConcurrentDictionary<string, Dokime.V1.Plugin> entries = new();

  public override Task<Dokime.V1.Plugin> Add(AddRemotePlugin request, ServerCallContext context)
  {
    if (!Uri.TryCreate(request.Url, UriKind.Absolute, out var url) || url.Scheme != "http")
      throw new RpcException(new Status(StatusCode.InvalidArgument, "Plugin URL is required"));
    var plugin = new Dokime.V1.Plugin
    {
      Entry = Guid.NewGuid().ToString(),
      Remote = new RemotePlugin { Url = request.Url },
      State = PluginState.Connecting
    };
    entries[plugin.Entry] = plugin;
    return Task.FromResult(plugin.Clone());
  }

  public override Task<PluginEntries> List(ListPlugins request, ServerCallContext context)
  {
    var reply = new PluginEntries();
    reply.Plugins.AddRange(entries.Values.OrderBy(entry => entry.Entry).Select(entry => entry.Clone()));
    return Task.FromResult(reply);
  }
}
