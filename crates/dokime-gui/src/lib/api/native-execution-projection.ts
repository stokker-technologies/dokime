/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Native Response Projection
 */

/* === Imports === */

import type * as hostModel from "@generated/host/model";

import { metadataJson, nativeValueToJson } from "@/lib/host/values";

import type { Wire } from "@/lib/rpc/canonical";

import type { StepExecutionRecord, TestCase, TestExecution } from "@/lib/types";

import { ExecutionState, ReportState } from "@generated/host/model";

import { mapCriterionStatus, mapStepStatus, projectExecutionStatus } from "./native-planning-projection";

import { nowIso, safeNumber, timestampIso } from "@/lib/api/native-values";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type CycleSummary = Wire<NonNullable<hostModel.CyclesGetResponse["cycle"]>>;

type ExecutionDetail = Wire<NonNullable<hostModel.ExecutionsGetResponse["execution"]>>;

type ExecutionStep = Wire<hostModel.ExecutionsStepsResponse["steps"][number]>;

type StepCriterion = ExecutionStep["criteria"][number];

type Value = NonNullable<Wire<hostModel.ExecutionsParametersResponse["parameters"][number]>["value"]>;

export type NativeExecution = {
  detail: ExecutionDetail;
  inputs: { name: string; value?: Value }[];
  outputs: { name: string; value?: Value }[];
  steps: { step: ExecutionStep; criteria: StepCriterion[] }[];
};

/* === Classes === */

/* === Public Functions === */

export function mapNativeExecution(
  native: NativeExecution,
  cases: Pick<TestCase, "case_id" | "template_id">[],
  summary: CycleSummary,
): TestExecution {
  const detail = native.detail;
  const item = detail.summary;
  if (!item) {
    throw new Error("The host omitted the execution summary. Reconnect to the host and reload the execution.");
  }
  const testCase = cases.find((candidate) => candidate.case_id === item.caseId);
  const templateId = detail.admittedTemplate?.identifier ?? testCase?.template_id ?? "";
  const fallback = timestampIso(summary.updatedAt) ?? nowIso();
  const startedAt = timestampIso(detail.startedAt);
  return {
    controls: detail.controls,
    archived: item.archived,
    sealed: item.state === ExecutionState.SEALED,
    report: detail.report
      ? {
          state: detail.report.state === ReportState.READY ? "ready" : detail.report.state === ReportState.FAILED ? "failed" : "pending",
          failure: detail.report.failure ?? null,
        }
      : null,
    test_execution_id: item.executionId,
    test_cycle_id: detail.cycleId,
    queue_item_id: item.caseId,
    case_id: item.caseId,
    template_id: templateId,
    attempt_index: safeNumber(item.ordinal, "execution ordinal"),
    invalidation: null,
    status: projectExecutionStatus(item.state, detail.outcome ?? item.outcome, detail.controls, detail.liveState),
    started_at: startedAt,
    finished_at: timestampIso(detail.finishedAt),
    parameters: Object.fromEntries(
      [...native.inputs, ...native.outputs].flatMap((parameter) => {
        const value = nativeValueToJson(parameter.value);
        return value === undefined ? [] : [[parameter.name, value]];
      }),
    ),
    metadata: {
      ...(detail.outcome ? { outcome: detail.outcome.attribution.case, reason: detail.outcome.attribution.value?.reason ?? "" } : {}),
    },
    steps: native.steps.flatMap(({ step, criteria }) => [
      mapStepRecord(step, criteria, detail, fallback),
      ...criteria.map((criterion) => mapCriterionRecord(criterion, detail, step, fallback)),
    ]),
  };
}

/* === Private Functions === */

function mapStepRecord(step: ExecutionStep, _criteria: StepCriterion[], detail: ExecutionDetail, fallback: string): StepExecutionRecord {
  const startedAt = timestampIso(step.startedAt) ?? fallback;
  const finishedAt = timestampIso(step.finishedAt) ?? fallback;
  const status = mapStepStatus(step.status);
  return {
    record_type: "step",
    test_execution_id: detail.summary?.executionId ?? "",
    test_cycle_id: detail.cycleId,
    queue_item_id: detail.summary?.caseId ?? "",
    case_id: detail.summary?.caseId ?? "",
    template_id: detail.admittedTemplate?.identifier ?? "",
    step_id: step.stepId,
    method_name: step.metadata?.identifier || step.stepId,
    order: safeNumber(step.ordinal, "step ordinal"),
    status,
    result: {
      status,
      details: {
        measurement_count: safeNumber(step.measurementCount, "measurement count"),
        criterion_count: safeNumber(step.criterionCount, "criterion count"),
      },
    },
    started_at: startedAt,
    finished_at: finishedAt,
    duration_s: Math.max(0, (Date.parse(finishedAt) - Date.parse(startedAt)) / 1000),
    metadata: metadataJson(step.metadata),
    detail: step.detail
      ? {
          message: step.detail.message,
          chain: [...step.detail.chain],
          location: step.detail.location,
          span_trace: [...step.detail.spanTrace],
          backtrace: step.detail.backtrace,
        }
      : null,
  };
}

function mapCriterionRecord(criterion: StepCriterion, detail: ExecutionDetail, step: ExecutionStep, fallback: string): StepExecutionRecord {
  const status = mapCriterionStatus(criterion.status);
  return {
    record_type: "criterion",
    test_execution_id: detail.summary?.executionId ?? "",
    test_cycle_id: detail.cycleId,
    queue_item_id: detail.summary?.caseId ?? "",
    case_id: detail.summary?.caseId ?? "",
    template_id: detail.admittedTemplate?.identifier ?? "",
    step_id: criterion.criterionId,
    method_name: criterion.metadata?.identifier || criterion.criterionId,
    order: safeNumber(step.ordinal, "step ordinal") * 100 + safeNumber(criterion.ordinal, "criterion ordinal"),
    status,
    result: {
      status,
      details: {
        ...(criterion.message ? { message: criterion.message } : {}),
        ...(criterion.observed ? { observed: nativeValueToJson(criterion.observed) } : {}),
      },
    },
    started_at: fallback,
    finished_at: fallback,
    duration_s: 0,
    metadata: metadataJson(criterion.metadata),
  };
}

/* === Entry Point === */
