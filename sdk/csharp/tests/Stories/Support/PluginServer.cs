// SPDX-License-Identifier: Apache-2.0

using Grpc.Net.Client;
using Harmos.Plugins;

namespace Stories.Support;

internal sealed class PluginServer : IAsyncDisposable
{
  private readonly Listener listener;
  private readonly GrpcChannel channel;
  internal string Address
  {
    get;
  }
  internal Wire Wire
  {
    get;
  }
  private PluginServer(Listener listener)
  {
    this.listener = listener;
    Address = "http://" + listener.Address;
    channel = GrpcChannel.ForAddress(Address);
    Wire = new Wire(channel.CreateCallInvoker());
  }
  internal static async Task<PluginServer> StartAsync(Dokime.Plugin plugin)
  {
    var listener = await plugin.Adapter.Build().StartAsync("127.0.0.1:0");
    return new PluginServer(listener);
  }
  public async ValueTask DisposeAsync()
  {
    channel.Dispose();
    await listener.DisposeAsync();
  }
}
