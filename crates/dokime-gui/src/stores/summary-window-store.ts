/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { create } from "zustand";

/** An x range in a plot's cartesian x space. */
export type SummaryWindow = { from: number; to: number };

type SummaryWindowStore = {
  /**
   * The x range each live plot shows after a zoom, by plot id. Its series
   * summaries are requested over this range; a plot following its window has
   * no entry. Session state only: never persisted, never in a report.
   */
  windows: Readonly<Record<string, SummaryWindow>>;
  setSummaryWindow: (plotId: string, window: SummaryWindow | null) => void;
};

export const useSummaryWindowStore = create<SummaryWindowStore>()((set) => ({
  windows: {},
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
