/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { isTauriDesktop } from "@/lib/desktop-open";

/* === Public Functions === */

export async function setDesktopWindowTitle(title: string): Promise<boolean> {
  const trimmed = title.trim();
  if (!trimmed || !isTauriDesktop()) {
    return false;
  }
  try {
    const { commands } = await import("@/lib/desktop/commands");
    await commands.desktopSetTitle(trimmed);
    return true;
  } catch {
    return false;
  }
}
