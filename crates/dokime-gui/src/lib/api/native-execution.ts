/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native execution detail and control adapters for the original routed UI. */

/* === External Imports === */

/* === Local Imports === */

import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/host/client";
import { ExecutionMode, ExecutionParameterSet, ReportState } from "@host/dokime/client/v1/executions_pb.js";
import { ExecutionState } from "@host/dokime/v1/execution_pb.js";
import { metadataJson, nativeValueToJson } from "@/lib/host/values";

type CycleSummary = Wire<NonNullable<Awaited<ReturnType<typeof rpc.cyclesGet>>["cycle"]>>;
import { required } from "@/lib/rpc/canonical";
type ExecutionDetail = Wire<NonNullable<Awaited<ReturnType<typeof rpc.executionsGet>>["execution"]>>;
type ExecutionStep = Wire<Awaited<ReturnType<typeof rpc.executionsSteps>>["steps"][number]>;
type StepCriterion = ExecutionStep["criteria"][number];
type Value = NonNullable<Wire<Awaited<ReturnType<typeof rpc.executionsParameters>>["parameters"][number]>["value"]>;

import type { SessionIdentity } from "@/lib/rpc/transport";
import type { RerunNotices } from "@/stores/rerun-notice-store";
import type { StartTestCycleResponse, StepExecutionRecord, TestCase, TestCyclePlan, TestExecution } from "@/lib/types";
import {
  ensureNativeStartAccepted,
  fetchNativeTestCyclePlan,
  listNativeCases,
  listNativeExecutions,
  mapCriterionStatus,
  mapNativeExecutionSummary,
  mapStepStatus,
  type NativeStartAdmission,
  projectExecutionStatus,
} from "./native-planning";
import { mapBounded, nowIso, safeNumber, timestampIso } from "./native-values";

export async function fetchNativeTestCycleDetail(
  session: SessionIdentity,
  cycleId: string,
): Promise<{ executions: TestExecution[]; plan: TestCyclePlan | null }> {
  const plan = await fetchNativeTestCyclePlan(session, cycleId);
  if (!plan) return { executions: [], plan: null };
  const [cycle, summaries] = await Promise.all([
    rpc.cyclesGet(session, { name: plan.test_cycle.test_cycle_id }).then((reply) => required(reply.cycle, "Get Cycle")),
    listNativeExecutions(session, plan.test_cycle.test_cycle_id),
  ]);
  return {
    executions: summaries.map((summary) => mapNativeExecutionSummary(summary, plan.cases, cycle)),
    plan,
  };
}

/** Load steps, criteria, parameters, and result data for one selected execution. */
export async function fetchNativeTestExecutionDetail(session: SessionIdentity, executionId: string): Promise<TestExecution> {
  const native = await loadNativeExecution(session, executionId);
  const cycleId = native.detail.cycleId;
  const [cycle, cases] = await Promise.all([
    rpc.cyclesGet(session, { name: cycleId }).then((reply) => required(reply.cycle, "Get Cycle")),
    listNativeCases(session, cycleId),
  ]);
  const mappedCases = cases.map((item) => ({
    suite_id: item.suiteId,
    case_id: item.caseId,
    template_id: item.templateId,
  }));
  return mapNativeExecution(native, mappedCases, cycle);
}

export async function fetchNativeTestCycleSteps(
  session: SessionIdentity,
  options: {
    cycleId: string;
    executionId?: string | null;
    queueItemId?: string | null;
    caseId?: string | null;
  },
): Promise<StepExecutionRecord[]> {
  if (options.executionId) {
    const execution = await loadNativeExecution(session, options.executionId);
    return execution.steps.flatMap(({ step, criteria }) => [
      mapStepRecord(step, criteria, execution.detail, timestampIso(execution.detail.startedAt) ?? nowIso()),
      ...criteria.map((criterion) =>
        mapCriterionRecord(criterion, execution.detail, step, timestampIso(execution.detail.startedAt) ?? nowIso()),
      ),
    ]);
  }
  const executions = await loadNativeExecutionsForPlan(session, options.cycleId);
  return executions
    .filter(
      (execution) =>
        (!options.queueItemId || execution.case_id === options.queueItemId) && (!options.caseId || execution.case_id === options.caseId),
    )
    .flatMap((execution) => execution.steps);
}

