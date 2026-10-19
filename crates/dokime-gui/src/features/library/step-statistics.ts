/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* How long each step of a template usually takes (concept 14).
 *
 * The Library is where an operator decides what to run, so the question the
 * timeline has to answer is "how long is this". It is one aggregate over the
 * lake's own step rows, not a statistic the backend keeps: a template that has
 * never passed simply has no answer, and an answer that fails to arrive is not
 * a fault the operator has to read about. */

/* === External Imports === */

import { useQuery } from "@tanstack/react-query";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import { fetchNativeStepDurations, type StepDurationStatistics } from "@/lib/api/native-query";

/* === Types === */

export type TemplateStepStatistics = StepDurationStatistics;

/* === Constants === */

/** One identity for "nothing to show", so a pending render is not a new object. */
const NO_STATISTICS: TemplateStepStatistics = {};

/* The units the timeline speaks, coarsest last: seconds or minutes only, so a
   sub-second step reads "0.85 s" beside a "12 s" neighbour instead of switching
   to milliseconds. A duration is stated in the first unit it fits. */
const DURATION_SCALES: { divisor: number; limit: number; unit: string }[] = [
  { divisor: 1_000, limit: 60_000, unit: "s" },
  { divisor: 60_000, limit: Number.POSITIVE_INFINITY, unit: "min" },
];

/** Two significant digits: the timeline says the order, not the measurement. */
const SIGNIFICANT_DIGITS = 2;

/* === Query Keys === */

export const stepStatisticsQueryKeys = {
  all: ["library", "step-statistics"] as const,
  template: (templateId: string) => [...stepStatisticsQueryKeys.all, templateId] as const,
};

/* === Hooks === */

/**
 * The step statistics of one template, keyed by the step's declaration id.
 *
 * Empty while the query is in flight and empty when it fails: a missing mean
 * is an absent line under a label, never an error state the operator has to
 * clear before they can configure the template.
 */
export function useTemplateStepStatistics(templateId: string): TemplateStepStatistics {
  const session = useNativeSession();
  const query = useQuery({
    queryFn: () => fetchNativeStepDurations(session.identity, templateId),
    queryKey: [...stepStatisticsQueryKeys.template(templateId), session.identity.generation],
    staleTime: 60_000,
  });
  return query.data ?? NO_STATISTICS;
}

/* === Public Functions === */

/**
 * A mean duration as the timeline prints it: at most two significant digits
 * and the unit it fits.
 *
 * Rounding can push a value past its unit ("60 s" is a minute), so the scale
 * is chosen again after rounding rather than before it.
 */
export function formatMeanDuration(meanMs: number): string {
  if (!Number.isFinite(meanMs) || meanMs < 0) {
    return "";
  }
  for (const [index, scale] of DURATION_SCALES.entries()) {
    if (meanMs >= scale.limit) continue;
    const rounded = Number((meanMs / scale.divisor).toPrecision(SIGNIFICANT_DIGITS));
    const next = DURATION_SCALES[index + 1];
    if (next && rounded * scale.divisor >= scale.limit) continue;
    return `${rounded} ${scale.unit}`;
  }
  // Unreachable: the coarsest scale has no limit and no successor to fall to.
  return `${meanMs / 1_000} s`;
}

/**
 * The line to print under each step label, keyed the same way the timeline
 * keys its nodes. A step no execution has passed yet is simply absent.
 */
export function stepDurationLabels(statistics: TemplateStepStatistics): Record<string, string> {
  return Object.fromEntries(
    Object.entries(statistics).flatMap(([stepId, statistic]) => {
      if (statistic.sampleCount <= 0) return [];
      const label = formatMeanDuration(statistic.meanMs);
      return label ? [[stepId, label]] : [];
    }),
  );
}
