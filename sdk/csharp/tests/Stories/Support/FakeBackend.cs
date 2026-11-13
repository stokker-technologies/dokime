// SPDX-License-Identifier: Apache-2.0

using Dokime.V1;
using Grpc.Core;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.TestHost;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace Stories.Support;

public class FakeBackend : Harmos.V1.Status.StatusBase
{
  public override Task<Harmos.V1.StatusGetResponse> Get(Harmos.V1.StatusGetRequest request, ServerCallContext context)
  {
    if (context.RequestHeaders.GetValue("authorization") != "Bearer test-token")
      throw new RpcException(new Status(StatusCode.Unauthenticated, "Read Status: Supply Credentials"));
    return Task.FromResult(new Harmos.V1.StatusGetResponse
    {
      Status = new Harmos.V1.HostStatus
      {
        Identity = new Harmos.V1.HostIdentity { HostId = "bench", ApplicationVersion = "1.0.0" },
        Available = true
      }
    });
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
    app.MapGrpcService<FakeLogs>();
    app.MapGrpcService<FakePlugins>();
    app.MapGrpcService<FakeCases>();
    await app.StartAsync();
    var handler = app.GetTestServer().CreateHandler();
    return (app, handler);
  }
}

public sealed class FakeLogs : Harmos.V1.StatusResources.Logs.LogsBase
{
  public override async Task Watch(Harmos.V1.StatusResources.LogsWatchRequest request,
      IServerStreamWriter<Harmos.V1.StatusResources.LogsWatchResponse> response, ServerCallContext context)
  {
    if (context.RequestHeaders.GetValue("authorization") != "Bearer test-token")
      throw new RpcException(new Status(StatusCode.Unauthenticated, "Watch Logs: Supply Credentials"));
    await response.WriteAsync(new Harmos.V1.StatusResources.LogsWatchResponse());
    var batch = new Harmos.V1.StatusResources.LogsWatchResponse();
    batch.Events.Add(new Harmos.V1.StatusResources.Event { Level = "INFO", Message = "ready" });
    batch.Events.Add(new Harmos.V1.StatusResources.Event { Level = "ERROR", Message = "fixture failure" });
    await response.WriteAsync(batch);
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

public sealed class FakeCases : Dokime.Protocol.V1.Cases.CasesBase
{
  public override Task<Dokime.Protocol.V1.CasesGetResponse> Get(Dokime.Protocol.V1.CasesGetRequest request, ServerCallContext context)
  {
    if (context.RequestHeaders.GetValue("authorization") != "Bearer test-token")
      throw new RpcException(new Status(StatusCode.Unauthenticated, "Get Case: Supply Credentials"));
    if (string.IsNullOrWhiteSpace(request.Name))
      throw new RpcException(new Status(StatusCode.InvalidArgument, "Get Case: Supply a Name"));
    return Task.FromResult(new Dokime.Protocol.V1.CasesGetResponse
    {
      Value = new CaseDetail { Summary = new CaseSummary { CaseId = request.Name, Status = CaseStatus.Planned } },
      ResolvedAt = request.At?.Clone()
    });
  }
}