/** Admit One Existing Attempt By Its Durable Execution Identity */
export async function startNativeTestExecution(session: SessionIdentity, executionId: string): Promise<NativeStartAdmission> {
  return ensureNativeStartAccepted(
    await rpc.executionsStart(session, {
      name: executionId,
      requestId: crypto.randomUUID(),
      mode: ExecutionMode.AUTOMATIC,
    }),
  );
}

export async function pauseNativeTestExecution(session: SessionIdentity, executionId: string): Promise<void> {
  await replaceExecutions(session, executionId, { paused: true });
}

export async function resumeNativeTestExecution(session: SessionIdentity, executionId: string): Promise<void> {
  await replaceExecutions(session, executionId, { paused: false });
}

export async function stopNativeTestExecution(session: SessionIdentity, executionId: string): Promise<void> {
  await rpc.executionsStop(session, { name: executionId, requestId: crypto.randomUUID() });
}

/** Cancel an Active Execution and Retain Its Required Cleanup. */
export async function cancelNativeTestExecution(session: SessionIdentity, executionId: string): Promise<void> {
  await rpc.executionsCancel(session, { name: executionId, requestId: crypto.randomUUID() });
}

export async function sealNativeTestCycle(session: SessionIdentity, cycleId: string): Promise<void> {
  await rpc.cyclesSeal(session, { name: cycleId, requestId: crypto.randomUUID() });
}

/** Archives or unarchives a terminal cycle; its lifecycle is unchanged. */
export async function setNativeCycleArchived(session: SessionIdentity, cycleId: string, archived: boolean): Promise<void> {
  await replaceCycles(session, cycleId, { archived: archived });
}

/** Archives or unarchives a terminal execution; its lifecycle is unchanged. */
export async function setNativeExecutionArchived(session: SessionIdentity, executionId: string, archived: boolean): Promise<void> {
  await replaceExecutions(session, executionId, { archived: archived });
}

/** Renames a cycle in any lifecycle state; resolves to the label as stored. */
export async function renameNativeTestCycle(session: SessionIdentity, cycleId: string, label: string): Promise<string> {
  return required((await replaceCycles(session, cycleId, { label })).value, "Rename Cycle").label;
}

/** Renames an execution's case, every attempt of it; resolves to the stored label. */
export async function renameNativeTestExecution(session: SessionIdentity, executionId: string, label: string): Promise<string> {
  return required((await replaceExecutions(session, executionId, { label })).value, "Rename Execution").label;
}

/** The cycle's label as the backend holds it, for a lineage link to it. */
export async function fetchNativeCycleLabel(session: SessionIdentity, cycleId: string): Promise<string> {
  const summary = await rpc.cyclesGet(session, { name: cycleId }).then((reply) => required(reply.cycle, "Get Cycle"));
  return summary.metadata?.label || cycleId;
}

/**
 * Plans a new cycle with the source's cases against the installed plugin.
 * Resolves to the new cycle's id and what the plugin no longer declares.
 */
export async function rerunNativeTestCycle(session: SessionIdentity, cycleId: string): Promise<{ cycleId: string } & RerunNotices> {
  const reply = await rpc.cyclesCreate(session, { sourceCycle: cycleId, requestId: crypto.randomUUID() });
  const notices = (items: typeof reply.skipped) =>
    items.map((item) => ({ caseId: item.caseId, reason: item.reason, suiteId: item.suiteId }));
  return {
    cycleId: required(reply.cycle, "Rerun Cycle").cycleId,
    droppedParameters: notices(reply.droppedParameters),
    skipped: notices(reply.skipped),
  };
}

export async function deleteNativeTestCycleCases(
  session: SessionIdentity,
  cycleId: string,
  caseIds: string[],
): Promise<StartTestCycleResponse> {
  await mapBounded(caseIds, (caseId) => rpc.casesDelete(session, { parent: cycleId, name: caseId, requestId: crypto.randomUUID() }));
  const plan = await fetchNativeTestCyclePlan(session, cycleId);
  if (!plan) throw new Error("The cycle disappeared after deleting cases.");
  return { plan };
}

export async function updateNativeTestCyclePolicy(
  session: SessionIdentity,
  cycleId: string,
  continueAutomatically: boolean,
): Promise<StartTestCycleResponse> {
  await replaceCycles(session, cycleId, { continueAutomatically: continueAutomatically });
  const plan = await fetchNativeTestCyclePlan(session, cycleId);
  if (!plan) throw new Error("The cycle disappeared after updating its policy.");
  return { plan };
}

