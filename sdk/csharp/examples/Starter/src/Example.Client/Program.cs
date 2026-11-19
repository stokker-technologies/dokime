/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

using Dokime;
using Example.Plugin;
using Example.Plugin.Suites.Capture;

/* === Entry Point === */

// Initialize Client
var client = await Client.ConnectAsync();
Directory.CreateDirectory("_output");

// Initialize & Register Plugin
var plugin = new ExamplePlugin();
await client.Plugins.RegisterAsync(plugin);
Console.WriteLine($"Plugin {plugin.Identifier} Is Ready");

// Initialize Test Cycle
var started = DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss");
var cycle = await client.CycleAsync(label: $"Capture {started}");

// Initialize Test Cases
var cases = await client.Cases.MaterializeAsync<ConfirmationTestTemplate>();
foreach (var testCase in cases)
{
  // Initialize Test Execution
  var execution = await cycle.ExecutionAsync(testCase);

  // Run Test Execution
  Console.WriteLine($"Running {testCase.Label}");
  await foreach (var progress in execution.RunAsync())
  {
    Console.WriteLine($"  {progress.Phase}");
  }

  // Seal Test Execution & Report Outcome
  var outcome = await execution.SealAsync();
  Console.WriteLine(outcome.Status == ResultStatus.Passed ? "Passed" : "Not Passed");
}

// Seal Test Cycle
await cycle.SealAsync();

// Save Evidence
var evidence = await cycle.EvidenceAsync();
var saved = $"_output/{cycle.Identifier}.zip";
await File.WriteAllBytesAsync(saved, evidence);
Console.WriteLine($"Saved {saved}");

// Close Client
await client.CloseAsync();
