/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { commands } from "@/lib/desktop/commands";

export async function restartDesktopApp(): Promise<void> {
  await commands.desktopRestart();
}
