// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Plugins;
using Dokime.Templates;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Stories.Support;
using Xunit;

namespace Stories.Integration;

public sealed class AuthorDeclaresQualifiedMembers
{
  [Fact]
  public async Task PluginHooksUseProtectedWellKnownOperationsAndAuthorState()
  {
    var plugin = new LifecyclePlugin();
    Assert.Empty(plugin.Events);
    await using var server = await PluginServer.StartAsync(plugin);
    var description = await server.Wire.DescribeAsync();
    foreach (var name in new[] { "initialize", "finalize" })
    {
      var identifier = $"{plugin.Identifier}.{name}";
      var operation = Assert.Single(description.Declarations.Operations, value => value.Id == identifier);
      Assert.Equal(Harmos.Visibility.Protected, operation.Visibility);
      Assert.Equal("google.protobuf.Empty", operation.Request);
      Assert.Equal("google.protobuf.Empty", operation.Response);
      await server.Wire.InvokeAsync<Empty>(description, identifier, new Empty());
    }
    Assert.Equal(new[] { "initialize", "finalize" }, plugin.Events);
  }

  private sealed class LifecyclePlugin : Dokime.Plugin
  {
    internal List<string> Events { get; } = [];
    public override Task InitializeAsync(Dokime.Operations.Context context, CancellationToken cancellationToken = default)
    {
      Assert.Same(this, context.Plugin);
      Events.Add("initialize");
      return Task.CompletedTask;
    }
    public override Task FinalizeAsync(Dokime.Operations.Context context, CancellationToken cancellationToken = default)
    {
      Assert.Same(this, context.Plugin);
      Events.Add("finalize");
      return Task.CompletedTask;
    }
  }

  [Fact]
  public void ClassesDeclareQualifiedMembersWithoutRunningThem()
  {
    var plugin = new ExamplePlugin();
    var description = plugin.Describe();
    var suite = Assert.Single(description.Suites);
    var template = Assert.Single(suite.Templates);
    Assert.Equal("capture.confirmation", template.Identifier);
    Assert.Equal("capture.confirmation.confirm", Assert.Single(template.Steps));
    Assert.Equal("capture.confirmation.confirmed", Assert.Single(template.Criteria));
    Assert.Equal("capture.confirmation.confirmation", Assert.Single(template.Records));
    Assert.Equal(0, plugin.Reads);
  }

  [Fact]
  public void PlainPayloadsPreserveTypedValues()
  {
    RoundTrip(true);
    RoundTrip("plain");
    RoundTrip(long.MinValue);
    RoundTrip(2.125);
    Assert.Equal(new byte[] { 0, 1, 255 }, Payload<byte[]>.Unpack(Payload<byte[]>.Pack([0, 1, 255]).ToByteString()));
    RoundTrip(new BoolValue { Value = true });
  }

  [Fact]
  public void OperationContextHasNoExecutionOrCarrierEscapeHatch()
  {
    var properties = typeof(Dokime.Operations.Context).GetProperties().Select(property => property.Name);
    Assert.DoesNotContain("Execution", properties);
    Assert.DoesNotContain("Host", properties);
    Assert.DoesNotContain("Transport", properties);
    Assert.Contains("Plugin", properties);
    Assert.NotEqual(typeof(Dokime.Operations.Context), typeof(Dokime.Templates.Context));
  }

  private static void RoundTrip<T>(T value) => Assert.Equal(value, Payload<T>.Unpack(Payload<T>.Pack(value).ToByteString()));
}
