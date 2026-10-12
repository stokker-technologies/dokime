/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { Pause, Play, RotateCcw, Square, TimerReset } from "lucide-react";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import type { RunCommandState } from "@/features/execution/run-command";

/* === Types === */

type RunCommandPanelProps = {
  onPause: () => void;
  onRerun: () => void;
  onRestart: () => void;
  onResume: () => void;
  onStart: () => void;
  onStop: () => void;
  state: RunCommandState;
};

/* === Components === */

export function RunCommandPanel({
  onPause,
  onRerun,
  onRestart,
  onResume,
  onStart,
  onStop,
  state,
}: RunCommandPanelProps) {
  if (state.kind === "none") {
    return null;
  }

  if (state.kind === "start") {
    return (
      <div className="control-buttons flex w-full flex-col gap-2">
        <Button
          className="h-11 w-full"
          disabled={!state.runtimeReady}
          onClick={onStart}
          variant="default"
        >
          <Play data-icon="inline-start" />
          {state.runtimeReady ? "Start" : "Runtime unavailable"}
        </Button>
      </div>
    );
  }

  if (state.kind === "running") {
    return (
      <fieldset className="control-buttons grid w-full gap-2 sm:grid-cols-[1fr_1fr_1fr]">
        <legend className="sr-only">Run controls</legend>
        <Button className="h-10 w-full" onClick={onStop} variant="danger">
          <Square data-icon="inline-start" />
          Stop
        </Button>
        <Button
          className="h-10 w-full"
          onClick={state.paused ? onResume : onPause}
          variant={state.paused ? "default" : "outline"}
        >
          {state.paused ? <Play data-icon="inline-start" /> : <Pause data-icon="inline-start" />}
          {state.paused ? "Resume" : "Pause"}
        </Button>
        <Button
          className="h-10 w-full"
          disabled={!state.canRestart || !state.runtimeReady}
          onClick={onRestart}
          variant="outline"
        >
          <RotateCcw data-icon="inline-start" />
          Restart
        </Button>
      </fieldset>
    );
  }

  if (state.kind === "stopping") {
    return (
      <fieldset className="control-buttons grid w-full gap-2 sm:grid-cols-[1fr_1fr_1fr]">
        <legend className="sr-only">Run controls</legend>
        <Button className="h-10 w-full" disabled onClick={onStop} variant="danger">
          <Square data-icon="inline-start" />
          Stopping...
        </Button>
        <Button className="h-10 w-full" disabled onClick={onPause} variant="outline">
          <Pause data-icon="inline-start" />
          Pause
        </Button>
        <Button className="h-10 w-full" disabled onClick={onRestart} variant="outline">
          <RotateCcw data-icon="inline-start" />
          Restart
        </Button>
      </fieldset>
    );
  }

  if (state.kind === "rerun") {
    return (
      <div className="control-buttons flex w-full flex-col gap-2">
        <Button
          className="h-11 w-full"
          disabled={!state.runtimeReady}
          onClick={onRerun}
          variant="outline"
        >
          <TimerReset data-icon="inline-start" />
          {state.runtimeReady ? "Re-run" : "Runtime unavailable"}
        </Button>
      </div>
    );
  }

  return null;
}
