/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { commands } from "@/lib/desktop/commands";

/* === Public Functions === */

export async function restartDesktopApp(): Promise<void> {
  await commands.desktopRestart();
}
