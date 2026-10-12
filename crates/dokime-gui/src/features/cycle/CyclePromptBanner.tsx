/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The prompts this cycle is waiting on, in a bounded banner under the header.
 *
 * The banner names every pending prompt and opens one response form at a time
 * through the existing `PromptPanel`. `usePromptValues` keys the draft by
 * correlation id, so a refresh of the pending list never discards what the
 * operator has typed.
 */

/* === External Imports === */

import { useEffect, useState } from "react";

/* === Local Imports === */

import { PromptPanel } from "@/components/PromptPanel";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { type PromptValue, promptResponseValues } from "@/lib/prompts";
import type { PromptField, PromptRequest } from "@/lib/types";
import { cn } from "@/lib/utils";
import { STATUS_PILL_CLASS } from "./cycle-workspace-model";
import { usePromptResponseMutation } from "./server-state";
import { usePromptValues } from "./use-prompt-values";

/* === Types === */

export type CyclePromptBannerProps = {
  prompts: PromptRequest[];
};

/* === Components === */

export function CyclePromptBanner({ prompts }: CyclePromptBannerProps) {
  const [openPromptId, setOpenPromptId] = useState<string | null>(null);
  const responseMutation = usePromptResponseMutation();
  const openPrompt = prompts.find((prompt) => prompt.correlation_id === openPromptId) ?? undefined;
  const [values, setValues] = usePromptValues(openPrompt);

  useEffect(() => {
    if (openPromptId && !prompts.some((prompt) => prompt.correlation_id === openPromptId)) {
      setOpenPromptId(null);
    }
  }, [openPromptId, prompts]);

  if (prompts.length === 0) return null;

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
      className="grid max-h-96 gap-3 overflow-y-auto border-b bg-muted/40 px-4 py-3"
    >
      <ul className="grid gap-0.5">
        {prompts.map((prompt) => {
          const open = prompt.correlation_id === openPromptId;
          return (
            <li
              className={cn(
                "grid grid-cols-[minmax(0,1fr)_auto_auto] items-center gap-3 rounded-lg px-2 py-1.5",
                open ? "bg-background" : null,
              )}
              key={prompt.correlation_id}
            >
              <span className="min-w-0 truncate text-sm leading-6" title={prompt.title}>
                {prompt.title}
                {prompt.message ? (
                  <span className="text-muted-foreground"> — {prompt.message}</span>
                ) : null}
              </span>
              <Badge className={STATUS_PILL_CLASS} variant="warning">
                {promptKindLabel(prompt.kind)}
              </Badge>
              <Button
                aria-expanded={open}
                onClick={() => setOpenPromptId(open ? null : prompt.correlation_id)}
                size="sm"
                variant={open ? "secondary" : "outline"}
              >
                {open ? "Hide" : "Respond"}
              </Button>
            </li>
          );
        })}
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

/* === Private Functions === */

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
