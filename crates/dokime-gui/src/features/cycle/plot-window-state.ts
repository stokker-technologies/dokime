// Dokime — SPDX-License-Identifier: Apache-2.0
import { createStore } from "zustand/vanilla";
import { defaultPlotWindowIndex, plotWindows, type PlotWindowState } from "./plot-window";

/** One Document's Window Selection And Explicit Refresh Status. */
export function createPlotWindow() {
  return createStore<PlotWindowState>((set) => ({
    index: defaultPlotWindowIndex,
    option: plotWindows[defaultPlotWindowIndex] ?? plotWindows[0],
    choose(next) {
      const index = plotWindows[next] ? next : 0;
      set({ index, option: plotWindows[index] });
    },
    refresh: {
      key: 0,
      refreshing: false,
      updatedAt: null,
      onChange(status) {
        set(({ refresh }) => ({ refresh: { ...refresh, ...status } }));
      },
      request() {
        set((state) => {
          const refresh = state.refresh;
          if (refresh.refreshing) {
            return state;
          }
          return { refresh: { ...refresh, refreshing: true, key: refresh.key + 1 } };
        });
      },
    },
  }));
}
