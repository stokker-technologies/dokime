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
import { isTauriDesktop } from "@dokime/document/lib/surface-environment";

/* === Public Functions === */

export async function openLocalPath(path: string | null | undefined): Promise<boolean> {
  const trimmed = path?.trim();
  if (!trimmed || !isTauriDesktop()) {
    return false;
  }
  await commands.desktopOpenPath(trimmed);
  return true;
}
