/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { absoluteTimestampLabel } from "@/lib/time-format";
import type { HubRecord } from "@/lib/types";
import { ExecutionSectionCard } from "./ExecutionSectionCard";
import { answeredPrompts } from "./prompt-banner-model";

export function PromptHistory({ records }: { records: readonly HubRecord[] }) {
  const answered = answeredPrompts(records);
  if (!answered.length) return null;
  return (
    <ExecutionSectionCard title="Prompts">
      <PromptAnswers answered={answered} />
    </ExecutionSectionCard>
  );
}

export function PromptAnswers({ answered }: { answered: ReturnType<typeof answeredPrompts> }) {
  if (!answered.length) return null;
  return (
    <ol aria-label="Answered Prompts" className="grid min-w-0 grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] gap-x-4 border-t pt-2 text-xs">
      {answered.map((row) => (
        <li className="col-span-3 grid grid-cols-subgrid items-baseline py-1" data-answered-prompt={row.promptId} key={row.promptId}>
          <span className="min-w-0 font-medium [overflow-wrap:anywhere]">{row.question}</span>
          <span className="min-w-0 text-muted-foreground [overflow-wrap:anywhere]">{row.answer}</span>
          <time className="text-muted-foreground tabular-nums" dateTime={row.at}>
            {absoluteTimestampLabel(row.at)}
          </time>
        </li>
      ))}
    </ol>
  );
}
