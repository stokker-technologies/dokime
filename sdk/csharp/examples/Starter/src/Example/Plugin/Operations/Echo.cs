/* ***
 *
 * Starter Project
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

using Dokime.Operations;

namespace Example.Plugin.Operations;

/* === Classes === */

public static class Echo
{
  [Operation("example.echo", Visibility = Visibility.Public)]
  public static Task<string> Call(Context context, string request)
  {
    return Task.FromResult(request);
  }
}
