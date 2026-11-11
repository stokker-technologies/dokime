/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { create } from "zustand";

/* === Types === */

/** An x range in a plot's cartesian x space. */
export type SummaryWindow = { from: number; to: number };

type SummaryWindowStore = {
  /**
   * The x range each live plot shows after a zoom, by plot id. Its series
   * detail and summaries are requested over this range; a following plot has
   * no entry. Session state only: never persisted, never in a report.
   */
  windows: Readonly<Record<string, SummaryWindow>>;
  widths: Readonly<Record<string, number>>;
  heights: Readonly<Record<string, number>>;
  setPanelSize: (plotId: string, size: { width: number; height: number } | null) => void;
  setPanelWidth: (plotId: string, width: number | null) => void;
  setSummaryWindow: (plotId: string, window: SummaryWindow | null) => void;
};

/* === Stores === */

export const useSummaryWindowStore = create<SummaryWindowStore>()((set) => ({
  windows: {},
  widths: {},
  heights: {},
  setPanelSize: (plotId, size) => set((state) => {
    if (state.widths[plotId] === size?.width && state.heights[plotId] === size?.height) return state;
    const widths = { ...state.widths };
    const heights = { ...state.heights };
    if (size) { widths[plotId] = size.width; heights[plotId] = size.height; }
    else { delete widths[plotId]; delete heights[plotId]; }
    return { widths, heights };
  }),
  setPanelWidth: (plotId, width) =>
    set((state) => {
      if (state.widths[plotId] === width || (width === null && state.widths[plotId] === undefined)) return state;
      const widths = { ...state.widths };
      if (width === null) delete widths[plotId];
      else widths[plotId] = width;
      return { widths };
    }),
  setSummaryWindow: (plotId, window) =>
    set((state) => {
      const current = state.windows[plotId];
      if (window && current?.from === window.from && current.to === window.to) return state;
      if (!window && !current) return state;
      const windows = { ...state.windows };
      if (window) windows[plotId] = window;
      else delete windows[plotId];
      return { windows };
    }),
}));
