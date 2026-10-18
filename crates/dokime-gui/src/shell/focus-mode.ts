/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Focus: the evidence and the controls that act on it.
 *
 * Focus is a flag on the cycle route's URL rather than component state, so a
 * reload, a refresh after a crash, or a link pasted to a colleague all land in
 * the same view. Only the cycle workspace offers it; every other route reads
 * `false` and keeps its page bar.
 */

/* === External Imports === */

import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback } from "react";
import { z } from "zod";

/* === Local Imports === */

import { cycleIdOfPathname } from "@/shell/route-scope";

/* === Types === */

export type FocusPresentation = {
  /** The page bar's actions: Start, Stop and the focus toggle. A frozen report has none. */
  actions: boolean;
  /** The page bar's back chevron and title. */
  heading: boolean;
  /** The focus toggle's visible text. */
  toggleLabel: string;
};

export type FocusMode = {
  /** True only on a cycle route whose URL carries the flag. */
  focused: boolean;
  /** No-op away from a cycle route, which has nowhere to write the flag. */
  setFocused: (next: boolean) => void;
};

/* === Constants === */

/**
 * `focus=1` and nothing else.
 *
 * A coercing boolean would read `focus=0` as focus, which is exactly the URL an
 * operator writes to leave it.
 */
export const focusSearchSchema = z
  .preprocess((value) => {
    const candidate = Array.isArray(value) ? value.at(-1) : value;
    return candidate === true || candidate === "1" || candidate === "true" ? true : undefined;
  }, z.literal(true).optional())
  .catch(undefined);

/* === Public Functions === */

/**
 * What the page bar keeps in each mode.
 *
 * Focus gives up the heading the operator can already read off the evidence,
 * never the actions: hiding the bar once took Start and the only visible way
 * back out with it, leaving Escape as an undiscoverable exit.
 */
export function focusPresentation(focused: boolean, frozen = false): FocusPresentation {
  return {
    actions: !frozen,
    heading: !focused,
    toggleLabel: focused ? "Leave Focus" : "Focus",
  };
}

/** Reads and writes the focus flag of the cycle route the operator is on. */
export function useFocusMode(): FocusMode {
  const navigate = useNavigate();
  const { cycleId, focused } = useRouterState({
    select: (state) => {
      const cycleId = cycleIdOfPathname(state.location.pathname);
      return {
        cycleId,
        focused: cycleId !== null && focusSearchSchema.parse((state.location.search as { focus?: unknown }).focus) === true,
      };
    },
  });

  const setFocused = useCallback(
    (next: boolean) => {
      if (!cycleId) return;
      void navigate({
        params: { cycleId },
        replace: true,
        search: (current: Record<string, unknown>) => ({
          ...current,
          focus: next ? (true as const) : undefined,
        }),
        to: "/cycles/$cycleId",
      });
    },
    [cycleId, navigate],
  );

  return { focused, setFocused };
}
