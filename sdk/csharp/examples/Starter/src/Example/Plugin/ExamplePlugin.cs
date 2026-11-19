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
using Example.Plugin.Suites.Capture;

namespace Example.Plugin;

/* === Classes === */

public sealed class ExamplePlugin : Dokime.Plugin
{
  public override IReadOnlyList<Type> Suites => [typeof(CaptureSuite)];
  public override IReadOnlyList<Delegate> Operations => [(Func<Dokime.Operations.Context, string, Task<string>>)Echo.Call];
}
