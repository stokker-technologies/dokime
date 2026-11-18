// SPDX-License-Identifier: Apache-2.0

using Dokime.Plugins;

[SuiteAttribute("Capture")]
public sealed class Capture
{
  public static readonly Record<bool> Confirmation = new("Confirmation");

  [StepAttribute("Confirm")]
  public async Task Confirm(Context context)
  {
    var answer = await context.PromptAsync("Continue?", choices: ["Yes", "No"]);
    await context.RecordAsync(Confirmation, answer == "Yes");
  }

  [CriterionAttribute("Confirmed")]
  public Task<bool> Confirmed(Context context) => context.Records(Confirmation, step: (Func<Context, Task>)Confirm).LastAsync();

  public static Plugin Plugin => new("example", "0.1.0", Suite.From<Capture>());
}
