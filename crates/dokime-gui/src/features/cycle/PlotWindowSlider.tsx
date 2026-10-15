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

import { useEffect, useState } from "react";

// Local Imports

import { Slider } from "@/components/ui/slider";

import { elapsedWindowLabel } from "@/plots/axis";

/* === Re-Exports === */

/* === Constants === */

export const PLOT_WINDOW_OPTIONS: PlotWindowOption[] = [
  { label: "30 s", detail: "last 30 seconds", value: 30 },
  { label: "1 min", detail: "last minute", value: 60 },
  { label: "2 min", detail: "last 2 minutes", value: 120 },
  { label: "5 min", detail: "last 5 minutes", value: 300 },
  { label: "10 min", detail: "last 10 minutes", value: 600 },
  { label: "30 min", detail: "last 30 minutes", value: 1800 },
  { label: "60 min", detail: "last 60 minutes", value: 3600 },
  { label: "All", detail: "all retained samples", value: null },
];

export const DEFAULT_PLOT_WINDOW_INDEX = 2;

/* === Types === */

export type PlotWindowOption = {
  label: string;
  detail: string;
  value: number | null;
};

/** The operator's plot window: the chosen option and whether they chose it. */
export type PlotWindowState = {
  index: number;
  option: PlotWindowOption;
  overridden: boolean;
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
  const draftOption =
    PLOT_WINDOW_OPTIONS[draftIndex] ?? option ?? PLOT_WINDOW_OPTIONS[DEFAULT_PLOT_WINDOW_INDEX];
  const title = plotWindowTitle(elapsedUnit);
  const detail = plotWindowDetail(draftOption, elapsedUnit);

  useEffect(() => {
    setDraftIndex(index);
  }, [index]);

  return (
    /* Label, slider and value sit on one baseline; the value keeps a fixed
       64px slot so the slider never resizes as the label text changes. */
    <div className="grid h-full min-w-0 grid-cols-[auto_minmax(0,1fr)_4rem] items-center gap-2">
      <span className="inline-flex min-w-0 items-center gap-1.5 text-xs font-bold uppercase text-muted-foreground">
        <TimerReset className="shrink-0" size={14} />
        <span className="truncate">{title}</span>
      </span>
      <Slider
        aria-label={title}
        aria-valuetext={detail}
        className="min-w-0"
        max={PLOT_WINDOW_OPTIONS.length - 1}
        min={0}
        step={1}
        value={[draftIndex]}
        onValueChange={(value) => {
          setDraftIndex(value[0] ?? DEFAULT_PLOT_WINDOW_INDEX);
        }}
        onValueCommit={(value) => {
          const next = value[0] ?? DEFAULT_PLOT_WINDOW_INDEX;
          setDraftIndex(next);
          onIndexChange(next);
        }}
      />
      <span className="text-right text-sm font-semibold tabular-nums" title={detail}>
        {plotWindowValueLabel(draftOption, elapsedUnit)}
      </span>
    </div>
  );
}

/**
 * The plot window in the header of the plots it scales. A frozen plot shows
 * the window it was drawn with: saying so is honest, offering a slider that
 * changes nothing is not.
 */
export function PlotWindowControl({
  elapsedUnit,
  live,
  window,
}: {
  elapsedUnit: string | null;
  live: boolean;
  window: PlotWindowState;
}) {
  if (!live) {
    return (
      <span className="text-xs text-muted-foreground">
        {elapsedUnit ? "Elapsed window" : "Plot window"}{" "}
        {plotWindowValueLabel(window.option, elapsedUnit)}
      </span>
    );
  }
  return (
    <div className="w-56 xl:w-80">
      <PlotWindowSlider
        elapsedUnit={elapsedUnit}
        index={window.index}
        onIndexChange={window.choose}
        option={window.option}
      />
    </div>
  );
}

/* === Hooks === */

/** The plot window one document view scales its plots by. */
export function usePlotWindow(): PlotWindowState {
  const [index, setIndex] = useState(DEFAULT_PLOT_WINDOW_INDEX);
  const [overridden, setOverridden] = useState(false);
  const option = PLOT_WINDOW_OPTIONS[index] ?? PLOT_WINDOW_OPTIONS[DEFAULT_PLOT_WINDOW_INDEX];

  return {
    choose: (next) => {
      setIndex(next);
      setOverridden(true);
    },
    index,
    option,
    overridden,
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

export function plotWindowValueLabel(
  option: PlotWindowOption,
  elapsedUnit: string | null | undefined,
): string {
  return elapsedUnit && option.value !== null
    ? elapsedWindowLabel(option.value, elapsedUnit)
    : option.label;
}

export function plotWindowDetail(
  option: PlotWindowOption,
  elapsedUnit: string | null | undefined,
): string {
  return elapsedUnit && option.value !== null
    ? `last ${elapsedWindowLabel(option.value, elapsedUnit)} of elapsed time`
    : option.detail;
}

/* === Private Functions === */
