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

/* === Imports === */

/* === Re-Exports === */

/* === Constants === */

/** Within one row of the bottom still counts as following. */
export const FOLLOW_SLACK_PX = 22;

/** Breathing room above the first row and below the last, inside the scroll area. */
export const CONSOLE_LIST_PADDING_PX = 8;

/* === Types === */

/** What decides whether the list is pinned to its newest row. */
export type ScrollMetrics = {
  clientHeight: number;
  scrollHeight: number;
  scrollTop: number;
};

/** The rows a fixed-pitch list draws for one scroll position. */
export type ConsoleWindow = {
  /** First drawn row index, inclusive. */
  first: number;
  /** Last drawn row index, exclusive. */
  last: number;
  /** The scrolled content height, padding included. */
  contentHeight: number;
  /** Where the first drawn row sits inside the content. */
  offset: number;
};

/* === Classes === */

/* === Public Functions === */

/** Whether the list shows its last row, give or take `slack` pixels. */
export function consoleTailPinned({ clientHeight, scrollHeight, scrollTop }: ScrollMetrics, slack: number = FOLLOW_SLACK_PX): boolean {
  return scrollHeight - scrollTop - clientHeight <= slack;
}

/**
 * Whether a new row pulls the list to its tail. The geometry the operator
 * last scrolled to decides, not the drawer's follow flag alone: that flag
 * reaches the list a render after the wheel, and a row landing in between
 * would otherwise snap the list back down before the release takes effect.
 */
export function consoleFollowsTail(follow: boolean, pinnedAtLastScroll: boolean): boolean {
  return follow && pinnedAtLastScroll;
}

/**
 * The slice of `count` rows of `rowHeight` a viewport of `height` shows at
 * `scrollTop`, with `overscan` rows on each side. The padding sits inside the
 * scrolled content, so the first and last rows never touch the frame and the
 * row arithmetic stays exact.
 */
export function consoleWindow({
  count,
  height,
  overscan,
  padding = CONSOLE_LIST_PADDING_PX,
  rowHeight,
  scrollTop,
}: {
  count: number;
  height: number;
  overscan: number;
  padding?: number;
  rowHeight: number;
  scrollTop: number;
}): ConsoleWindow {
  const rowsTop = Math.max(0, scrollTop - padding);
  const first = Math.max(0, Math.floor(rowsTop / rowHeight) - overscan);
  const last = Math.min(count, Math.ceil((rowsTop + height) / rowHeight) + overscan);
  return {
    first: Math.min(first, count),
    last: Math.max(Math.min(first, count), last),
    contentHeight: count * rowHeight + 2 * padding,
    offset: padding + first * rowHeight,
  };
}

/* === Private Functions === */

/* === Entry Point === */
