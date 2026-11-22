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
import {
  observeRowWindow,
  ROW_WINDOW_THRESHOLD,
  type RowWindow,
  type RowWindowMeasurements,
  rowWindow,
  scrollingAncestor,
  WINDOW_ROW_ATTRIBUTE,
} from "./row-window";

/* === Types === */

export type RowWindowState = {
  /** The element that holds the rows: a list or a table body. */
  anchor: RefObject<HTMLElement | null>;
  window: RowWindow;
  /** Brings row `index` into the window and the viewport, then focuses it. */
  focusIndex: (index: number, focus: (row: HTMLElement) => void) => void;
};

/* === Public Functions === */

/** Window `count` rows of one height inside whichever ancestor scrolls them. */
export function useRowWindow(count: number): RowWindowState {
  const anchor = useRef<HTMLElement | null>(null);
  const [measurements, setMeasurements] = useState<RowWindowMeasurements>({ pitch: null, offset: 0, height: 0 });
  const windowed = count >= ROW_WINDOW_THRESHOLD;

  useLayoutEffect(() => {
    const list = anchor.current;
    if (!windowed || !list) {
      return;
    }
    return observeRowWindow(list, (next) => {
      setMeasurements((current) => {
        const unchanged = current.pitch === next.pitch && current.height === next.height && current.offset === next.offset;
        return unchanged ? current : next;
      });
    });
  }, [windowed]);

  const { pitch, offset, height } = measurements;
  const slice = rowWindow(count, pitch, offset, height);
  const focusIndex = useCallback(
    (index: number, focus: (row: HTMLElement) => void) => {
      const list = anchor.current;
      if (!list) {
        return;
      }
      const scroller = scrollingAncestor(list);
      if (windowed && pitch && scroller) {
        const top = list.getBoundingClientRect().top - scroller.getBoundingClientRect().top + scroller.scrollTop;
        scroller.scrollTop = top + index * pitch - scroller.clientHeight / 2;
      }
      /* The row renders on the next frame once the window has moved. */
      window.requestAnimationFrame(() => {
        const row = list.querySelector<HTMLElement>(`[${WINDOW_ROW_ATTRIBUTE}="${index}"]`);
        if (row) {
          focus(row);
        }
      });
    },
    [pitch, windowed],
  );
  return { anchor, focusIndex, window: slice };
}