/** Append A Fresh Unstarted Attempt Without Dispatching Hardware */
export async function appendNativeCaseAttempt(session: SessionIdentity, cycleId: string, caseId: string): Promise<string> {
  const executionId = crypto.randomUUID();
  await rpc.executionsCreate(session, { parent: cycleId, requestId: crypto.randomUUID(), executions: [{ executionId, caseId }] });
  return executionId;
}

async function loadNativeExecution(session: SessionIdentity, executionId: string): Promise<NativeExecution> {
  const reply = await rpc.executionsGet(session, { name: executionId });
  const detail = required(reply.execution, "Get Execution");
  const inputs = await listExecutionParameters(session, executionId, ExecutionParameterSet.INPUTS);
  const outputs = await listExecutionParameters(session, executionId, ExecutionParameterSet.OUTPUTS);
  const steps = await listExecutionSteps(session, executionId);
  return { detail, inputs, outputs, steps: steps.map((step) => ({ step, criteria: step.criteria })) };
}

type NativeExecution = {
  detail: ExecutionDetail;
  inputs: { name: string; value?: Value }[];
  outputs: { name: string; value?: Value }[];
  steps: { step: ExecutionStep; criteria: StepCriterion[] }[];
};

function mapNativeExecution(
  native: NativeExecution,
  cases: Pick<TestCase, "case_id" | "template_id">[],
  summary: CycleSummary,
): TestExecution {
  const detail = native.detail;
  const item = detail.summary;
  if (!item) throw new Error(`Execution ${detail.cycleId} has no summary.`);
  const testCase = cases.find((candidate) => candidate.case_id === item.caseId);
  const templateId = detail.admittedTemplate?.identifier ?? testCase?.template_id ?? "";
  const fallback = timestampIso(summary.updatedAt) ?? nowIso();
  const startedAt = timestampIso(detail.startedAt);
  return {
    controls: detail.controls
      ? {
          requestId: detail.controls.requestId,
          paused: detail.controls.paused,
          stopRequested: detail.controls.stopRequested,
          canStart: detail.controls.canStart,
          canRerun: detail.controls.canRerun,
          canCancel: detail.controls.canCancel,
          canInspect: detail.controls.canInspect,
          canExport: detail.controls.canExport,
          canPause: detail.controls.canPause,
          canResume: detail.controls.canResume,
          canStop: detail.controls.canStop,
          mode:
            detail.controls.mode === ExecutionMode.AUTOMATIC
              ? "automatic"
              : detail.controls.mode === ExecutionMode.INTERACTIVE
                ? "interactive"
                : null,
        }
      : null,
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

async function listExecutionParameters(session: SessionIdentity, executionId: string, set: ExecutionParameterSet) {
  return (await rpc.executionsParameters(session, { name: executionId, set })).parameters;
}

async function listExecutionSteps(session: SessionIdentity, executionId: string) {
  return (await rpc.executionsSteps(session, { name: executionId })).steps;
}

async function loadNativeExecutionsForPlan(session: SessionIdentity, cycleId: string): Promise<TestExecution[]> {
  const summaries = await listNativeExecutions(session, cycleId);
  const cases = await listNativeCases(session, cycleId);
  const mappedCases = cases.map((item) => ({
    suite_id: item.suiteId,
    case_id: item.caseId,
    template_id: item.templateId,
  }));
  const cycle = await rpc.cyclesGet(session, { name: cycleId }).then((reply) => required(reply.cycle, "Get Cycle"));
  const executions = await mapBounded(summaries, (item) => loadNativeExecution(session, item.executionId));
  return executions.map((execution) => mapNativeExecution(execution, mappedCases, cycle));
}

async function replaceExecutions(
  session: SessionIdentity,
  name: string,
  patch: Partial<NonNullable<Parameters<typeof rpc.executionsSet>[1]["value"]>>,
) {
  const current = await rpc.executionsGet(session, { name });
  const value = { ...required(current.value, "Set Execution"), ...patch };
  return rpc.executionsSet(session, { name, value, expectedRevision: current.revision, requestId: crypto.randomUUID() });
}

async function replaceCycles(
  session: SessionIdentity,
  name: string,
  patch: Partial<NonNullable<Parameters<typeof rpc.cyclesSet>[1]["value"]>>,
) {
  const current = await rpc.cyclesGet(session, { name });
  const value = { ...required(current.value, "Set Cycle"), ...patch };
  return rpc.cyclesSet(session, { name, value, expectedRevision: current.revision, requestId: crypto.randomUUID() });
}
