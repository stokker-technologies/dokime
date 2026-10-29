/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Debounced, Serialized Settings Writes With Inline Status. */

import { useCallback, useEffect, useRef, useState } from "react";

export type AutosaveState = { kind: "error"; message: string } | { kind: "idle" } | { kind: "saved" } | { kind: "saving" };

export type Autosave<T> = {
  /** Writes after the debounce; use for text and number inputs. */
  change: (value: T) => void;
  retry: () => void;
  state: AutosaveState;
};

const GUI_SETTINGS_DEBOUNCE_MS = 100;
const GUI_SETTINGS_SAVED_MS = 2_000;

const SAVED_MS = GUI_SETTINGS_SAVED_MS;

export function useAutosave<T>(save: (value: T) => Promise<unknown>, message: (error: unknown) => string = describeError): Autosave<T> {
  const [state, setState] = useState<AutosaveState>({ kind: "idle" });
  const last = useRef<{ value: T } | null>(null);
  const saveRef = useRef(save);
  saveRef.current = save;
  const writer = useRef<ReturnType<typeof createDebouncedWriter<T>> | null>(null);
  if (!writer.current)
    writer.current = createDebouncedWriter(
      async (value) => {
        await saveRef.current(value);
      },
      (next) => setState(next.kind === "error" ? { ...next, message: message(new Error(next.message)) } : next),
    );
  useEffect(() => () => writer.current?.flush(), []);
  const change = useCallback((value: T) => {
    last.current = { value };
    writer.current?.change(value);
  }, []);

  /* "Saved" states what just happened, then stops taking up the row. */
  useEffect(() => {
    if (state.kind !== "saved") return undefined;
    const clear = setTimeout(() => setState({ kind: "idle" }), SAVED_MS);
    return () => clearTimeout(clear);
  }, [state]);

  const retry = useCallback(() => {
    if (last.current !== null) {
      writer.current?.reset();
      writer.current?.change(last.current.value);
    }
  }, []);

  return { change, retry, state };
}

/** Serialize Debounced Writes And Keep The Latest Edit While A Write Is In Flight. */
export function createDebouncedWriter<T>(write: (value: T) => Promise<void>, changed: (state: AutosaveState) => void) {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pending: { value: T } | undefined;
  let writing = false;
  let ready = false;
  let failed = false;

  async function drain() {
    if (writing || !pending || !ready || failed) return;
    const current = pending;
    pending = undefined;
    writing = true;
    try {
      await write(current.value);
      if (!pending) changed({ kind: "saved" });
    } catch (error) {
      pending ??= current;
      failed = true;
      changed({ kind: "error", message: describeError(error) });
    } finally {
      writing = false;
      if (pending && ready && !failed) void drain();
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

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
