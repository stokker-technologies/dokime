// Dokime — SPDX-License-Identifier: Apache-2.0
import { createContext, useContext } from "react";
import type { PlotMeasurements } from "../../plots/plot-interaction";
import type { ReportDefaults } from "../../plots/plot-settings";
import { usePlotPalette } from "../../plots/use-plot-palette";
import { SvelteMount } from "../SvelteMount";
import PlotGroupGrid, { type PlotGroupGridProps } from "./PlotGroupGrid.svelte";

export type { PlotSettingsChange } from "./PlotGroupGrid.svelte";

export const PlotMeasurementsContext = createContext<PlotMeasurements | null>(null);
export const PlotReportDefaultsContext = createContext<ReportDefaults | undefined>(undefined);

export function LivePlotGroupGrid(props: PlotGroupGridProps) {
  const measurements = useContext(PlotMeasurementsContext);
  const plotPalette = usePlotPalette();
  const reportDefaults = useContext(PlotReportDefaultsContext);
  return <SvelteMount component={PlotGroupGrid} props={{ ...props, measurements, plotPalette, reportDefaults }} />;
}
