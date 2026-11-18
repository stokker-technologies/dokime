// SPDX-License-Identifier: Apache-2.0

using Dokime;
using Dokime.Clients;
using System.Globalization;

// Read Composed Library Status and Product Suites Through One Client Tree.
using var client = Client.Connect(args.FirstOrDefault() ?? "http://127.0.0.1:7000");
var status = await client.Status.Get();
Console.WriteLine(status.Status.Identity.HostId);
var suites = await client.Suites.List("");
foreach (var suite in suites.Values)
  Console.WriteLine(suite.Metadata.Label);

// Keep Exact Statistics in Their Typed Representation When Displaying Them.
if (args is [_, var execution, var stream, var field])
{
  var response = await client.Evidence.Statistics(stream, [], new Window(), [], field, "", scope: new EvidenceScope { ExecutionId = execution });
  var statistics = response.Statistics;
  Console.WriteLine($"Count: {statistics.Count.ToString(CultureInfo.InvariantCulture)}");
  Console.WriteLine($"Sum: {statistics.Sum}");
  Console.WriteLine($"Mean: {statistics.Mean}");
}
