/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

import { TimerReset } from "lucide-react";

import { useEffect, useMemo, useState } from "react";

// Local Imports

import { Slider } from "@/components/ui/slider";

import { elapsedWindowLabel } from "@/plots/axis";

/* === Re-Exports === */

/* === Constants === */

const GUI_PLOT_WINDOW_PRESETS_S = [60, 120, 300, 600, 1_800, 3_600];

/* === Types === */

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
};

/* === Components === */

export function PlotWindowSlider({
  elapsedUnit = null,
  index,
  option,
  onIndexChange,
}: {
  /** The unit of the elapsed axis under this control, when there is one. */
  elapsedUnit?: string | null;
  index: number;
  option: PlotWindowOption;
  onIndexChange: (index: number) => void;
}) {
  const [draftIndex, setDraftIndex] = useState(index);
  const options = usePlotWindowOptions();
  const draftOption = options[draftIndex] ?? option;
  const title = plotWindowTitle(elapsedUnit);
  const detail = plotWindowDetail(draftOption, elapsedUnit);

  useEffect(() => {
    setDraftIndex(index);
  }, [index]);

  return (
    /* Label, slider and value sit on one baseline; the value keeps a fixed
       6rem slot so the slider never resizes as the label text changes. */
    <div className="grid h-full min-w-0 grid-cols-(--layout-plot-window) items-center gap-2">
      <span className="inline-flex min-w-0 items-center gap-2 text-xs font-bold uppercase text-muted-foreground">
        <TimerReset className="shrink-0" size={14} />
        <span className="truncate">{title}</span>
      </span>
      <Slider
        aria-label={title}
        aria-valuetext={detail}
        className="min-w-0"
        max={options.length - 1}
        min={0}
        step={1}
        value={[draftIndex]}
        onValueChange={(value) => {
          setDraftIndex(value[0] ?? 0);
        }}
        onValueCommit={(value) => {
          const next = value[0] ?? 0;
          setDraftIndex(next);
          onIndexChange(next);
        }}
      />
      <span className="whitespace-nowrap text-right text-sm font-semibold tabular-nums" title={detail}>
        {plotWindowValueLabel(draftOption, elapsedUnit)}
      </span>
    </div>
  );
}

/** The live view's window control; reports always cover their full evidence cut. */
export function PlotWindowControl({ elapsedUnit, live, window }: { elapsedUnit: string | null; live: boolean; window: PlotWindowState }) {
  if (!live) {
    return null;
  }
  return (
    <div className="w-56 xl:w-80">
      <PlotWindowSlider elapsedUnit={elapsedUnit} index={window.index} onIndexChange={window.choose} option={window.option} />
    </div>
  );
}

/* === Hooks === */

/** The plot window one document view scales its plots by. */
export function usePlotWindow(): PlotWindowState {
  const options = usePlotWindowOptions();
  const [index, setIndex] = useState(0);
  const option = options[index] ?? options[0];

  return {
    choose: (next) => {
      setIndex(options[next] ? next : 0);
    },
    index,
    option,
  };
}

/* === Public Functions === */

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
  return [{ label: "All", detail: "all stored samples", value: null }, ...durations];
}

/* === Private Functions === */

function usePlotWindowOptions(): [PlotWindowOption, ...PlotWindowOption[]] {
  return useMemo(() => plotWindowOptions(GUI_PLOT_WINDOW_PRESETS_S), []);
}
