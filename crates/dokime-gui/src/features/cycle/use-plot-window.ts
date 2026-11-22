// Dokime — SPDX-License-Identifier: Apache-2.0
import { useState } from "react";
import { useStore } from "zustand";
import { createPlotWindow } from "./plot-window-state";

export function usePlotWindow() {
  const [store] = useState(createPlotWindow);
  return useStore(store);
}
