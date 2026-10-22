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
 * neither a toast nor an error card appears for it. The stop becomes visible
 * only once it outlasts a moment, so an idle close shows nothing at all.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { listen } from "@tauri-apps/api/event";
import { useSyncExternalStore } from "react";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

export const STOPPING_EVENT = "desktop://stopping";

const listeners = new Set<() => void>();
let phase: "running" | "hidden" | "visible" = "running";

/* === Types === */

/** `hidden` from the first announcement, `visible` once the stop is shown. */
export type DesktopStopping = "running" | "hidden" | "visible";

export type StoppingAnnouncement = { stopping: boolean; visible: boolean };

/* === Classes === */

/* === Public Functions === */

/** Apply one shell announcement. A kept-open window runs normally again. */
export function announceStopping(announcement: StoppingAnnouncement): void {
  const next: DesktopStopping = !announcement.stopping ? "running" : announcement.visible || phase === "visible" ? "visible" : "hidden";
  if (next === phase) return;
  phase = next;
  for (const listener of listeners) listener();
}

export function desktopStopping(): DesktopStopping {
  return phase;
}

/** Never settle once the stop owns a failure; otherwise reject with it. */
export function settleUnlessStopping<T>(error: unknown): Promise<T> {
  if (phase === "running" && !shuttingDown(error)) return Promise.reject(error);
  if (phase === "running") announceStopping({ stopping: true, visible: false });
  return new Promise<T>(() => {});
}

export function subscribeStopping(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/** Follow the shell's announcements; outside the desktop shell there are none. */
export function watchStopping(): () => void {
  let unlisten: (() => void) | undefined;
  let disposed = false;
  try {
    void listen<StoppingAnnouncement>(STOPPING_EVENT, (event) => announceStopping(event.payload))
      .then((stop) => {
        if (disposed) stop();
        else unlisten = stop;
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

export function useDesktopStopping(): DesktopStopping {
  return useSyncExternalStore(subscribeStopping, desktopStopping);
}

/* === Private Functions === */

/** The shell's own refusal once its stop has begun, bare or as an RPC session error. */
function shuttingDown(error: unknown): boolean {
  if (!error || typeof error !== "object" || !("kind" in error)) return false;
  if (error.kind === "shutting_down") return true;
  if (error.kind !== "session" || !("detail" in error)) return false;
  const detail = error.detail;
  return !!detail && typeof detail === "object" && "kind" in detail && detail.kind === "shutting_down";
}

/* === Entry Point === */
