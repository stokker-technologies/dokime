// Dokime — SPDX-License-Identifier: Apache-2.0
import { useCallback, useRef, useState } from "react";
import { plotWindows, type PlotWindowState } from "./plot-window";

/** The plot window one document view scales its plots by. */
export function usePlotWindow(): PlotWindowState {
  const [index, setIndex] = useState(() => plotWindows.findIndex((option) => option.value === 300));
  const option = plotWindows[index] ?? plotWindows[0];
  const [refreshKey, setRefreshKey] = useState(0);
  const [status, setStatus] = useState({ refreshing: false, updatedAt: null as number | null });
  const refreshingRef = useRef(false);
  const onChange = useCallback((next: { refreshing: boolean; updatedAt?: number | null }) => {
    refreshingRef.current = next.refreshing;
    setStatus((previous) => ({ ...previous, ...next }));
  }, []);
  const request = useCallback(() => {
    if (refreshingRef.current) {
      return;
    }
    refreshingRef.current = true;
    setStatus((previous) => ({ ...previous, refreshing: true }));
    setRefreshKey((previous) => previous + 1);
  }, []);

  return {
    choose: (next) => {
      setIndex(plotWindows[next] ? next : 0);
    },
    index,
    option,
    refresh: { key: refreshKey, ...status, request, onChange },
  };
}
