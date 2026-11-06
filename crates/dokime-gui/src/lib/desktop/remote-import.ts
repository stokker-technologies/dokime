/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Native Remote File Import Notifications
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { failureOf } from "@/lib/notification-details";
import { listen } from "@tauri-apps/api/event";

// Local Imports
import type { NativeSession } from "@/app/native-session/session";

import { toast } from "@/lib/notifications";
import { isTauriDesktop } from "@/lib/desktop-open";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export function watchRemoteImports(owner: NativeSession): () => void {
  if (!isTauriDesktop()) return () => {};
  let disposed = false;
  const releases: (() => void)[] = [];
  const retain = (release: () => void) => {
    if (disposed) release();
    else releases.push(release);
  };
  void listen("desktop://remote-imported", () => {
    void owner.refresh();
  }).then(retain);
  void listen("desktop://remote-import-failed", (event) => {
    void owner.refresh();
    toast.error(failureOf(event.payload, "Remote Import Failed"));
  }).then(retain);
  return () => {
    disposed = true;
    for (const release of releases) release();
  };
}

/* === Private Functions === */

/* === Entry Point === */
