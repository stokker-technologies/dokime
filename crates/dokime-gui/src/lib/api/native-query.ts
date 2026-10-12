/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native typed-query adapters.
 *
 * `Query.Aggregate` is one generalized RPC over the lake, so what a caller
 * gets back is a table: named columns and untyped rows. This module is where
 * that table becomes a projection with names of its own, which is why no
 * caller has to know that "mean_ms" was column three. */

/* === External Imports === */

import { create } from "@bufbuild/protobuf";

/* === Local Imports === */

import * as rpc from "@/lib/rpc/generated/client";
import {
  AggregateFunction,
  type AggregateReply,
  QueryAggregateSchema,
  QueryFilterOperator,
  QueryFilterSchema,
  QuerySource,
} from "@/lib/rpc/generated/dokime/v1/query_pb";
import type { Value } from "@/lib/rpc/generated/dokime/v1/value_pb";
import type { SessionIdentity } from "@/lib/rpc/transport";
import { jsonToNativeValue, NativeApiLimitError, safeNumber } from "./native-values";

/* === Types === */

/** What the lake knows about one step of one template, in the GUI's terms. */
export type StepDurationStatistic = {
  meanMs: number;
  sampleCount: number;
};

/** Keyed by the step's declaration identity, which is what the catalog names. */
export type StepDurationStatistics = Record<string, StepDurationStatistic>;

/* === Constants === */

/* The widened STEPS source joins its execution and that execution's frozen
   case revision, so the template and the execution's verdict are columns of
   the step row. The names are the lake's, not the protocol's. */
const STEP_COLUMN = "identifier";
const TEMPLATE_COLUMN = "template_identifier";
const STATUS_COLUMN = "execution_status";
const DURATION_COLUMN = "duration_ms";

/* Only a passed execution is evidence of how long a step takes; a failed or
   interrupted one measures the failure. `ExecutionStatus` serializes in
   snake_case, so the stored text is lower case. */
const PASSED_STATUS = "passed";

const SAMPLE_COUNT_ALIAS = "sample_count";
const MEAN_MS_ALIAS = "mean_ms";

/* === Public API === */

/**
 * The mean duration of each step of one template over its passed executions.
 *
 * Counting `duration_ms` rather than the rows means a step still running, and
 * therefore without a duration, is not counted as a sample of itself.
 */
export async function fetchNativeStepDurations(
  session: SessionIdentity,
  templateId: string,
): Promise<StepDurationStatistics> {
  const reply = await rpc.queryAggregate(session, {
    aggregates: [
      create(QueryAggregateSchema, {
        alias: SAMPLE_COUNT_ALIAS,
        column: DURATION_COLUMN,
        function: AggregateFunction.COUNT,
      }),
      create(QueryAggregateSchema, {
        alias: MEAN_MS_ALIAS,
        column: DURATION_COLUMN,
        function: AggregateFunction.MEAN,
      }),
    ],
    filters: [
      create(QueryFilterSchema, {
        column: TEMPLATE_COLUMN,
        op: QueryFilterOperator.EQ,
        values: [jsonToNativeValue(templateId)],
      }),
      create(QueryFilterSchema, {
        column: STATUS_COLUMN,
        op: QueryFilterOperator.EQ,
        values: [jsonToNativeValue(PASSED_STATUS)],
      }),
    ],
    groupBy: [STEP_COLUMN],
    source: QuerySource.STEPS,
  });
  return mapStepDurations(reply);
}

/* === Mapping === */

/**
 * Read the aggregate table by column name.
 *
 * The reply states its own columns, and a query that grows a column would
 * silently shift every value if this read them by position.
 */
export function mapStepDurations(reply: AggregateReply): StepDurationStatistics {
  /* A truncated aggregate is a partial answer, and a partial mean is worse
     than no mean: it would print a confident number for the wrong sample. */
  if (!reply.complete) {
    throw new NativeApiLimitError("Query.Aggregate", "step statistics were truncated");
  }
  const step = columnIndex(reply, STEP_COLUMN);
  const sampleCount = columnIndex(reply, SAMPLE_COUNT_ALIAS);
  const meanMs = columnIndex(reply, MEAN_MS_ALIAS);

  const statistics: StepDurationStatistics = {};
  for (const row of reply.rows) {
    const key = textOf(row.values[step]);
    const samples = numberOf(row.values[sampleCount]);
    const mean = numberOf(row.values[meanMs]);
    // A group without a readable key, count or mean is a statistic that does
    // not exist yet, which the timeline draws as absent rather than as zero.
    if (!key || samples === null || samples <= 0 || mean === null) continue;
    statistics[key] = { meanMs: mean, sampleCount: samples };
  }
  return statistics;
}

/* === Private Functions === */

function columnIndex(reply: AggregateReply, name: string): number {
  const index = reply.columns.indexOf(name);
  if (index < 0) {
    throw new Error(`Query.Aggregate returned no column "${name}".`);
  }
  return index;
}

function textOf(value?: Value): string | null {
  return value?.kind.case === "textValue" ? value.kind.value : null;
}

function numberOf(value?: Value): number | null {
  switch (value?.kind.case) {
    case "integerValue":
    case "unsignedValue":
      return safeNumber(value.kind.value, "Query.Aggregate numeric column");
    case "numberValue":
      return Number.isFinite(value.kind.value) ? value.kind.value : null;
    default:
      return null;
  }
}
