/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Debounced, Serialized Settings Writes With Inline Status. */

/* === External Imports === */

import { failureOf } from "@/lib/notification-details";

/* === Types === */

export type AutosaveState =
  | { kind: "error"; message: string; failure?: unknown }
  | { kind: "idle" }
  | { kind: "saved" }
  | { kind: "saving" };

/* === Constants === */

const GUI_SETTINGS_DEBOUNCE_MS = 100;
export const GUI_SETTINGS_SAVED_MS = 2_000;

/** Serialize Debounced Writes And Keep The Latest Edit While A Write Is In Flight. */
export function createDebouncedWriter<T>(write: (value: T) => Promise<void>, changed: (state: AutosaveState) => void) {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pending: { value: T } | undefined;
  let writing = false;
  let ready = false;
  let failed = false;

  async function drain() {
    if (writing || !pending || !ready || failed) {
      return;
    }
    const current = pending;
    pending = undefined;
    writing = true;
    try {
      await write(current.value);
      if (!pending) {
        changed({ kind: "saved" });
      }
    } catch (error) {
      pending ??= current;
      failed = true;
      changed({ kind: "error", message: failureOf(error, "Operation Failed").message, failure: error });
    } finally {
      writing = false;
      if (pending && ready && !failed) {
        void drain();
      }
    }
  }

  return {
    change(value: T) {
      clearTimeout(timer);
      pending = { value };
      ready = false;
      changed({ kind: "saving" });
      timer = setTimeout(() => {
        ready = true;
        void drain();
      }, GUI_SETTINGS_DEBOUNCE_MS);
    },
    flush() {
      clearTimeout(timer);
      ready = true;
      void drain();
    },
    cancel() {
      clearTimeout(timer);
      pending = undefined;
      ready = false;
    },
    reset() {
      clearTimeout(timer);
      pending = undefined;
      ready = false;
      failed = false;
    },
  };
}

/* === Private Functions === */
