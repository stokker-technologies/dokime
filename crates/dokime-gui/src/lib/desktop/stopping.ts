/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The desktop's orderly stop as the shell announces it on `desktop://stopping`
 * (`src-tauri/src/app/closing.rs`).
 *
 * From the first announcement the backend is going away, so nothing the window
 * asks for afterwards can succeed: a call that fails then never settles, and
 * neither a toast nor an error card appears for it. The native shell has
 * already hidden the window before starting the backend drain.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { listen } from "@tauri-apps/api/event";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

export const STOPPING_EVENT = "desktop://stopping";

let stopping = false;

/* === Types === */

export type StoppingAnnouncement = { stopping: boolean };

/* === Classes === */

/* === Public Functions === */

/** Apply one shell announcement. A kept-open window runs normally again. */
export function announceStopping(announcement: StoppingAnnouncement): void {
  stopping = announcement.stopping;
}

/** Never settle once the stop owns a failure; otherwise reject with it. */
export function settleUnlessStopping<T>(error: unknown): Promise<T> {
  if (!stopping && !shuttingDown(error)) {
    return Promise.reject(error);
  }
  if (!stopping) {
    announceStopping({ stopping: true });
  }
  return new Promise<T>(() => {});
}

/** Follow the shell's announcements; outside the desktop shell there are none. */
export function watchStopping(): () => void {
  let unlisten: (() => void) | undefined;
  let disposed = false;
  try {
    void listen<StoppingAnnouncement>(STOPPING_EVENT, (event) => announceStopping(event.payload))
      .then((stop) => {
        if (disposed) {
          stop();
        } else {
          unlisten = stop;
        }
      })
      .catch(() => undefined);
  } catch {
    // No Tauri bridge: a browser build or a test renders without a shell.
  }
  return () => {
    disposed = true;
    unlisten?.();
  };
}

/* === Private Functions === */

/** The shell's own refusal once its stop has begun, bare or as an RPC session error. */
function shuttingDown(error: unknown): boolean {
  if (!error || typeof error !== "object" || !("kind" in error)) {
    return false;
  }
  if (error.kind === "shutting_down") {
    return true;
  }
  if (error.kind !== "session" || !("detail" in error)) {
    return false;
  }
  const detail = error.detail;
  return !!detail && typeof detail === "object" && "kind" in detail && detail.kind === "shutting_down";
}

/* === Entry Point === */
