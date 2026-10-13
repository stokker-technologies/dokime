/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The questions this cycle is waiting on, as the first thing under the header.
 *
 * Every pending prompt is a card in one grid, most urgent first, coloured by
 * the level its author declared on the record severity scale. A card states
 * the level, the kind, how long it has waited, the title and the instruction,
 * and opens its response form through the existing `PromptPanel`; it leaves
 * only when the prompt is answered. `usePromptValues` keys the draft by
 * correlation id, so a refresh of the pending list never discards what the
 * operator has typed.
 *
 * The banner exists only while the cycle is live. The answered history stays
 * where it always was: the request record in the timeline and the sealed
 * report, whose level pill carries the same declared level.
 */

/* === External Imports === */

import { BellRing } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

/* === Local Imports === */

import { PromptPanel } from "@/components/PromptPanel";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { type PromptValue, promptResponseValues } from "@/lib/prompts";
import {
  recordLevelLabel,
  recordLevelVariant,
  recordToneRowClass,
  recordToneShellClass,
} from "@/lib/record-display";
import { relativeAgeLabel } from "@/lib/time-format";
import type { PromptField, PromptRequest } from "@/lib/types";
import { STATUS_PILL_CLASS } from "./cycle-workspace-model";
import {
  orderPendingPrompts,
  pendingPromptCountLabel,
  promptLevelTone,
} from "./prompt-banner-model";
import { usePromptResponseMutation } from "./server-state";
import { usePromptValues } from "./use-prompt-values";

/* === Types === */

export type CyclePromptBannerProps = {
  prompts: PromptRequest[];
};

type PromptCardProps = {
  now: number;
  onToggle: () => void;
  open: boolean;
  prompt: PromptRequest;
};

/* === Constants === */

/** Ages read in seconds while young, so the clock ticks once a second. */
const AGE_TICK_MS = 1_000;

/* === Components === */

export function CyclePromptBanner({ prompts }: CyclePromptBannerProps) {
  const [openPromptId, setOpenPromptId] = useState<string | null>(null);
  const responseMutation = usePromptResponseMutation();
  const ordered = useMemo(() => orderPendingPrompts(prompts), [prompts]);
  const openPrompt = ordered.find((prompt) => prompt.correlation_id === openPromptId) ?? undefined;
  const [values, setValues] = usePromptValues(openPrompt);
  const now = useAgeClock(ordered.length > 0);

  useEffect(() => {
    if (openPromptId && !prompts.some((prompt) => prompt.correlation_id === openPromptId)) {
      setOpenPromptId(null);
    }
  }, [openPromptId, prompts]);

  if (ordered.length === 0) return null;

  const submit = (action: string) => {
    if (!openPrompt) return;
    responseMutation.mutate({
      action,
      correlationId: openPrompt.correlation_id,
      values: promptResponseValues(openPrompt, values),
    });
  };

  const changeValue = (field: PromptField, value: PromptValue) =>
    setValues((current) => ({ ...current, [field.name]: value }));

  return (
    <section
      aria-label="Pending Prompts"
      className="grid max-h-[50vh] gap-3 overflow-y-auto border-b bg-background px-[var(--shell-inset)] py-3"
    >
      <h2 className="flex items-center gap-2 font-heading text-sm leading-6 font-semibold">
        <BellRing aria-hidden className="size-4" />
        Operator Input Needed
        <span className="font-sans font-normal text-muted-foreground">
          · {pendingPromptCountLabel(ordered.length)}
        </span>
      </h2>

      <ul
        aria-live="polite"
        className="grid grid-cols-[repeat(auto-fill,minmax(min(100%,22rem),1fr))] gap-2"
      >
        {ordered.map((prompt) => (
          <PromptCard
            key={prompt.correlation_id}
            now={now}
            onToggle={() =>
              setOpenPromptId(prompt.correlation_id === openPromptId ? null : prompt.correlation_id)
            }
            open={prompt.correlation_id === openPromptId}
            prompt={prompt}
          />
        ))}
      </ul>

      {openPrompt ? (
        <div className="grid gap-3">
          <PromptPanel
            onAction={submit}
            onValueChange={changeValue}
            prompt={openPrompt}
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
      ) : null}
    </section>
  );
}

/* === Private Components === */

/**
 * One pending prompt on a fixed four-row grid: pills and age, title,
 * instruction, action. The tone colours the card's ground and its leading
 * rule; the words stay in the foreground ink so a red card is still readable.
 */
function PromptCard({ now, onToggle, open, prompt }: PromptCardProps) {
  const tone = promptLevelTone(prompt.level);

  return (
    <li
      className={recordToneShellClass(
        tone,
        recordToneRowClass(
          tone,
          "grid grid-rows-[auto_auto_1fr_auto] gap-1.5 rounded-lg border-l-4 p-3",
        ),
      )}
      data-prompt-id={prompt.correlation_id}
      data-prompt-level={prompt.level}
    >
      <div className="grid grid-cols-[auto_auto_minmax(0,1fr)_auto] items-center gap-2">
        <Badge className={STATUS_PILL_CLASS} variant={recordLevelVariant(prompt.level)}>
          {recordLevelLabel(prompt.level)}
        </Badge>
        <Badge className={STATUS_PILL_CLASS} variant="outline">
          {promptKindLabel(prompt.kind)}
        </Badge>
        <span />
        <time
          className="text-xs leading-6 text-muted-foreground tabular-nums"
          dateTime={prompt.created_at}
          title={prompt.created_at}
        >
          {relativeAgeLabel(prompt.created_at, now)}
        </time>
      </div>
      <p
        className="line-clamp-2 font-heading text-base leading-6 font-semibold text-foreground"
        title={prompt.title}
      >
        {prompt.title}
      </p>
      <p
        className="line-clamp-3 text-sm leading-5 text-foreground/80"
        title={prompt.message ?? undefined}
      >
        {prompt.message ?? ""}
      </p>
      <div className="grid justify-end">
        <Button
          aria-expanded={open}
          onClick={onToggle}
          size="sm"
          variant={open ? "secondary" : "default"}
        >
          {open ? "Hide" : "Respond"}
        </Button>
      </div>
    </li>
  );
}

/* === Private Functions === */

/** A clock for the ages on the cards, running only while a card is shown. */
function useAgeClock(running: boolean): number {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    if (!running) return;
    setNow(Date.now());
    const handle = window.setInterval(() => setNow(Date.now()), AGE_TICK_MS);
    return () => window.clearInterval(handle);
  }, [running]);

  return now;
}

function promptKindLabel(kind: PromptRequest["kind"]): string {
  switch (kind) {
    case "confirmation":
      return "Confirm";
    case "choice":
      return "Choice";
    case "status":
      return "Working";
    case "verification":
      return "Verify";
    default:
      return "Form";
  }
}
