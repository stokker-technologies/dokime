/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

using Dokime.Templates;

namespace Example.Plugin.Suites.Capture;

/* === Classes === */

public sealed class ConfirmationTestTemplate : TestTemplate
{
  [Step("Confirm")]
  public async Task Confirm(Context context)
  {
    var answer = await context.PromptAsync("Continue?", choices: ["Yes", "No"]);
    await context.RecordAsync(ExamplePlugin.Confirmation, answer == "Yes");
  }

  [Criterion("Confirmed")]
  public Task<bool> Confirmed(Context context)
  {
    return context.Records(ExamplePlugin.Confirmation, step: Confirm).LastAsync();
  }
}
