// SPDX-License-Identifier: Apache-2.0

using Stories.Support;
using Xunit;
using Xunit.Abstractions;

namespace Stories.Integration;

public sealed class PluginPassesHarmosConformance(ITestOutputHelper output)
{
  [Fact]
  [Trait("Layer", "Conformance")]
  public async Task RunEveryReferenceCaseWithManagedAndRemotePlacement()
  {
    // Locate the Exact-Revision Host and SDK-Authored Conformance Plugin.
    var host = ReferenceHost.Artifact("HARMOS_CONFORMANCE_HOST");
    var plugin = ReferenceHost.Artifact("DOKIME_CONFORMANCE_PLUGIN");

    // Run All Managed Cases and Require Successful Process Shutdown.
    var managed = await ReferenceHost.RunAsync(host, plugin);
    output.WriteLine(managed.Evidence);
    Assert.True(managed.ExitCode == 0, managed.Evidence);

    // Run All Remote Cases and Verify Stop Leaves the Server Available.
    var remote = await ReferenceHost.RunRemoteAsync(host, plugin);
    output.WriteLine(remote.Evidence);
    Assert.True(remote.ExitCode == 0, remote.Evidence);
    Assert.True(remote.StillServing);
  }
}
