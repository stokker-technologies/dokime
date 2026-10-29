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
import { isTauriDesktop } from "@/lib/desktop-open";

/* === Public Functions === */

export async function setDesktopWindowTitle(title: string): Promise<boolean> {
  const trimmed = title.trim();
  if (!trimmed || !isTauriDesktop()) {
    return false;
  }
  try {
    await commands.desktopSetTitle(trimmed);
    return true;
  } catch {
    return false;
  }
}
