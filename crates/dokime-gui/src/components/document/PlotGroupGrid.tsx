// Dokime — SPDX-License-Identifier: Apache-2.0
import { usePlotPalette } from "../../plots/use-plot-palette";
import { SvelteMount } from "../SvelteMount";
import PlotGroupGrid, { type PlotGroupGridProps } from "./PlotGroupGrid.svelte";

export type { PlotSettingsChange } from "./PlotGroupGrid.svelte";

export function LivePlotGroupGrid(props: PlotGroupGridProps) {
  const plotPalette = usePlotPalette();
  return <SvelteMount component={PlotGroupGrid} props={{ ...props, plotPalette }} />;
}
