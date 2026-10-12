/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Settings save themselves.
 *
 * A Save button beside every field made the operator confirm a preference they
 * had already expressed. Typing settles for 400 ms and then writes; a switch, a
 * select or a segmented control writes at once. The row reports the outcome
 * where the change was made, and a failure keeps the entered value and offers
 * the write again.
 */

/* === External Imports === */

import { useCallback, useEffect, useRef, useState } from "react";

/* === Types === */

export type AutosaveState =
  | { kind: "error"; message: string }
  | { kind: "idle" }
  | { kind: "saved" }
  | { kind: "saving" };

export type Autosave<T> = {
  /** Writes after the debounce; use for text and number inputs. */
  change: (value: T) => void;
  /** Writes at once; use for switches, selects and segmented controls. */
  commit: (value: T) => void;
  retry: () => void;
  state: AutosaveState;
};

/* === Constants === */

const DEBOUNCE_MS = 400;
const SAVED_MS = 2_000;

/* === Public Functions === */

export function useAutosave<T>(
  save: (value: T) => Promise<unknown>,
  message: (error: unknown) => string = describeError,
): Autosave<T> {
  const [state, setState] = useState<AutosaveState>({ kind: "idle" });
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const last = useRef<T | null>(null);
  const saveRef = useRef(save);
  saveRef.current = save;

  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current);
    },
    [],
  );

  const commit = useCallback(
    (value: T) => {
      if (timer.current) clearTimeout(timer.current);
      last.current = value;
      setState({ kind: "saving" });
      saveRef
        .current(value)
        .then(() => setState({ kind: "saved" }))
        .catch((error: unknown) => setState({ kind: "error", message: message(error) }));
    },
    [message],
  );

  const change = useCallback(
    (value: T) => {
      if (timer.current) clearTimeout(timer.current);
      timer.current = setTimeout(() => commit(value), DEBOUNCE_MS);
    },
    [commit],
  );

  /* "Saved" states what just happened, then stops taking up the row. */
  useEffect(() => {
    if (state.kind !== "saved") return undefined;
    const clear = setTimeout(() => setState({ kind: "idle" }), SAVED_MS);
    return () => clearTimeout(clear);
  }, [state]);

  const retry = useCallback(() => {
    if (last.current !== null) commit(last.current);
  }, [commit]);

  return { change, commit, retry, state };
}

/* === Private Functions === */

function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
