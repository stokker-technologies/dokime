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
  public static readonly Record<bool> Confirmation = new("Confirmation");

  [Step("Confirm")]
  public async Task Confirm(Context context)
  {
    var answer = await context.PromptAsync("Continue?", choices: ["Yes", "No"]);
    await context.RecordAsync(Confirmation, answer == "Yes");
  }

  [Criterion("Confirmed")]
  public Task<bool> Confirmed(Context context)
  {
    return context.Records(Confirmation, step: Confirm).LastAsync();
  }
}
