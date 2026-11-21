/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Which cycle and which attempt the URL names.
 *
 * Four parts of the shell need this — the sidebar tree, focus mode, the
 * console drawer and the console's badge — and each of them used to carry its
 * own copy of the pattern. One answer, in one place, so they can never
 * disagree about what the operator is looking at.
 */

/* === External Imports === */

import { useRouterState } from "@tanstack/react-router";

/* === Constants === */

const CYCLE_ROUTE = /^\/cycles\/([^/]+)/;

/* === Public Functions === */

export function cycleIdOfPathname(pathname: string): string | null {
  return CYCLE_ROUTE.exec(pathname)?.[1] ?? null;
}

/* === Hooks === */

/** The cycle the route names, or null everywhere else. */
export function useRouteCycleId(): string | null {
  return useRouterState({ select: (state) => cycleIdOfPathname(state.location.pathname) });
}

/** The cycle and the attempt the route names, as one read. */
export function useRouteCycleScope(): { cycleId: string | null; executionId: string | null } {
  return useRouterState({
    select: (state) => ({
      cycleId: cycleIdOfPathname(state.location.pathname),
      executionId: (state.location.search as { executionId?: string }).executionId ?? null,
    }),
  });
}
