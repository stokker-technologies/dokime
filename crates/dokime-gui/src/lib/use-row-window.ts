/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Windowed rendering for long lists of one row height: the cycle tree and the
 * tables of the content pages.
 *
 * Every row has one height, so the visible slice is arithmetic rather than
 * measurement, as the console does it: the row pitch is read once from a
 * rendered row, and spacers above and below stand in for the rows outside
 * the viewport and its overscan. The list scrolls in whatever ancestor
 * scrolls it, so the hook finds that ancestor rather than owning one. Below
 * `ROW_WINDOW_THRESHOLD` rows every row renders: a short list costs nothing
 * and keeps every row in the document for keyboard focus and search.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { type RefObject, useCallback, useLayoutEffect, useRef, useState } from "react";

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/** From this many rows on, only the visible slice renders. */
export const ROW_WINDOW_THRESHOLD = 200;

/** Rows rendered beyond each edge of the viewport, so scrolling never shows a gap. */
const OVERSCAN = 16;

/** How many rows render before the row pitch is known. */
const UNMEASURED_ROWS = 64;

/** The attribute every windowed row carries, so the pitch is read off a real row. */
export const WINDOW_ROW_ATTRIBUTE = "data-window-row";

/* === Types === */

/** The slice that renders, and the space the rows outside it hold. */
export type RowWindow = {
  first: number;
  last: number;
  /** Pixels of the rows above `first`. */
  before: number;
  /** Pixels of the rows from `last` on. */
  after: number;
};

export type RowWindowState = {
  /** The element that holds the rows: a list or a table body. */
  anchor: RefObject<HTMLElement | null>;
  window: RowWindow;
  /** Brings row `index` into the window and the viewport, then focuses it. */
  focusIndex: (index: number, focus: (row: HTMLElement) => void) => void;
};

/* === Classes === */

/* === Public Functions === */

/**
 * The slice of `count` rows of `pitch` pixels that a viewport of `height`
 * pixels shows when it is scrolled `offset` pixels into the list.
 */
export function rowWindow(count: number, pitch: number | null, offset: number, height: number): RowWindow {
  if (count < ROW_WINDOW_THRESHOLD) return { after: 0, before: 0, first: 0, last: count };
  if (!pitch || pitch <= 0) return { after: 0, before: 0, first: 0, last: Math.min(count, UNMEASURED_ROWS) };
  const first = Math.min(count, Math.max(0, Math.floor(offset / pitch) - OVERSCAN));
  const last = Math.min(count, Math.max(first, Math.ceil((offset + height) / pitch) + OVERSCAN));
  return { after: (count - last) * pitch, before: first * pitch, first, last };
}

/** Window `count` rows of one height inside whichever ancestor scrolls them. */
export function useRowWindow(count: number): RowWindowState {
  const anchor = useRef<HTMLElement | null>(null);
  const [pitch, setPitch] = useState<number | null>(null);
  const [view, setView] = useState({ height: 0, offset: 0 });
  const windowed = count >= ROW_WINDOW_THRESHOLD;

  useLayoutEffect(() => {
    if (!windowed) return undefined;
    const list = anchor.current;
    if (!list) return undefined;
    const scroller = scrollingAncestor(list);
    const read = () => {
      const row = list.querySelector<HTMLElement>(`[${WINDOW_ROW_ATTRIBUTE}]`);
      if (row) {
        const gap = Number.parseFloat(getComputedStyle(list).rowGap) || 0;
        const measured = row.getBoundingClientRect().height + gap;
        if (measured > 0) setPitch((current) => (current === measured ? current : measured));
      }
      const top = list.getBoundingClientRect().top - (scroller ? scroller.getBoundingClientRect().top : 0);
      const height = scroller ? scroller.clientHeight : window.innerHeight;
      setView((current) => {
        const offset = Math.max(0, -top);
        return current.offset === offset && current.height === height ? current : { height, offset };
      });
    };
    read();
    const target: HTMLElement | Window = scroller ?? window;
    target.addEventListener("scroll", read, { passive: true });
    const observer = new ResizeObserver(read);
    observer.observe(scroller ?? document.documentElement);
    return () => {
      target.removeEventListener("scroll", read);
      observer.disconnect();
    };
  }, [windowed]);

  const slice = rowWindow(count, pitch, view.offset, view.height);
  const focusIndex = useCallback(
    (index: number, focus: (row: HTMLElement) => void) => {
      const list = anchor.current;
      if (!list) return;
      const scroller = scrollingAncestor(list);
      if (windowed && pitch && scroller) {
        const top = list.getBoundingClientRect().top - scroller.getBoundingClientRect().top + scroller.scrollTop;
        scroller.scrollTop = top + index * pitch - scroller.clientHeight / 2;
      }
      /* The row renders on the next frame once the window has moved. */
      window.requestAnimationFrame(() => {
        const row = list.querySelector<HTMLElement>(`[${WINDOW_ROW_ATTRIBUTE}="${index}"]`);
        if (row) focus(row);
      });
    },
    [pitch, windowed],
  );
  return { anchor, focusIndex, window: slice };
}

/* === Private Functions === */

/** The nearest ancestor that scrolls vertically, or `null` for the page itself. */
function scrollingAncestor(node: HTMLElement): HTMLElement | null {
  for (let parent = node.parentElement; parent; parent = parent.parentElement) {
    const overflow = getComputedStyle(parent).overflowY;
    if (overflow === "auto" || overflow === "scroll") return parent;
  }
  return null;
}

/* === Entry Point === */
