// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Plugins;
using Harmos;

namespace Generation;

public static class Surface
{
  // Compiler Story: No Host or Product Runtime Is Needed.
  public static async Task Calls(Dokime.Client client, Context context)
  {
    _ = await client.Analyses.List();
    _ = await context.Host.Files.Capture(new Dokime.FileSource(), "output");
    _ = await context.Host.Content.Artifacts.Create(new FileReference(), "capture.bin", "application/octet-stream");
    _ = await context.Host.Evidence.Statistics("stream", Array.Empty<uint>(), new Window(), Array.Empty<Dokime.Clients.Output>(), "field", "step");
  }
}
