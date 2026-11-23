/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

using Example.Plugin.Operations;
using Dokime.Templates;
using Example.Plugin.Suites.Capture;

namespace Example.Plugin;

/* === Classes === */

public sealed class ExamplePlugin : Dokime.Plugin
{
  public static readonly Record<bool> Confirmation = new("Confirmation", identifier: "confirmation");
  public override IReadOnlyList<IRecord> Records => [Confirmation];
  public override IReadOnlyList<Type> Suites => [typeof(CaptureSuite)];
  public override IReadOnlyList<Delegate> Operations => [(Func<Dokime.Operations.Context, string, Task<string>>)Echo.Call];
}
