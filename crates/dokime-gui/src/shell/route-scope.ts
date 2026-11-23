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

/* === Constants === */

const CYCLE_ROUTE = /^\/cycles\/([^/]+)/;

/* === Public Functions === */

export function cycleIdOfPathname(pathname: string): string | null {
  return CYCLE_ROUTE.exec(pathname)?.[1] ?? null;
}
