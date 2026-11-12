// SPDX-License-Identifier: Apache-2.0

using Grpc.Net.Client;

namespace Stories.Support;

internal sealed class PluginServer : IAsyncDisposable
{
  private readonly Task serving;
  private readonly CancellationTokenSource stop;
  private readonly GrpcChannel channel;
  internal string Address { get; }
  internal Wire Wire { get; }

  private PluginServer(Task serving, CancellationTokenSource stop, GrpcChannel channel, string address)
  {
    this.serving = serving;
    this.stop = stop;
    this.channel = channel;
    Address = address;
    Wire = new Wire(channel.CreateCallInvoker());
  }

  internal static async Task<PluginServer> StartAsync(Dokime.Plugin plugin)
  {
    // Capture the SDK's Readiness Signal, Without Polling or Scenario Delays.
    var original = Console.Out;
    var ready = new ReadinessWriter(original);
    Console.SetOut(ready);
    var stop = new CancellationTokenSource();
    var serving = plugin.ServeAsync(remote: true, cancellationToken: stop.Token);
    try
    {
      var winner = await Task.WhenAny(ready.Address.Task, serving);
      if (winner == serving) await serving;
      var address = "http://" + await ready.Address.Task;
      return new PluginServer(serving, stop, GrpcChannel.ForAddress(address), address);
    }
    finally { Console.SetOut(original); }
  }

  public async ValueTask DisposeAsync()
  {
    channel.Dispose();
    await stop.CancelAsync();
    await serving;
    stop.Dispose();
  }

  private sealed class ReadinessWriter(TextWriter original) : TextWriter
  {
    public override System.Text.Encoding Encoding => original.Encoding;
    internal TaskCompletionSource<string> Address { get; } = new(TaskCreationOptions.RunContinuationsAsynchronously);
    public override void WriteLine(string? value)
    {
      if (value?.StartsWith("harmos listening ", StringComparison.Ordinal) == true)
        Address.TrySetResult(value["harmos listening ".Length..]);
      original.WriteLine(value);
    }
  }
}
