// Dokime — SPDX-License-Identifier: Apache-2.0
import { getContext, setContext } from "svelte";

export type PlotMeasurements = {
  setVisible: (plotId: string, visible: boolean | null) => void;
  setPanelSize: (plotId: string, size: { width: number; height: number } | null) => void;
  setPanelWidth: (plotId: string, width: number | null) => void;
  setSummaryWindow: (plotId: string, window: { from: number; to: number } | null) => void;
};

/** Frozen Plots Reset Zoom. Live Plots Resume Following. */
export type PlotInteraction = {
  zoom: boolean;
  syncKey?: string;
  measurements?: PlotMeasurements | null;
};

export const plotInteractionContext = Symbol("plot-interaction");
const defaultInteraction: PlotInteraction = { zoom: false };

export function providePlotInteraction(interaction: PlotInteraction) {
  setContext(plotInteractionContext, interaction);
}

export function getPlotInteraction(): PlotInteraction {
  return getContext<PlotInteraction | undefined>(plotInteractionContext) ?? defaultInteraction;
}
