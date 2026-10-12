/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Public Functions === */

/* Loaded on demand: the desktop command module is unavailable in a browser. */
export async function restartDesktopApp(): Promise<void> {
  const { commands } = await import("@/lib/desktop/commands");
  await commands.desktopRestart();
}
