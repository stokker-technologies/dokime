// Dokime — SPDX-License-Identifier: Apache-2.0
import { createStore } from "zustand/vanilla";
import { storeState } from "../../stores/store-state";
import { defaultPlotWindowIndex, plotWindows, type PlotWindowState } from "./plot-window";

/**
 * One Document's Window Selection And Explicit Refresh Status. Each Reader
 * Selects What It Follows, So Refresh Status Reaches Only Its Control.
 */
export function createPlotWindow() {
  const store = createStore<PlotWindowState>((set) => ({
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
  return {
    ...store,
    state: storeState(store, (state) => state),
    option: storeState(store, (state) => state.option),
    refreshKey: storeState(store, (state) => state.refresh.key),
  };
}

export type PlotWindow = ReturnType<typeof createPlotWindow>;
