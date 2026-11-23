// Dokime — SPDX-License-Identifier: Apache-2.0
import { elapsedWindowLabel } from "@/plots/axis";

export type PlotWindowOption = {
  label: string;
  detail: string;
  value: number | null;
};

/** The operator's plot window for one view. */
export type PlotWindowState = {
  index: number;
  option: PlotWindowOption;
  choose: (index: number) => void;
  refresh: PlotRefreshState;
};

/** Refresh ownership shared by a view's toolbar and its plot reader. */
export type PlotRefreshState = {
  key: number;
  refreshing: boolean;
  updatedAt: number | null;
  request: () => void;
  onChange: (status: { refreshing: boolean; updatedAt?: number | null }) => void;
};

/**
 * What the window control is called, and how it states its value.
 *
 * An elapsed axis counts from the start of the run in its own unit, so a
 * window over it is an elapsed window and has to be stated in that unit; a
 * wall-time axis keeps the plain name and the human-scaled label. The control
 * says which one it is rather than leaving the operator to reconcile
 * "2 min" with an axis that spans 0.2 s.
 */
export function plotWindowTitle(elapsedUnit: string | null | undefined): string {
  return elapsedUnit ? "Elapsed Window" : "Plot Window";
}

export function plotWindowValueLabel(option: PlotWindowOption, elapsedUnit: string | null | undefined): string {
  return elapsedUnit && typeof option.value === "number" ? elapsedWindowLabel(option.value, elapsedUnit) : option.label;
}

export function plotWindowDetail(option: PlotWindowOption, elapsedUnit: string | null | undefined): string {
  return elapsedUnit && typeof option.value === "number"
    ? `last ${elapsedWindowLabel(option.value, elapsedUnit)} of elapsed time`
    : option.detail;
}

export function plotWindowOptions(presets: readonly number[]): [PlotWindowOption, ...PlotWindowOption[]] {
  const durations = presets.map((seconds) => {
    const unit = seconds % 3_600 === 0 ? "h" : seconds % 60 === 0 ? "min" : "s";
    const label = elapsedWindowLabel(seconds, unit);
    return { label, detail: `last ${label}`, value: seconds };
  });
  const options: [PlotWindowOption, ...PlotWindowOption[]] = [{ label: "All", detail: "all stored samples", value: null }];
  options.unshift(...durations);
  return options;
}

export const plotWindows = plotWindowOptions([60, 120, 300, 600, 1_800, 3_600]);
export const defaultPlotWindowIndex = plotWindows.findIndex((option) => option.value === 300);
