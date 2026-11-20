// SPDX-License-Identifier: Apache-2.0

using System.Security.Cryptography;
using Dokime;
using Dokime.Clients;
using Google.Protobuf;
using Google.Protobuf.WellKnownTypes;
using Stories.Support;
using Xunit;

namespace Stories.Integration;

public sealed class ClientInspectsABackend
{
  [Fact]
  public async Task ProductAdministrationPreservesTypedValuesAndMutationInputs()
  {
    // Connect the Authenticated Client.
    var backend = new FakeBackend();
    var headers = new Grpc.Core.Metadata { { "authorization", "Bearer test-token" } };
    await using var client = new Client(backend, headers: headers);
    var options = client.Options();

    // Create and Read the Managed Entry.
    var digest = ByteString.CopyFrom(SHA256.HashData([1, 2, 3]));
    var content = new Harmos.FileReference
    {
      Metadata = new Harmos.FileMetadata { Id = "capture" },
      Length = 3,
      Sha256 = digest
    };
    var placement = new PluginPlacement { Managed = new PluginManaged { Content = content, Sha256 = digest } };
    var created = await client.Transport.Plugins.Create("example", placement, new PluginValue { Enabled = true }, "create-example", "capture", context: options);
    var entry = (await client.Transport.Plugins.Get(created.Entry.Name, context: options)).Entry;
    Assert.Equal(created.Entry, entry);
    Assert.Equal(entry, Assert.Single((await client.Transport.Plugins.List(context: options)).Entries));

    // Set the Entry and Restart Its Activation.
    var changed = await client.Transport.Plugins.Set(entry.Name, new PluginValue { Enabled = false }, entry.Revision, options);
    Assert.False(changed.Entry.Value.Enabled);
    var restarted = await client.Transport.Plugins.Restart(entry.Name, entry.Revision, options);
    Assert.True(restarted.Activation > entry.Activation);

    // Replace the Placement and Retain Its Origin.
    var replacement = placement.Clone();
    replacement.Managed.Content.Metadata.Id = "capture-replacement";
    var replaced = await client.Transport.Plugins.Replace(entry.Name, replacement, entry.Revision, "replace-example", options);
    Assert.Equal(replacement, replaced.Entry.Placement);
    Assert.Equal(created.Entry.Origin, replaced.Entry.Origin);

    // Round Trip the Whole Settings Value.
    var values = new Harmos.SettingValues { Values = { ["enabled"] = new Harmos.SettingValue { Flag = false } } };
    var settings = await client.Transport.Plugins.Settings.List(entry.Name, context: options);
    var saved = await client.Transport.Plugins.Settings.Set(entry.Name, values, settings.Revision, "set-example", options);
    Assert.Equal(values, saved.Value);
    Assert.Equal(values, (await client.Transport.Plugins.Settings.List(entry.Name, context: options)).Value);

    // Observe the Entry and Delete It.
    await foreach (var frame in client.Transport.Plugins.Watch(entry.Name, context: options))
      Assert.Equal(replaced.Entry, Assert.Single(frame.Snapshot.Entries));
    await client.Transport.Plugins.Delete(entry.Name, entry.Revision, options);
    Assert.Empty((await client.Transport.Plugins.List(context: options)).Entries);

    // Verify Mutation Inputs and Product Routes.
    var registration = Assert.Single(backend.Requests.OfType<PluginsCreateRequest>());
    Assert.Equal("create-example", registration.RequestId);
    Assert.Equal("capture", registration.Origin);
    Assert.Equal(placement, registration.Placement);
    var update = Assert.Single(backend.Requests.OfType<PluginsReplaceRequest>());
    Assert.Equal("replace-example", update.RequestId);
    Assert.Equal(entry.Revision, update.ExpectedRevision);
    var setting = Assert.Single(backend.Requests.OfType<Dokime.Clients.PluginsResources.SettingsSetRequest>());
    Assert.Equal("set-example", setting.RequestId);
    Assert.Equal(settings.Revision, setting.ExpectedRevision);
    Assert.Equal(new[]
    {
      "/dokime.host.v1.Plugins/Create", "/dokime.host.v1.Plugins/Get", "/dokime.host.v1.Plugins/List",
      "/dokime.host.v1.Plugins/Set", "/dokime.host.v1.Plugins/Restart", "/dokime.host.v1.Plugins/Replace",
      "/dokime.host.v1.plugins.Settings/List", "/dokime.host.v1.plugins.Settings/Set", "/dokime.host.v1.plugins.Settings/List",
      "/dokime.host.v1.Plugins/Watch", "/dokime.host.v1.Plugins/Delete", "/dokime.host.v1.Plugins/List"
    }, backend.Calls);
  }

