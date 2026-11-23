// SPDX-License-Identifier: Apache-2.0

using Dokime.Operations;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Harmos;
using Stories.Support;
using Xunit;

namespace Stories.Integration;

public sealed class ActivationFacilities
{
  public sealed class ConnectionSettings : Settings
  {
    public string Address => Read<string>("address");
    public override IReadOnlyList<SettingDeclaration> Declarations => [new()
    {
      Name = "address", Title = "Address", Default = new SettingValue { Text = "first" }, Required = true,
      Choices = { new SettingValue { Text = "first" }, new SettingValue { Text = "second" } }
    }];
  }

  public sealed class Connection : Resource
  {
    internal FacilityPlugin Owner { get; private set; } = null!;
    public override Task OpenAsync(Context context)
    {
      Owner = (FacilityPlugin)context.Plugin;
      Owner.Opened.Add(context.Settings<ConnectionSettings>().Address);
      return Task.CompletedTask;
    }
    public override Task CloseAsync()
    {
      Owner.Closed++;
      return Task.CompletedTask;
    }
  }

  public sealed class Disconnected : Resource
  {
    public override Task OpenAsync(Context context) => throw new IOException("Device Is Disconnected");
    public override Task CloseAsync() => throw new InvalidOperationException("Close Resource: Failed Opens Own Their Cleanup");
  }

  public sealed class FacilityPlugin : Dokime.Plugin
  {
    public override string Version => "1.0.0";
    public override IReadOnlyList<System.Type> Settings => [typeof(ConnectionSettings)];
    public override IReadOnlyList<System.Type> Resources => [typeof(Connection), typeof(Disconnected)];
    internal List<string> Opened { get; } = [];
    internal int Closed
    {
      get; set;
    }
    internal List<ConnectionSettings> Snapshots { get; } = [];
    public override async Task InitializeAsync(Context context, CancellationToken cancellationToken = default)
    {
      Snapshots.Add(context.Settings<ConnectionSettings>());
      var first = await context.Resource<Connection>();
      var second = await context.Resource<Connection>();
      Assert.Same(first, second);
    }
    public override Func<Exception, OperationException?> Failure => error => error is IOException
      ? new OperationException(new Failure { Classification = Classification.Transient, Message = error.Message, Action = "Reconnect the Device" })
      : null;
  }

  [Fact]
  public async Task SettingsAndResourcesBelongToOneActivation()
  {
    var plugin = new FacilityPlugin();
    await using var server = await PluginServer.StartAsync(plugin);
    var description = await server.Wire.DescribeAsync();
    foreach (var address in new[] { "first", "second" })
    {
      await server.Wire.InvokeAsync<Empty>(description, $"{plugin.Identifier}.initialize", new Empty(), child: call =>
      {
        Assert.Equal("harmos.plugin.v1.plugins.Settings/List", call.Operation);
        return new Harmos.Clients.SettingsListResponse { Value = new SettingValues { Values = { ["address"] = new SettingValue { Text = address } } } }.ToByteString();
      });
      await server.Wire.InvokeAsync<Empty>(description, $"{plugin.Identifier}.finalize", new Empty());
    }
    Assert.Equal(["first", "second"], plugin.Opened);
    Assert.Equal(2, plugin.Closed);
    Assert.Equal("first", plugin.Snapshots[0].Address);
    Assert.Equal("second", plugin.Snapshots[1].Address);
    Assert.NotSame(plugin.Snapshots[0], plugin.Snapshots[1]);
  }

  [Fact]
  public async Task PluginFailureMapperPreservesTheRecoveryAction()
  {
    var plugin = new FacilityPlugin().Operation(new Operation<bool, bool>("example.fail"), async (context, value) =>
    {
      await context.Resource<Disconnected>();
      return value;
    });
    await using var server = await PluginServer.StartAsync(plugin);
    var description = await server.Wire.DescribeAsync();
    var failure = await Assert.ThrowsAsync<OperationException>(() => server.Wire.InvokeAsync<BoolValue>(description, "example.fail", new BoolValue()));
    Assert.Equal(Classification.Transient, failure.Failure.Classification);
    Assert.Equal("Reconnect the Device", failure.Failure.Action);
    Assert.Equal("example.fail", failure.Failure.Operation);
    await server.Wire.InvokeAsync<Empty>(description, $"{plugin.Identifier}.finalize", new Empty());
  }
}
