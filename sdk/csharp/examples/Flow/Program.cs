// SPDX-License-Identifier: Apache-2.0

using System.Diagnostics;
using Dokime;
using Dokime.Clients;

if (args.Length != 2)
  throw new ArgumentException("Run Capture Flow: Supply the Host URL and the Built Test Executable Path");

// Start the Sidecar and Read Its Bound Address.
var start = new ProcessStartInfo(args[1])
{
  RedirectStandardOutput = true,
  RedirectStandardError = true
};
start.ArgumentList.Add("--remote");
start.ArgumentList.Add("--address");
start.ArgumentList.Add("127.0.0.1:0");
using var process = Process.Start(start)
    ?? throw new InvalidOperationException("Start Capture Plugin: Supply a Runnable Test Executable");
var errors = process.StandardError.ReadToEndAsync();
Task<string>? output = null;
try
{
  var readiness = await process.StandardOutput.ReadLineAsync()
      ?? throw new InvalidOperationException("Start Capture Plugin: Inspect the Sidecar Error Output");
  var address = readiness[(readiness.LastIndexOf(' ') + 1)..];
  output = process.StandardOutput.ReadToEndAsync();

  // Admit the Plugin and Observe Its Activation.
  using var client = Client.Connect(args[0]);
  var created = await client.Plugins.Create("example", new()
  {
    Remote = new()
    {
      Url = "http://" + address
    }
  }, new()
  {
    Enabled = true
  });
  try
  {
    var entry = created.Entry;
    await using var watching = client.Plugins.Watch(entry.Name).GetAsyncEnumerator();
    while (entry.Description is null)
    {
      if (entry.Failure is not null)
        throw new InvalidOperationException($"Start Capture Plugin: Resolve the Host Failure: {entry.Failure}");
      if (!await watching.MoveNextAsync())
        throw new InvalidOperationException("Start Capture Plugin: Reconnect After the Host Watch Closed");
      entry = (await client.Plugins.Get(entry.Name)).Entry;
    }

    // Materialize the Authored Suite and Start One Execution.
    var suites = (await client.Suites.List(entry.Name)).Values;
    var suite = (await client.Suites.Get(suites[0].Metadata.Identifier, entry.Name, 1)).Value;
    var plan = new SelectionPlan
    {
      Selections = { new RegisteredSelection { SuiteId = suite.Metadata.Identifier, TemplateId = suite.Templates[0].Metadata.Identifier } }
    };
    var cases = (await client.Cases.Create(plan, false)).Cases;
    var cycle = (await client.Cycles.Create([], new()
    {
      Label = "Capture"
    }, true, "")).Cycle;
    var execution = Guid.NewGuid().ToString();
    await client.Executions.Create(cycle.CycleId, [new ExecutionBinding { ExecutionId = execution, CaseId = cases[0].CaseId }]);

    // Answer Operator Questions Through the Client.
    await foreach (var progress in client.Executions.Progress.Watch(execution))
    {
      if (progress.Phase != "prompt")
        continue;
      var pending = await client.Prompts.List("", true, executionId: execution);
      foreach (var item in pending.Prompts)
      {
        var question = (await client.Prompts.Get(item.Prompt.PromptId)).Prompt;
        Console.WriteLine(question.Prompt.Definition.Message);
        foreach (var action in question.Prompt.Definition.Actions)
          Console.WriteLine($"{action.Identifier}: {action.Label}");
        var answer = Console.ReadLine()
            ?? throw new InvalidOperationException("Answer Capture Prompt: Enter One of the Listed Action Identifiers");
        await client.Prompts.Answers.Create(item.Prompt.PromptId, answer, new MapValue(), true);
      }
    }
    Console.WriteLine((await client.Executions.Get(execution)).Execution);
  }
  finally
  {
    var entry = (await client.Plugins.Get(created.Entry.Name)).Entry;
    await client.Plugins.Delete(entry.Name, entry.Revision);
  }
}
finally
{
  if (!process.HasExited)
    process.Kill(entireProcessTree: true);
  await process.WaitForExitAsync();
  if (output is not null)
    await output;
  Console.Error.Write(await errors);
}
