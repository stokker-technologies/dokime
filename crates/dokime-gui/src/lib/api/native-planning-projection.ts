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

import type { TestSuiteEntry } from "@/lib/types";

import type * as hostModel from "@generated/host/model";

import {
  CaseStatus,
  CriterionStatus,
  CyclePhase,
  CycleStatus,
  ExecutionState,
  LiveExecutionState,
  ResultStatus,
  StepStatus,
  Termination,
  type ExecutionControls,
  type ExecutionOutcome,
} from "@generated/host/model";

import { metadataJson, nativeValueToJson } from "@/lib/host/values";

import type { Wire } from "@/lib/rpc/canonical";

import type {
  CaseActions,
  ExecutionQueueItem,
  StepExecutionRecord,
  SuiteTemplateEntry,
  TestCase,
  TestCycle,
  TestCyclePlan,
  TestExecution,
  TestTemplateMetadata,
} from "@/lib/types";

import { durationSeconds, nowIso, safeNumber, timestampIso } from "@/lib/api/native-values";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type CaseDetail = Wire<hostModel.CaseDetail>;

export type CaseSummary = Wire<hostModel.CaseSummary>;

export type ExecutionSummary = Wire<hostModel.ExecutionSummary>;

export type CycleSummary = Wire<hostModel.CycleSummary>;

type NativeCaseActions = Wire<hostModel.CaseActions>;

type Value = Wire<hostModel.Value>;

/* === Classes === */

/* === Public Functions === */

export function mapNativeCyclePlan(
  summary: CycleSummary,
  caseDetails: { item: CaseSummary; detail: CaseDetail; parameters: { name: string; value?: Value }[] }[],
  suites: (TestSuiteEntry | null)[],
): TestCyclePlan {
  const mappedCases = caseDetails.map((entry) => mapNativeCase(entry.item, entry.detail, entry.parameters));
  const queue = caseDetails.map((entry, index) => mapQueueItem(summary, entry.detail.summary ?? entry.item, index));
  /* The case the cycle is on, as the case projection states it: exactly one
     case can be RUNNING, and its own latest attempt is the live one. */
  const active = queue.find((item) => item.status === "running") ?? null;
  const templates = suites.flatMap((suite) => suite?.members.map((template) => mapTemplateMetadata(template)) ?? []);
  const generatedAt = timestampIso(summary.createdAt) ?? timestampIso(summary.updatedAt) ?? nowIso();
  return {
    test_cycle: {
      test_cycle_id: summary.cycleId,
      title: summary.metadata?.label || summary.cycleId,
      phase: mapCyclePhase(summary.phase),
      status: mapCycleStatus(summary.status),
      case_count: mappedCases.length,
      queue_count: mappedCases.length,
      current_test_execution_id: active?.current_test_execution_id ?? null,
      current_queue_item_id: active?.case_id ?? null,
      continue_automatically: summary.continueAutomatically,
      started_at: timestampIso(summary.startedAt),
      finished_at: summary.phase === CyclePhase.SEALED ? timestampIso(summary.updatedAt) : null,
      seal_failure: summary.sealFailure ?? null,
      rerun_of: summary.rerunOf ?? null,
      duration_s: durationSeconds(summary.startedAt, summary.updatedAt),
      duration_delta_s: null,
      metadata: metadataJson(summary.metadata),
      generated_at: generatedAt,
      archived: summary.archived,
      sealed: summary.sealed,
    },
    templates,
    cases: mappedCases,
    queue,
    generated_at: generatedAt,
  };
}

export function mapCyclePhase(phase: CycleSummary["phase"]): TestCycle["phase"] {
  switch (phase) {
    case CyclePhase.ACTIVE:
      return "active";
    case CyclePhase.SEALING:
      return "sealing";
    case CyclePhase.SEAL_FAILED:
      return "seal_failed";
    case CyclePhase.SEALED:
      return "sealed";
    default:
      throw new Error("The cycle has no container phase.");
  }
}

export function mapNativeExecutionSummary(item: ExecutionSummary, cases: TestCase[], cycle: CycleSummary): TestExecution {
  const testCase = cases.find((candidate) => candidate.case_id === item.caseId);
  return {
    controls: item.controls,
    archived: item.archived,
    sealed: item.state === ExecutionState.SEALED,
    test_execution_id: item.executionId,
    test_cycle_id: cycle.cycleId,
    queue_item_id: item.caseId,
    case_id: item.caseId,
    template_id: testCase?.template_id ?? "",
    attempt_index: safeNumber(item.ordinal, "execution ordinal"),
    invalidation: null,
    status: projectExecutionStatus(item.state, item.outcome, item.controls),
    started_at: null,
    finished_at: null,
    parameters: null,
    metadata: {},
    steps: [],
  };
}

export function mapCycleStatus(status: CycleStatus | undefined): TestCyclePlan["test_cycle"]["status"] {
  switch (status) {
    case CycleStatus.RUNNING:
      return "running";
    case CycleStatus.PAUSED:
      return "paused";
    case CycleStatus.STOPPING:
      return "stopping";
    case CycleStatus.COMPLETED:
      return "completed";
    case CycleStatus.FAILED:
    case CycleStatus.BLOCKED:
      return "failed";
    case CycleStatus.STOPPED:
      return "stopped";
    default:
      return "planned";
  }
}

