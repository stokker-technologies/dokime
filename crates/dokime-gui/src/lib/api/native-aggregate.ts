/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The Harmos aggregate vocabulary between its document spelling and the wire.
 *
 * Documents name an aggregate by its compact display form (`"mean"`,
 * `"p95"`); the wire carries the Harmos enum message, one oneof member per
 * statistic with `Percentile` holding its `rank`. Every generated operation
 * declares its own copy of that message, so these adapters work on its shape
 * rather than on one generated type. */

/* === External Imports === */

/* === Local Imports === */

import type { Aggregate } from "@/lib/document-contracts";

/* === Types === */

type UnitMember = "Count" | "Sum" | "Mean" | "Median" | "Min" | "Max" | "Rms" | "Std";

/** The init shape of any generated Harmos aggregate message. */
export type AggregateMessage = {
  HarmosVariant: { case: UnitMember; value: Record<string, never> } | { case: "Percentile"; value: { rank: number } };
};

/** What a decoded generated aggregate message exposes. */
type DecodedAggregate =
  | {
      HarmosVariant: { case: string | undefined; value?: unknown };
    }
  | undefined;

/* === Constants === */

const UNIT_MEMBERS: Record<string, UnitMember> = {
  count: "Count",
  sum: "Sum",
  mean: "Mean",
  median: "Median",
  min: "Min",
  max: "Max",
  rms: "Rms",
  std: "Std",
};

const PERCENTILE = /^p(\d{1,3})$/;
const MAX_RANK = 100;

/* === Public API === */

/** The wire message for one document aggregate; unknown names are refused. */
export function aggregateMessage(name: string): AggregateMessage {
  const unit = UNIT_MEMBERS[name];
  if (unit) return { HarmosVariant: { case: unit, value: {} } };
  const rank = PERCENTILE.exec(name)?.[1];
  if (rank !== undefined && Number(rank) <= MAX_RANK) {
    return { HarmosVariant: { case: "Percentile", value: { rank: Number(rank) } } };
  }
  throw new Error(`Unknown aggregate "${name}".`);
}

/** The document name of a decoded wire aggregate; an absent member is `count`. */
export function aggregateName(message: DecodedAggregate): Aggregate {
  const variant = message?.HarmosVariant;
  const member = variant?.case;
  if (member === undefined) return "count";
  if (member === "Percentile") {
    const rank = (variant?.value as { rank?: number } | undefined)?.rank ?? 0;
    return `p${rank}`;
  }
  const name = Object.keys(UNIT_MEMBERS).find((key) => UNIT_MEMBERS[key] === member);
  if (!name) throw new Error(`Unknown aggregate member "${member}".`);
  return name as Aggregate;
}
