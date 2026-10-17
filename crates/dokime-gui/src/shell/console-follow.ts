/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Follow-tail for the console's record list.
 *
 * Following is read from where the list sits, not from which way it last
 * moved: the list's own jump to the newest row fires the same scroll event an
 * operator's wheel does, and only the geometry tells the two apart. Scrolling
 * up releases the tail; reaching the bottom again, or "Jump to Latest", takes
 * it back.
 *
 * Pure: no React, no path aliases, so the Node test runner imports it as is.
 */

/* === Types === */

/** What decides whether the list is pinned to its newest row. */
export type ScrollMetrics = {
  clientHeight: number;
  scrollHeight: number;
  scrollTop: number;
};

/* === Constants === */

/** Within one row of the bottom still counts as following. */
export const FOLLOW_SLACK_PX = 22;

/* === Public Functions === */

/** Whether the list shows its last row, give or take `slack` pixels. */
export function consoleTailPinned({ clientHeight, scrollHeight, scrollTop }: ScrollMetrics, slack: number = FOLLOW_SLACK_PX): boolean {
  return scrollHeight - scrollTop - clientHeight <= slack;
}
