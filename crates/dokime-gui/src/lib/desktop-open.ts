/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { commands } from "@/lib/desktop/commands";
import { isReportBuild } from "@/lib/ui-mode";

export async function openLocalPath(path: string | null | undefined): Promise<boolean> {
  const trimmed = path?.trim();
  if (!trimmed || !isTauriDesktop()) {
    return false;
  }
  await commands.desktopOpenPath(trimmed);
  return true;
}

export function isTauriDesktop(): boolean {
  /* A frozen report is never the desktop surface, even when it is read inside
     the WebView: the internals are on `window` there, and answering `true`
     would let the report's shared components address a session that this file
     has no part in. */
  if (isReportBuild()) return false;
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