  [Fact]
  public async Task ProductWrappersPreserveTypedOperationPayloadsAndAdmission()
  {
    var contract = new Harmos.Plugins.Operation<DoubleValue, DoubleValue>("example.echo", Harmos.Visibility.Public).Contract;
    var discovered = new Harmos.DiscoveredContract
    {
      Id = contract.Id,
      Visibility = contract.Visibility,
      Fingerprint = contract.Fingerprint,
      Policy = contract.Policy?.Clone(),
      Protobuf = new Harmos.Protobuf { Request = contract.Request, Response = contract.Response }
    };
    var backend = new FakeBackend { Operation = discovered.Clone() };
    var headers = new Grpc.Core.Metadata { { "authorization", "Bearer test-token" } };
    await using var client = new Client(backend, headers: headers);
    await client.Transport.Status.Get(client.Options());
    var operation = client.Operation(new Dokime.Operations.Operation<double, double>("example.echo"));
    Assert.Equal(42.5, await operation.CallAsync(42.5));
    Assert.Equal(["/dokime.host.v1.Status/Get", "/dokime.host.v1.Operations/Get", "/dokime.host.v1.Invocations/Create"], backend.Calls);

    backend.Operation.Fingerprint = Google.Protobuf.ByteString.CopyFromUtf8("different");
    await Assert.ThrowsAsync<InvalidDataException>(() => operation.CallAsync(42.5));
    backend.Operation = discovered.Clone();
    backend.Operation.Visibility = Harmos.Visibility.Protected;
    await Assert.ThrowsAsync<InvalidDataException>(() => operation.CallAsync(42.5));
    Assert.Equal(1, backend.Calls.Count(method => method == "/dokime.host.v1.Invocations/Create"));
  }

  [Fact]
  public async Task BoundCaseReturnsProductValuesAndPreservesRefusal()
  {
    var server = new FakeBackend();
    var headers = new Grpc.Core.Metadata { { "authorization", "Bearer test-token" } };
    var client = new Client(server, headers: headers);
    var testCase = await client.Cases.GetAsync("case-a", "cycle-a");
    Assert.Equal("case-a", testCase.Identifier);
    var failure = await Assert.ThrowsAsync<SdkError>(() => client.Cases.GetAsync("", "cycle-a"));
    Assert.Equal("InvalidArgument", failure.Failure.Code);
    await client.CloseAsync();
    await Assert.ThrowsAsync<InvalidOperationException>(() => client.Cases.GetAsync("case-a"));
  }

  [Theory]
  [InlineData("http://localhost")]
  [InlineData("http://[::1]")]
  [InlineData("http://localhost:0")]
  [InlineData("https://host:443/path")]
  [InlineData("http://user@host:7000")]
  [InlineData(" http://host:7000")]
  public void RefuseNonOriginConnectionTargets(string target) => Assert.Throws<ArgumentException>(() => Client.Endpoint(target));

  [Theory]
  [InlineData("http://localhost:80")]
  [InlineData("http://[::1]:7000")]
  public void AcceptExplicitDefaultAndIpv6Ports(string target) => Assert.Equal(target, Client.Endpoint(target).OriginalString);

  [Theory]
  [InlineData("DOKIME_REMOTE=http://host:7000 # Host", "http://host:7000")]
  [InlineData("DOKIME_REMOTE='path#file.pair'", "path#file.pair")]
  [InlineData("DOKIME_REMOTE=\"a\\tb\"", "a\tb")]
  public void ParseConnectionAssignments(string text, string expected) => Assert.Equal(expected, Configuration.Parse(text, "settings.env")["DOKIME_REMOTE"]);

  [Theory]
  [InlineData("X=one\nX=two")]
  [InlineData("export X=one")]
  [InlineData("X=one\r\r\n")]
  [InlineData("X=\"unclosed")]
  [InlineData("X=\"bad\\q\"")]
  public void RefuseMalformedAssignments(string text) => Assert.Throws<ArgumentException>(() => Configuration.Parse(text, "settings.env"));
}
