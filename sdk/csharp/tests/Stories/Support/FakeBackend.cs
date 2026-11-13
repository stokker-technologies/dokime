// SPDX-License-Identifier: Apache-2.0

using Dokime.Protocol.Client.V1;
using Dokime.Protocol.Client.V1.ExecutionsResources;
using Google.Protobuf;
using Google.Protobuf.Reflection;
using System.Security.Cryptography;
using Dokime.Protocol.V1;
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
    builder.Services.AddSingleton<FakeSchemas>();
    var app = builder.Build();
    app.MapGrpcService<FakeBackend>();
    app.MapGrpcService<FakeLogs>();
    app.MapGrpcService<FakePlugins>();
    app.MapGrpcService<FakeCases>();
    app.MapGrpcService<FakeSchemas>();
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
  private readonly System.Collections.Concurrent.ConcurrentDictionary<string, Dokime.Protocol.Client.V1.Plugin> entries = new();

  public override Task<Dokime.Protocol.Client.V1.Plugin> Add(AddRemotePlugin request, ServerCallContext context)
  {
    if (!Uri.TryCreate(request.Url, UriKind.Absolute, out var url) || url.Scheme != "http")
      throw new RpcException(new Status(StatusCode.InvalidArgument, "Plugin URL is required"));
    var plugin = new Dokime.Protocol.Client.V1.Plugin
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

public sealed class FakeCases : Dokime.Protocol.Client.V1.Cases.CasesBase
{
  public override Task<Dokime.Protocol.Client.V1.CasesGetResponse> Get(Dokime.Protocol.Client.V1.CasesGetRequest request, ServerCallContext context)
  {
    if (context.RequestHeaders.GetValue("authorization") != "Bearer test-token")
      throw new RpcException(new Status(StatusCode.Unauthenticated, "Get Case: Supply Credentials"));
    if (string.IsNullOrWhiteSpace(request.Name))
      throw new RpcException(new Status(StatusCode.InvalidArgument, "Get Case: Supply a Name"));
    return Task.FromResult(new Dokime.Protocol.Client.V1.CasesGetResponse
    {
      Value = new CaseDetail { Summary = new CaseSummary { CaseId = request.Name, Status = CaseStatus.Planned } },
      ResolvedAt = request.At?.Clone()
    });
  }
}

public sealed class FakeSchemas : Schemas.SchemasBase
{
  public FileDescriptorSet Types { get; } = new()
  {
    File = { Google.Protobuf.WellKnownTypes.BoolValue.Descriptor.File.ToProto(), Google.Protobuf.WellKnownTypes.Empty.Descriptor.File.ToProto() }
  };
  public int Calls { get; private set; }
  public ByteString Digest => ByteString.CopyFrom(SHA256.HashData(Types.ToByteArray()));

  public override Task<SchemasGetResponse> Get(SchemasGetRequest request, ServerCallContext context)
  {
    if (context.RequestHeaders.GetValue("authorization") != "Bearer test-token")
      throw new RpcException(new Status(StatusCode.Unauthenticated, "Get Schemas: Supply Credentials"));
    Calls++;
    var cut = new Harmos.V1.Position { History = new Harmos.V1.History { Store = "schemas", Epoch = request.Name }, Sequence = 1 };
    var first = request.Page?.PageToken.Length == 0;
    if (!first && (!cut.Equals(request.At) || request.Page?.PageToken != "second"))
      throw new RpcException(new Status(StatusCode.InvalidArgument, "Get Schemas: Continue the Same Read Cut"));
    return Task.FromResult(new SchemasGetResponse
    {
      Digest = Digest,
      Types_ = new FileDescriptorSet { File = { Types.File[first ? 0 : 1].Clone() } },
      ResolvedAt = cut,
      NextPageToken = first ? "second" : ""
    });
  }
}
