// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Templates;

namespace Generation;

public static class Surface
{
  public static async Task Calls(Client client, Context context, Case testCase)
  {
    var cycle = await client.CycleAsync("Generation");
    var execution = await cycle.ExecutionAsync(testCase);
    await execution.StartAsync();
    await foreach (var progress in execution.WatchAsync())
      Console.WriteLine(progress.Phase);
    _ = await execution.SealAsync();
    await cycle.SealAsync();
    _ = await cycle.EvidenceAsync();
    _ = await context.EnvironmentAsync();
  }
}
