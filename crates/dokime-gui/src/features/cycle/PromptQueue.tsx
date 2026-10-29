/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * The prompt queue: the questions a node waits on, answered in the order the
 * run asked them.
 *
 * A prompt has one home, the node that asked it: an execution's queue sits on
 * its page directly under the Control card, and a prompt the cycle itself
 * asked, which no execution owns, on the cycle page. Each pending prompt is a
 * self-contained card — the question, its context (the step that asked, label
 * first and id muted after it), the answer control and its actions. The
 * oldest is open and takes focus; the later ones follow below, read-only
 * until the one above them is answered. Answered prompts collapse into one row
 * each beneath the queue — question, answer, time — read back from the
 * prompt's own records, so the history stays without the clutter; a frozen
 * report shows that history alone. `usePromptValues` keys each draft by
 * correlation id, so a refresh of the pending list never discards what the
 * operator typed.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useEffect, useMemo, useRef } from "react";

// Local Imports
import { stepRecordLabel } from "@/components/document/StepRecordsCard";
import { NotificationMessage } from "@/components/NotificationMessage";
import { PromptPanel } from "@/components/PromptPanel";
import { Badge } from "@/components/ui/badge";
import { type PromptValue, type PromptValues, promptResponseValues } from "@/lib/prompts";
import { recordLevelLabel, recordLevelVariant } from "@/lib/record-display";
import type { HubRecord, PromptField, PromptRequest, StepExecutionRecord } from "@/lib/types";
import { STATUS_PILL_CLASS } from "./cycle-workspace-model";
import { ExecutionSectionCard } from "./ExecutionSectionCard";
import { PromptAnswers } from "./PromptHistory";
import { answeredPrompts, pendingPromptCountLabel, queuePrompts } from "./prompt-banner-model";
import { usePromptResponseMutation } from "./server-state";
import { usePromptValues } from "./use-prompt-values";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type PromptQueueProps = {
  /** The answerable prompts this node owns, in any order; none in a file. */
  prompts: PromptRequest[];
  /** The node's records, which hold the answered prompts' history. */
  records: readonly HubRecord[];
  /** The node's step records, which name the step a prompt came from. */
  steps?: readonly StepExecutionRecord[];
};

/* === Components === */

export function PromptQueue({ prompts, records, steps = [] }: PromptQueueProps) {
  const queue = useMemo(() => queuePrompts(prompts), [prompts]);
  const answered = useMemo(() => answeredPrompts(records), [records]);
  if (!queue.length && !answered.length) return null;
  const [first, ...waiting] = queue;

  return (
    <ExecutionSectionCard
      action={queue.length ? <span className="text-xs text-muted-foreground">{pendingPromptCountLabel(queue.length)}</span> : null}
      title="Prompts"
    >
      {first ? (
        <ol aria-label="Pending Prompts" aria-live="polite" className="grid min-w-0 gap-3">
          <PromptQueueCard open prompt={first} steps={steps} />
          {waiting.map((prompt) => (
            <PromptQueueCard key={prompt.correlation_id} open={false} prompt={prompt} steps={steps} />
          ))}
        </ol>
      ) : null}
      <PromptAnswers answered={answered} />
    </ExecutionSectionCard>
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */

/** One pending prompt: open and focused at the head of the queue, read-only behind it. */
function PromptQueueCard({ open, prompt, steps }: { open: boolean; prompt: PromptRequest; steps: readonly StepExecutionRecord[] }) {
  const card = useRef<HTMLLIElement>(null);
  const step = steps.find((record) => record.record_type === "step" && record.step_id === prompt.scope?.step_id);
  useEffect(() => {
    if (open) card.current?.focus({ preventScroll: false });
  }, [open]);

  return (
    <li
      className={`grid min-w-0 gap-2 rounded-lg border p-3 outline-none focus-visible:ring-2 focus-visible:ring-ring ${open ? "bg-background" : "bg-muted/40"}`}
      data-prompt-id={prompt.correlation_id}
      data-prompt-level={prompt.level}
      data-prompt-state={open ? "open" : "waiting"}
      ref={card}
      tabIndex={-1}
    >
      <p className="grid min-w-0 grid-cols-[auto_minmax(0,1fr)] items-baseline gap-2 text-xs text-muted-foreground">
        <Badge className={STATUS_PILL_CLASS} variant={recordLevelVariant(prompt.level)}>
          {recordLevelLabel(prompt.level)}
        </Badge>
        <span className="min-w-0 [overflow-wrap:anywhere]">
          {step ? stepRecordLabel(step, false) : prompt.scope?.step_id ? null : "Asked by the run"}
          {prompt.scope?.step_id ? <span className="ml-2 font-mono">{prompt.scope.step_id}</span> : null}
        </span>
      </p>
      {open ? (
        <OpenPrompt prompt={prompt} />
      ) : (
        <div className="grid min-w-0 gap-1">
          <p className="font-heading text-sm font-semibold [overflow-wrap:anywhere]">{prompt.title}</p>
          {prompt.message ? <p className="text-sm text-muted-foreground [overflow-wrap:anywhere]">{prompt.message}</p> : null}
          <p className="text-xs text-muted-foreground">Opens once the prompt above is answered.</p>
        </div>
      )}
    </li>
  );
}

/** The head of the queue, open in place with its own draft and response. */
function OpenPrompt({ prompt }: { prompt: PromptRequest }) {
  const responseMutation = usePromptResponseMutation();
  const [values, setValues] = usePromptValues(prompt);

  const submit = (action: string, overrides?: PromptValues) =>
    responseMutation.mutate({
      action,
      correlationId: prompt.correlation_id,
      values: promptResponseValues(prompt, overrides ?? values),
    });
  const changeValue = (field: PromptField, value: PromptValue) => setValues((current) => ({ ...current, [field.name]: value }));

  return (
    <div className="grid min-w-0 gap-3">
      <PromptPanel
        onAction={submit}
        onValueChange={changeValue}
        prompt={prompt}
        submittingAction={responseMutation.isPending ? (responseMutation.variables?.action ?? null) : null}
        values={values}
      />
      {responseMutation.isError ? (
        <NotificationMessage source="Prompts" title="Response Not Accepted">
          Check the values and submit again.
        </NotificationMessage>
      ) : null}
    </div>
  );
}
