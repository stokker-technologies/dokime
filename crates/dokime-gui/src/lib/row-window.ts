// Dokime — SPDX-License-Identifier: Apache-2.0

export const ROW_WINDOW_THRESHOLD = 200;
export const WINDOW_ROW_ATTRIBUTE = "data-window-row";
const OVERSCAN = 16;
const UNMEASURED_ROWS = 64;

export type RowWindowMeasurements = { pitch: number | null; offset: number; height: number };

export type RowWindow = {
  first: number;
  last: number;
  /** Pixels of the rows above `first`. */
  before: number;
  /** Pixels of the rows from `last` on. */
  after: number;
};

export function rowWindow(count: number, pitch: number | null, offset: number, height: number): RowWindow {
  if (count < ROW_WINDOW_THRESHOLD) {
    return { after: 0, before: 0, first: 0, last: count };
  }
  if (!pitch || pitch <= 0) {
    return { after: 0, before: 0, first: 0, last: Math.min(count, UNMEASURED_ROWS) };
  }
  const first = Math.min(count, Math.max(0, Math.floor(offset / pitch) - OVERSCAN));
  const last = Math.min(count, Math.max(first, Math.ceil((offset + height) / pitch) + OVERSCAN));
  return { after: (count - last) * pitch, before: first * pitch, first, last };
}

/** Measure the row pitch once per layout pass and track the containing scrollport. */
export function observeRowWindow(list: HTMLElement, update: (measurements: RowWindowMeasurements) => void): () => void {
  const scroller = scrollingAncestor(list);
  const target = scroller ?? window;
  let pitch: number | null = null;
  const read = () => {
    const row = list.querySelector<HTMLElement>(`[${WINDOW_ROW_ATTRIBUTE}]`);
    if (row) {
      const gap = Number.parseFloat(getComputedStyle(list).rowGap) || 0;
      const height = row.getBoundingClientRect().height + gap;
      if (height > 0) {
        pitch = height;
      }
    }
    const top = list.getBoundingClientRect().top - (scroller?.getBoundingClientRect().top ?? 0);
    const height = scroller?.clientHeight ?? window.innerHeight;
    update({ pitch, offset: Math.max(0, -top), height });
  };
  read();
  target.addEventListener("scroll", read, { passive: true });
  const observer = new ResizeObserver(read);
  observer.observe(scroller ?? document.documentElement);
  return () => {
    target.removeEventListener("scroll", read);
    observer.disconnect();
  };
}

/** The nearest ancestor that scrolls vertically, or `null` for the page itself. */
export function scrollingAncestor(node: HTMLElement): HTMLElement | null {
  for (let parent = node.parentElement; parent; parent = parent.parentElement) {
    const overflow = getComputedStyle(parent).overflowY;
    if (overflow === "auto" || overflow === "scroll") {
      return parent;
    }
  }
  return null;
}
