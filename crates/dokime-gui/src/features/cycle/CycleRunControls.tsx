/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The cycle's run controls, published into the page bar's action slot while
 * the cycle itself is selected.
 *
 * Start, Pause or Resume, Stop Cycle, and Run Again once the cycle ended —
 * only what the lifecycle offers, each on `--bar-control-height` so the page
 * bar keeps its height. The component reads the cycle's own query, so the
 * node it is published as stays stable while the cycle runs. Stop asks first:
 * the confirmation belongs to the workspace, which `onStop` opens.
 */

/* === External Imports === */

import { Pause, Play, RotateCcw, Square } from "lucide-react";
import { useMemo } from "react";
import { toast } from "sonner";

/* === Local Imports === */

import { Button } from "@/components/ui/button";
import { useCycleCommands, useCycleDetailQuery } from "./cycle-queries";
import {
  buildCycleCases,
  cycleRunRefusal,
  isCycleTerminal,
  rerunnableCaseIds,
  runControlAvailability,
  runControlVisibility,
  startableCaseCount,
} from "./cycle-workspace-model";

/* === Types === */

export type CycleRunControlsProps = {
  cycleId: string;
  onStop: () => void;
};

/* === Constants === */

/** Every page-bar control: the bar's own control height, 48px at touch. */
export const PAGE_BAR_CONTROL_CLASS = "h-[var(--bar-control-height)] px-3";

/** The controls' row: one grid row, `--bar-gap` apart. */
export const PAGE_BAR_CONTROLS_CLASS =
  "grid auto-cols-max grid-flow-col items-center gap-x-[var(--bar-gap)]";

/* === Components === */

export function CycleRunControls({ cycleId, onStop }: CycleRunControlsProps) {
  const detailQuery = useCycleDetailQuery(cycleId);
  const commands = useCycleCommands(cycleId);
  const plan = detailQuery.data?.plan ?? null;
  const cases = useMemo(() => buildCycleCases(plan), [plan]);
  if (!plan) return null;

  const status = plan.test_cycle.status;
  const startable = startableCaseCount(cases);
  const refusal = cycleRunRefusal(cases);
  const availability = runControlAvailability(status, startable);
  const visibility = runControlVisibility(status, startable, refusal !== null);
  const terminal = isCycleTerminal(status);
  const rerunCases = rerunnableCaseIds(cases);
  const runAgainRefusal =
    rerunCases.length === 0 ? "No case of this cycle can be run again." : null;

  /* A command reports what happened, not what was attempted. */
  const run = (report: { done: string; failed: string }, action: () => Promise<unknown>) => {
    action()
      .then(() => toast.success(report.done))
      .catch((error: unknown) =>
        toast.error(report.failed, {
          description: error instanceof Error ? error.message : String(error),
        }),
      );
  };

  return (
    <fieldset aria-label="Cycle Controls" className={PAGE_BAR_CONTROLS_CLASS}>
      {visibility.run && !terminal ? (
        <Button
          aria-label="Start Cycle"
          className={PAGE_BAR_CONTROL_CLASS}
          disabled={commands.runQueued.isPending || !availability.run}
          onClick={() =>
            run({ done: "Queued cases started", failed: "The queued cases could not start." }, () =>
              commands.runQueued.mutateAsync(),
            )
          }
          title={availability.run ? undefined : (refusal ?? undefined)}
          variant="default"
        >
          <Play data-icon="inline-start" />
          Start
        </Button>
      ) : null}
      {visibility.pauseResume === "pause" ? (
        <Button
          aria-label="Pause Cycle"
          className={PAGE_BAR_CONTROL_CLASS}
          disabled={commands.pauseCycle.isPending || !availability.pause}
          onClick={() =>
            run({ done: "Cycle paused", failed: "The cycle could not be paused." }, () =>
              commands.pauseCycle.mutateAsync(),
            )
          }
          variant="outline"
        >
          <Pause data-icon="inline-start" />
          Pause
        </Button>
      ) : visibility.pauseResume === "resume" ? (
        <Button
          aria-label="Resume Cycle"
          className={PAGE_BAR_CONTROL_CLASS}
          disabled={commands.resumeCycle.isPending || !availability.resume}
          onClick={() =>
            run({ done: "Cycle resumed", failed: "The cycle could not be resumed." }, () =>
              commands.resumeCycle.mutateAsync(),
            )
          }
          variant="outline"
        >
          <Play data-icon="inline-start" />
          Resume
        </Button>
      ) : null}
      {visibility.stop || status === "stopping" ? (
        <Button
          className={PAGE_BAR_CONTROL_CLASS}
          disabled={commands.stopCycle.isPending || !availability.stop}
          onClick={onStop}
          variant="danger"
        >
          <Square data-icon="inline-start" />
          {status === "stopping" ? "Stopping Cycle" : "Stop Cycle"}
        </Button>
      ) : null}
      {terminal ? (
        <Button
          aria-label="Run Cycle Again"
          className={PAGE_BAR_CONTROL_CLASS}
          disabled={commands.rerunCycle.isPending || runAgainRefusal !== null}
          onClick={() =>
            run({ done: "Cycle running again", failed: "The cycle could not run again." }, () =>
              commands.rerunCycle.mutateAsync(rerunCases),
            )
          }
          title={runAgainRefusal ?? undefined}
          variant="default"
        >
          <RotateCcw data-icon="inline-start" />
          Run Again
        </Button>
      ) : null}
    </fieldset>
  );
}
