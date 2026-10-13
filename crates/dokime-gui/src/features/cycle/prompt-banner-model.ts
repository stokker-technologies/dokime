/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * How pending prompts are ranked and coloured wherever they are counted.
 *
 * The banner, the cycle header and the navigation entry read the same order
 * and the same tone, so the most urgent question is the first one an operator
 * meets everywhere. The level scale is the record severity scale, and the tone
 * follows the contract's own `tone_for`: Info is the info tone, not the muted
 * neutral a record row falls back to, because a prompt always wants an answer.
 * The module imports types only, so a node test can load it directly.
 */

/* === Local Imports === */

import type { Severity as PromptLevel } from "@/lib/types";

/* === Types === */

export type PromptLevelTone = "danger" | "warning" | "info" | "muted";

/** The fields a prompt needs to be ranked; `PromptRequest` satisfies it. */
export type RankedPrompt = {
  correlation_id: string;
  created_at: string;
  level: PromptLevel;
};

/* === Constants === */

const LEVEL_RANK: Record<PromptLevel, number> = {
  error: 3,
  warning: 2,
  info: 1,
  debug: 0,
};

/* === Public Functions === */

/**
 * Most urgent first; within a level the oldest first, because it has waited
 * longest; then by id, so two prompts asked in the same instant never swap.
 */
export function orderPendingPrompts<T extends RankedPrompt>(prompts: readonly T[]): T[] {
  return [...prompts].sort(
    (left, right) =>
      LEVEL_RANK[right.level] - LEVEL_RANK[left.level] ||
      createdAtMs(left) - createdAtMs(right) ||
      left.correlation_id.localeCompare(right.correlation_id),
  );
}

/** The level that colours a count of these prompts, or `null` for none. */
export function highestPromptLevel(prompts: readonly RankedPrompt[]): PromptLevel | null {
  return orderPendingPrompts(prompts)[0]?.level ?? null;
}

export function promptLevelTone(level: PromptLevel): PromptLevelTone {
  if (level === "error") return "danger";
  if (level === "warning") return "warning";
  if (level === "debug") return "muted";
  return "info";
}

export function pendingPromptCountLabel(count: number): string {
  return `${count} ${count === 1 ? "prompt" : "prompts"} waiting`;
}

/* === Private Functions === */

/** An unreadable timestamp sorts last within its level rather than first. */
function createdAtMs(prompt: RankedPrompt): number {
  const parsed = Date.parse(prompt.created_at);
  return Number.isFinite(parsed) ? parsed : Number.POSITIVE_INFINITY;
}
