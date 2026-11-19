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

namespace Example.Plugin.Suites.Capture;

/* === Classes === */

public sealed class CaptureSuite : Suite
{
  public override IReadOnlyList<Type> Templates => [typeof(ConfirmationTestTemplate)];
}