/** Durable Attribution Determines the Verdict; Live Controls Only Describe Active Work. */
export function projectExecutionStatus(
  state: ExecutionState | undefined,
  outcome?: Wire<ExecutionOutcome>,
  controls?: Wire<ExecutionControls>,
  liveState?: LiveExecutionState,
): TestExecution["status"] {
  if (state === ExecutionState.OPEN) {
    if (liveState === LiveExecutionState.CLEANING_UP) {
      return "finalizing";
    }
    return controls?.requestId || (liveState !== undefined && liveState !== LiveExecutionState.QUEUED) ? "running" : "planned";
  }
  const attribution = outcome?.attribution;
  if (attribution?.case === "evaluated" || attribution?.case === "declared") {
    switch (attribution.value.status) {
      case ResultStatus.PASSED:
        return "passed";
      case ResultStatus.FAILED:
      case ResultStatus.BLOCKED:
        return "failed";
      case ResultStatus.CANCELLED:
        return "stopped";
      default:
        throw new Error("Read Execution: Missing Evaluated Verdict");
    }
  }
  if (attribution?.case === "notEvaluated") {
    switch (attribution.value.termination) {
      case Termination.NOT_RUN:
        return "not_run";
      case Termination.FAILED:
        return "failed";
      case Termination.CANCELLED:
      case Termination.STOPPED:
        return "stopped";
      case Termination.INTERRUPTED:
        return "interrupted";
      default:
        return "skipped";
    }
  }
  throw new Error("Read Execution: Closed Execution Has No Outcome Attribution");
}

export function mapCaseStatus(status: CaseStatus | undefined): ExecutionQueueItem["status"] {
  switch (status) {
    case CaseStatus.NOT_RUN:
      return "not_run";
    case CaseStatus.PLANNED:
      return "planned";
    case CaseStatus.QUEUED:
      return "queued";
    case CaseStatus.RUNNING:
      return "running";
    case CaseStatus.PASSED:
      return "passed";
    case CaseStatus.FAILED:
      return "failed";
    case CaseStatus.BLOCKED:
      return "blocked";
    case CaseStatus.CANCELLED:
    case CaseStatus.STOPPED:
      return "stopped";
    case CaseStatus.INTERRUPTED:
      return "interrupted";
    case CaseStatus.SKIPPED:
      return "skipped";
    case CaseStatus.INVALIDATED:
      return "invalidated";
    default:
      return "unknown";
  }
}

export function mapStepStatus(status: StepStatus | undefined): StepExecutionRecord["status"] {
  switch (status) {
    case StepStatus.RUNNING:
      return "running";
    case StepStatus.PASSED:
      return "passed";
    case StepStatus.FAILED:
      return "failed";
    case StepStatus.SKIPPED:
      return "skipped";
    case StepStatus.CANCELLED:
      return "stopped";
    default:
      return "pending";
  }
}

export function mapCriterionStatus(status: CriterionStatus | undefined): StepExecutionRecord["status"] {
  switch (status) {
    case CriterionStatus.PASSED:
      return "passed";
    case CriterionStatus.FAILED:
      return "failed";
    case CriterionStatus.INDETERMINATE:
      return "indeterminate";
    case CriterionStatus.SKIPPED:
      return "skipped";
    default:
      return "pending";
  }
}

export function isNativeCycleLive(status: CycleStatus): boolean {
  return [CycleStatus.RUNNING, CycleStatus.PAUSED, CycleStatus.STOPPING, CycleStatus.BLOCKED].includes(status);
}

/* === Private Functions === */

function mapNativeCase(item: CaseSummary, detail: CaseDetail, parameters: { name: string; value?: Value }[]): TestCase {
  const summary = detail.summary ?? item;
  return {
    suite_id: summary.suiteId,
    case_id: summary.caseId,
    template_id: summary.templateId,
    parameters: Object.fromEntries(
      parameters.flatMap((parameter) => {
        const value = nativeValueToJson(parameter.value);
        return value === undefined ? [] : [[parameter.name, value]];
      }),
    ),
    generation_index: safeNumber(detail.generationIndex, "case generation index"),
    label: summary.metadata?.label || summary.caseId,
    metadata: metadataJson(summary.metadata),
  };
}

function mapTemplateMetadata(template: SuiteTemplateEntry): TestTemplateMetadata {
  // Keep the original cycle-plan projection: the HTTP runtime mapper also
  // supplied no authored metadata here. The Suite Definition carries the same
  // suite-picker fields; document/record owners provide those views.
  return {
    template_id: template.template_id,
    name: template.name,
    description: template.description ?? null,
    source_file: null,
    parameters: template.parameter_definitions,
    files: template.file_definitions,
    steps: template.steps,
    measurements: [],
    criteria: [],
    initialize_hooks: [],
    finalize_hooks: [],
    interrupt_hooks: [],
    cycle_summary: null,
    metadata: template.metadata,
  };
}

/**
 * One planned case as the navigator renders it, straight off `CaseSummary`.
 *
 * Every field here is the runtime's own statement about that case. The ordinal
 * is the position at this cursor rather than an identity, which is why it is
 * passed in rather than read back.
 */
function mapQueueItem(summary: CycleSummary, item: CaseSummary, ordinal: number): ExecutionQueueItem {
  return {
    queue_item_id: item.caseId,
    test_cycle_id: summary.cycleId,
    ordinal,
    template_id: item.templateId,
    case_id: item.caseId,
    repeat_index: 0,
    status: mapCaseStatus(item.status),
    current_test_execution_id: item.latestExecutionId ?? null,
    current_test_execution_status:
      item.latestExecutionStatus === undefined ? null : projectExecutionStatus(item.latestExecutionStatus, item.latestExecutionOutcome),
    attempt_count: item.attemptCount,
    actions: mapCaseActions(item.actions),
    blocked_reason: null,
  };
}

/** Action eligibility, or `null` where the projection reports none at all. */
function mapCaseActions(actions: NativeCaseActions | undefined): CaseActions | null {
  if (!actions) {
    return null;
  }
  return {
    delete: actions.delete,
    unavailable_reasons: [...actions.unavailableReasons],
  };
}

/* === Entry Point === */
