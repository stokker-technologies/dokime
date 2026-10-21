/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/*
 * How pending prompts are ranked and coloured wherever they are counted, and
 * how a node's prompt queue reads.
 *
 * The queue answers in the order the run asked: oldest first, the first one
 * open, the rest waiting read-only below it, and the answered ones collapsed
 * into one row each beneath the queue, read back from the prompt's own
 * request and response records.
 *
 * The banner, the cycle header and the navigation entry read the same order
 * and the same tone, so the most urgent question is the first one an operator
 * meets everywhere. The level scale is the record severity scale, and the tone
 * follows the contract's own `tone_for`: Info is the info tone, not the muted
 * neutral a record row falls back to, because a prompt always wants an answer.
 * The module imports types only, so a node test can load it directly.
 */

/* === Local Imports === */

import type { HubRecord, Severity as PromptLevel } from "@/lib/types";

/* === Types === */

export type PromptLevelTone = "danger" | "warning" | "info" | "muted";

/** The fields a prompt needs to be ranked; `PromptRequest` satisfies it. */
export type RankedPrompt = {
  correlation_id: string;
  created_at: string;
  level: PromptLevel;
};

/** The owner fields that decide whether a prompt can still be answered. */
export type ScopedPrompt = {
  scope?: { test_execution_id?: string | null } | null;
};

/** What the workspace knows about the prompt owners it shows. */
export type PromptOwners = {
  /** Whether the cycle has ended: completed, failed, stopped or interrupted. */
  cycleEnded: boolean;
  /** Whether the named execution still runs; unknown executions do not. */
  executionActive: (executionId: string) => boolean;
};

/** One answered prompt, as the queue's history row prints it. */
export type AnsweredPrompt = {
  promptId: string;
  question: string;
  answer: string;
  at: string;
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

/**
 * The prompts an operator can still answer here.
 *
 * The runtime settles a prompt in the transaction that ends its owner, so the
 * pending list already omits it; this is the same rule held at the edge, so a
 * list read a moment before the owner ended never offers Respond. An
 * execution's prompt lives while that execution runs, a cycle's until the
 * cycle ends.
 */
export function answerablePrompts<T extends ScopedPrompt>(prompts: readonly T[], owners: PromptOwners): T[] {
  return prompts.filter((prompt) => {
    const executionId = prompt.scope?.test_execution_id;
    return executionId ? owners.executionActive(executionId) : !owners.cycleEnded;
  });
}

/** The queue's order: the run asked the oldest first, so it is answered first. */
export function queuePrompts<T extends RankedPrompt>(prompts: readonly T[]): T[] {
  return [...prompts].sort(
    (left, right) => createdAtMs(left) - createdAtMs(right) || left.correlation_id.localeCompare(right.correlation_id),
  );
}

/**
 * The answered prompts among `records`, oldest first: each response record
 * joined to its request record for the question and the field labels. A
 * response states its values, or, without any, the action it took.
 */
export function answeredPrompts(records: readonly Pick<HubRecord, "record_type" | "details" | "timestamp">[]): AnsweredPrompt[] {
  const requests = new Map<string, Pick<HubRecord, "details">>();
  for (const record of records) {
    if (record.record_type.startsWith("prompt.request.")) requests.set(String(record.details.prompt_id ?? ""), record);
  }
  return records
    .filter((record) => record.record_type.startsWith("prompt.response."))
    .map((record) => {
      const promptId = String(record.details.prompt_id ?? "");
      const request = requests.get(promptId);
      return {
        answer: responseText(record, request?.details.fields),
        at: record.timestamp,
        promptId,
        question: typeof request?.details.title === "string" ? request.details.title : promptId,
      };
    })
    .sort((left, right) => Date.parse(left.at) - Date.parse(right.at));
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

function responseText(record: Pick<HubRecord, "record_type" | "details">, fields: unknown): string {
  if (record.record_type === "prompt.response.expired") return "Expired";
  if (record.record_type === "prompt.response.cancelled") return "Cancelled";
  const labels = new Map(
    (Array.isArray(fields) ? fields : []).flatMap((field) =>
      isObject(field) && typeof field.name === "string"
        ? [[field.name, typeof field.label === "string" && field.label ? field.label : field.name] as const]
        : [],
    ),
  );
  const values = record.details.values;
  const entries = isObject(values) ? Object.entries(values) : [];
  if (entries.length) return entries.map(([name, value]) => `${labels.get(name) ?? name}: ${answerValue(value)}`).join(" · ");
  const action = String(record.details.action ?? "");
  return action ? action.charAt(0).toUpperCase() + action.slice(1).replaceAll(/[_-]+/g, " ") : "Answered";
}

function answerValue(value: unknown): string {
  if (typeof value === "boolean") return value ? "Yes" : "No";
  if (value === null || value === undefined) return "-";
  return typeof value === "object" ? JSON.stringify(value) : String(value);
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
