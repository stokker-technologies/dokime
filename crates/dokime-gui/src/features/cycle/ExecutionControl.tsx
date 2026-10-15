/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The Control section: everything on the execution page that acts on the run.
 *
 * A pending prompt leads, printed in place exactly as the prompt states it —
 * its "Run Paused" line, title, instruction, fields and actions — with no box
 * of its own and nothing clipping it, so a long form pushes the page down
 * rather than scrolling inside itself. There is usually one; any others wait
 * as one line each under it and open in turn as the first is answered. The
 * step progress the runtime streams and the execution's run controls follow on
 * one row. The section exists only while live: a file has nothing to command.
 */

/* === External Imports === */

import { useMemo } from "react";

/* === Local Imports === */

import { useExecutionProgress } from "@/app/live/use-execution-progress";
import { PromptPanel } from "@/components/PromptPanel";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { type PromptValue, promptResponseValues } from "@/lib/prompts";
import { recordLevelLabel, recordLevelVariant } from "@/lib/record-display";
import type { PromptField, PromptRequest } from "@/lib/types";
import { CycleExecutionControls } from "./CycleExecutionControls";
import { type CycleSelection, progressStepLabel, STATUS_PILL_CLASS } from "./cycle-workspace-model";
import { ExecutionSectionCard } from "./ExecutionSectionCard";
import { orderPendingPrompts } from "./prompt-banner-model";
import { usePromptResponseMutation } from "./server-state";
import { usePromptValues } from "./use-prompt-values";

/* === Types === */

export type ExecutionControlProps = {
  cycleId: string;
  /** The running execution whose progress the section states; `null` before it starts. */
  executionId: string | null;
  /** Answerable prompts for this page, in any order. */
  prompts: PromptRequest[];
  selection: CycleSelection;
  onSelectExecution: (executionId: string) => void;
};

/* === Components === */

export function ExecutionControl({
  cycleId,
  executionId,
  prompts,
  selection,
  onSelectExecution,
}: ExecutionControlProps) {
  const ordered = useMemo(() => orderPendingPrompts(prompts), [prompts]);
  const [first, ...waiting] = ordered;
  const progress = useExecutionProgress(executionId);
  const progressShown = progress && progress.testExecutionId === executionId ? progress : null;

  return (
    <ExecutionSectionCard title="Control">
      {first ? (
        <section aria-label="Pending Prompts" aria-live="polite" className="grid min-w-0 gap-3">
          <ul className="grid min-w-0 gap-3">
            <li data-prompt-id={first.correlation_id} data-prompt-level={first.level}>
              <OpenPrompt key={first.correlation_id} prompt={first} />
            </li>
            {waiting.map((prompt) => (
              <li
                className="grid min-w-0 grid-cols-[auto_minmax(0,1fr)] items-baseline gap-2 text-sm"
                data-prompt-id={prompt.correlation_id}
                data-prompt-level={prompt.level}
                key={prompt.correlation_id}
              >
                <Badge className={STATUS_PILL_CLASS} variant={recordLevelVariant(prompt.level)}>
                  {recordLevelLabel(prompt.level)}
                </Badge>
                <span className="truncate text-muted-foreground" title={prompt.title}>
                  Waiting · {prompt.title}
                </span>
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      <div className="grid min-w-0 grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-2">
        {progressShown ? (
          <div className="grid min-w-0 gap-1.5">
            <p className="grid grid-cols-[minmax(0,1fr)_auto] items-baseline gap-2 text-xs">
              <span className="truncate">
                {progressStepLabel(progressShown.stepId, progressShown.phase)}
              </span>
              <span className="text-muted-foreground tabular-nums">
                {progressShown.completed}/{progressShown.total}
              </span>
            </p>
            <Progress
              aria-label="Execution Progress"
              value={
                progressShown.total > 0 ? (progressShown.completed / progressShown.total) * 100 : 0
              }
            />
          </div>
        ) : (
          <p className="text-sm text-muted-foreground">
            {first ? "The run waits on the prompt above." : "No step in progress."}
          </p>
        )}
        <CycleExecutionControls
          cycleId={cycleId}
          onSelectExecution={onSelectExecution}
          selection={selection}
        />
      </div>
    </ExecutionSectionCard>
  );
}

/* === Private Components === */

/** The most urgent prompt, open in place with its own draft and response. */
function OpenPrompt({ prompt }: { prompt: PromptRequest }) {
  const responseMutation = usePromptResponseMutation();
  const [values, setValues] = usePromptValues(prompt);

  const submit = (action: string) =>
    responseMutation.mutate({
      action,
      correlationId: prompt.correlation_id,
      values: promptResponseValues(prompt, values),
    });
  const changeValue = (field: PromptField, value: PromptValue) =>
    setValues((current) => ({ ...current, [field.name]: value }));

  return (
    <div className="grid min-w-0 gap-3">
      <PromptPanel
        onAction={submit}
        onValueChange={changeValue}
        prompt={prompt}
        submittingAction={
          responseMutation.isPending ? (responseMutation.variables?.action ?? null) : null
        }
        values={values}
      />
      {responseMutation.isError ? (
        <p className="text-sm text-destructive">
          The response was not accepted. Check the values and submit again.
        </p>
      ) : null}
    </div>
  );
}
