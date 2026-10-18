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
import * as rpc from "@/lib/rpc/generated/client";

type CycleSummary = Wire<Awaited<ReturnType<typeof rpc.executionGetCycle>>>;
type ExecutionDetail = Wire<Awaited<ReturnType<typeof rpc.executionGetExecution>>>;
type ExecutionStep = Wire<Awaited<ReturnType<typeof rpc.executionListExecutionSteps>>["steps"][number]>;
type StepCriterion = Wire<Awaited<ReturnType<typeof rpc.executionListStepCriteria>>["criteria"][number]>;
type ParameterCursor = Parameters<typeof rpc.executionListExecutionParameters>[1]["cursor"];
type ExecutionParameterSet = Parameters<typeof rpc.executionListExecutionParameters>[1]["set"];
type StepPageCursor = Wire<Parameters<typeof rpc.executionListExecutionSteps>[1]["cursor"]>;
type Value = NonNullable<Wire<Awaited<ReturnType<typeof rpc.executionListExecutionParameters>>["parameters"][number]>["value"]>;
type WirePluginIdentity = NonNullable<ExecutionDetail["admittedPlugin"]>;

import type { SessionIdentity } from "@/lib/rpc/transport";
import type {
  ExecutionPluginIdentity,
  StartTestCycleResponse,
  StepExecutionRecord,
  TestCase,
  TestCyclePlan,
  TestExecution,
} from "@/lib/types";
import {
  ensureNativeRunAccepted,
  fetchNativeTestCyclePlan,
  listNativeCases,
  listNativeCycles,
  listNativeExecutions,
  mapCriterionStatus,
  mapNativeExecutionSummary,
  mapStepStatus,
  type NativeRunAdmission,
  projectExecutionStatus,
} from "./native-planning";
import {
  mapBounded,
  metadataJson,
  NATIVE_MAX_PAGES,
  NATIVE_PAGE_SIZE,
  NativeApiLimitError,
  nativeValueToJson,
  nowIso,
  safeNumber,
  timestampIso,
} from "./native-values";

export async function fetchNativeTestCycleDetail(
  session: SessionIdentity,
  cycleId: string,
): Promise<{ executions: TestExecution[]; plan: TestCyclePlan | null }> {
  const plan = await fetchNativeTestCyclePlan(session, cycleId);
  if (!plan) return { executions: [], plan: null };
  const [cycle, summaries] = await Promise.all([
    rpc.executionGetCycle(session, { cycleId: plan.test_cycle.test_cycle_id }),
    listNativeExecutions(session, plan.test_cycle.test_cycle_id),
  ]);
  return {
    executions: summaries.map((summary) => mapNativeExecutionSummary(summary, plan.cases, cycle)),
    plan,
  };
}

export async function fetchNativeTestCycleExecutions(session: SessionIdentity, cycleId: string): Promise<TestExecution[]> {
  return (await fetchNativeTestCycleDetail(session, cycleId)).executions;
}

/** Load steps, criteria, parameters, and result data for one selected execution. */
export async function fetchNativeTestExecutionDetail(session: SessionIdentity, executionId: string): Promise<TestExecution> {
  const native = await loadNativeExecution(session, executionId);
  const cycleId = native.detail.cycleId;
  const [cycle, cases] = await Promise.all([rpc.executionGetCycle(session, { cycleId }), listNativeCases(session, cycleId)]);
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

/**
 * Admit the cycle's queued cases, and answer with the admission itself.
 *
 * The admission is the whole result: `command_id` is the durable
 * `GetRunRequest` identity, and a run is not a completion, so there is nothing
 * further to wait for here. It used to re-read the entire plan before
 * answering, which put a bounded multi-RPC read across the exact moment the
 * runtime is materializing attempts — a read that can fail, and whose failure
 * hid the admission that had already succeeded from the caller's own refresh.
 * The workspace refreshes on the command settling and on the runtime stream.
 */
export async function runNativeQueuedTestCycle(session: SessionIdentity, cycleId: string): Promise<NativeRunAdmission> {
  return ensureNativeRunAccepted(
    await rpc.executionRunCycle(session, {
      cycleId,
      idempotencyKey: crypto.randomUUID(),
    }),
  );
}

export async function runNativeQueuedTestExecution(session: SessionIdentity, executionId: string): Promise<NativeRunAdmission> {
  return ensureNativeRunAccepted(
    await rpc.executionRunExecution(session, {
      executionId,
      idempotencyKey: crypto.randomUUID(),
    }),
  );
}

export async function runNativeQueuedTestCase(session: SessionIdentity, cycleId: string, caseId: string): Promise<NativeRunAdmission> {
  return ensureNativeRunAccepted(
    await rpc.executionRunCase(session, {
      cycleId,
      caseId,
      idempotencyKey: crypto.randomUUID(),
    }),
  );
}

export async function pauseNativeTestExecution(session: SessionIdentity, executionId: string): Promise<void> {
  await rpc.executionPauseExecution(session, { executionId });
}

export async function resumeNativeTestExecution(session: SessionIdentity, executionId: string): Promise<void> {
  await rpc.executionResumeExecution(session, { executionId });
}

export async function stopNativeTestExecution(session: SessionIdentity, executionId: string): Promise<void> {
  await rpc.executionStopExecution(session, { executionId });
}

export async function pauseNativeTestCycle(session: SessionIdentity, cycleId: string): Promise<StartTestCycleResponse | null> {
  return controlNativeCycle(session, cycleId, (id) => rpc.executionPauseCycle(session, { cycleId: id }));
}

export async function resumeNativeTestCycle(session: SessionIdentity, cycleId: string): Promise<StartTestCycleResponse | null> {
  return controlNativeCycle(session, cycleId, (id) => rpc.executionResumeCycle(session, { cycleId: id }));
}

export async function stopNativeTestCycle(session: SessionIdentity, cycleId: string): Promise<StartTestCycleResponse | null> {
  return controlNativeCycle(session, cycleId, (id) => rpc.executionStopCycle(session, { cycleId: id }));
}

export async function sealNativeTestCycle(session: SessionIdentity, cycleId: string): Promise<void> {
  await rpc.executionSealCycle(session, { cycleId });
}

/** Archives or unarchives a terminal cycle; its lifecycle is unchanged. */
export async function setNativeCycleArchived(session: SessionIdentity, cycleId: string, archived: boolean): Promise<void> {
  await rpc.executionSetCycleArchived(session, { cycleId, archived });
}

/** Archives or unarchives a terminal execution; its lifecycle is unchanged. */
export async function setNativeExecutionArchived(session: SessionIdentity, executionId: string, archived: boolean): Promise<void> {
  await rpc.executionSetExecutionArchived(session, { executionId, archived });
}

/** Renames a cycle in any lifecycle state; resolves to the label as stored. */
export async function renameNativeTestCycle(session: SessionIdentity, cycleId: string, label: string): Promise<string> {
  return (await rpc.executionRenameCycle(session, { cycleId, label })).label;
}

/** Renames an execution's case, every attempt of it; resolves to the stored label. */
export async function renameNativeTestExecution(session: SessionIdentity, executionId: string, label: string): Promise<string> {
  return (await rpc.executionRenameExecution(session, { executionId, label })).label;
}

export async function reopenNativeTestCycle(session: SessionIdentity, cycleId: string): Promise<StartTestCycleResponse> {
  const created = await rpc.executionCopyCycle(session, { sourceCycleId: cycleId });
  const plan = await fetchNativeTestCyclePlan(session, created.cycleId);
  if (!plan) throw new Error("Native CopyCycle did not produce a cycle.");
  return { plan };
}

export async function deleteNativeTestCycleCases(
  session: SessionIdentity,
  cycleId: string,
  caseIds: string[],
): Promise<StartTestCycleResponse> {
  await mapBounded(caseIds, (caseId) => rpc.executionDeleteCase(session, { cycleId, caseId }));
  const plan = await fetchNativeTestCyclePlan(session, cycleId);
  if (!plan) throw new Error("The cycle disappeared after deleting cases.");
  return { plan };
}

export async function updateNativeTestCyclePolicy(
  session: SessionIdentity,
  cycleId: string,
  continueOnFailure: boolean,
): Promise<StartTestCycleResponse> {
  await rpc.executionUpdateCyclePolicy(session, { cycleId, continueOnFailure });
  const plan = await fetchNativeTestCyclePlan(session, cycleId);
  if (!plan) throw new Error("The cycle disappeared after updating its policy.");
  return { plan };
}

export async function rerunNativeTestCase(session: SessionIdentity, cycleId: string, caseId: string): Promise<NativeRunAdmission> {
  await rpc.executionScheduleCaseRerun(session, { cycleId, caseId });
  return runNativeQueuedTestCase(session, cycleId, caseId);
}

/**
 * Run a finished cycle again: schedule a re-run of each named case, then run
 * the cycle's queue once, so the cases run as one cycle request in plan order.
 */
export async function rerunNativeTestCycle(
  session: SessionIdentity,
  cycleId: string,
  caseIds: readonly string[],
): Promise<NativeRunAdmission> {
  if (caseIds.length === 0) throw new Error("No case of this cycle can be run again.");
  for (const caseId of caseIds) {
    await rpc.executionScheduleCaseRerun(session, { cycleId, caseId });
  }
  return runNativeQueuedTestCycle(session, cycleId);
}

async function controlNativeCycle(
  session: SessionIdentity,
  cycleId: string,
  control: (cycleId: string) => Promise<unknown>,
): Promise<StartTestCycleResponse | null> {
  const cycle = (await listNativeCycles(session)).find((item) => item.cycleId === cycleId);
  if (!cycle) return null;
  await control(cycle.cycleId);
  const plan = await fetchNativeTestCyclePlan(session, cycle.cycleId);
  return plan ? { plan } : null;
}

async function loadNativeExecution(session: SessionIdentity, executionId: string): Promise<NativeExecution> {
  const detail = await rpc.executionGetExecution(session, { executionId });
  const inputs = await listExecutionParameters(session, executionId, {
    HarmosVariant: { case: "Inputs", value: {} },
  });
  const outputs = await listExecutionParameters(session, executionId, {
    HarmosVariant: { case: "Outputs", value: {} },
  });
  const steps = await listExecutionSteps(session, executionId);
  const criteria = await mapBounded(steps, async (step) => ({
    step,
    criteria: await listStepCriteria(session, executionId, step.stepId),
  }));
  return { detail, inputs, outputs, steps: criteria };
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
  const templateId = detail.admittedTemplateId ?? testCase?.template_id ?? "";
  const fallback = timestampIso(summary.updatedAt) ?? nowIso();
  const startedAt = timestampIso(detail.startedAt);
  return {
    controls: detail.controls
      ? {
          requestId: detail.controls.requestId,
          paused: detail.controls.paused,
          stopRequested: detail.controls.stopRequested,
          canPause: detail.controls.canPause,
          canResume: detail.controls.canResume,
          canStop: detail.controls.canStop,
          mode:
            detail.controls.mode?.HarmosVariant.case === "Automatic"
              ? "automatic"
              : detail.controls.mode?.HarmosVariant.case === "Interactive"
                ? "interactive"
                : null,
        }
      : null,
    archived: item.archived,
    sealed: item.sealed,
    test_execution_id: item.executionId,
    test_cycle_id: detail.cycleId,
    queue_item_id: item.caseId,
    case_id: item.caseId,
    template_id: templateId,
    attempt_index: safeNumber(item.ordinal, "execution ordinal"),
    invalidation: detail.invalidation
      ? {
          previous: mapPluginIdentity(detail.invalidation.previous),
          current: detail.invalidation.current ? mapPluginIdentity(detail.invalidation.current) : null,
        }
      : null,
    status: projectExecutionStatus(item.status, startedAt, summary.status),
    started_at: startedAt,
    finished_at: timestampIso(detail.finishedAt),
    parameters: Object.fromEntries(
      [...native.inputs, ...native.outputs].flatMap((parameter) => {
        const value = nativeValueToJson(parameter.value);
        return value === undefined ? [] : [[parameter.name, value]];
      }),
    ),
    metadata: {
      ...(detail.result ? { result: detail.result } : {}),
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
    template_id: detail.admittedTemplateId ?? "",
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
    template_id: detail.admittedTemplateId ?? "",
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
  const result: { name: string; value?: Value }[] = [];
  let afterName = "";
  let cursor: ParameterCursor | undefined;
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.executionListExecutionParameters(session, {
      executionId,
      set,
      limit: NATIVE_PAGE_SIZE,
      afterName,
      cursor,
    });
    result.push(...reply.parameters);
    const next = reply.nextAfterName;
    if (!next) return result;
    if (next === afterName) throw new NativeApiLimitError("Execution.ListExecutionParameters", `cursor did not advance at ${next}`);
    afterName = next;
    cursor = reply.cursor;
  }
  throw new NativeApiLimitError("Execution.ListExecutionParameters", `more than ${NATIVE_MAX_PAGES} pages`);
}

async function listExecutionSteps(session: SessionIdentity, executionId: string) {
  const result: ExecutionStep[] = [];
  let cursor: StepPageCursor | undefined;
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.executionListExecutionSteps(session, {
      executionId,
      limit: NATIVE_PAGE_SIZE,
      cursor,
    });
    result.push(...reply.steps);
    if (!reply.nextCursor) return result;
    if (stepCursorKey(reply.nextCursor) === stepCursorKey(cursor))
      throw new NativeApiLimitError("Execution.ListExecutionSteps", "cursor did not advance");
    cursor = reply.nextCursor;
  }
  throw new NativeApiLimitError("Execution.ListExecutionSteps", `more than ${NATIVE_MAX_PAGES} pages`);
}

async function listStepCriteria(session: SessionIdentity, executionId: string, stepId: string) {
  const result: StepCriterion[] = [];
  let cursor: StepPageCursor | undefined;
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.executionListStepCriteria(session, {
      executionId,
      stepId,
      limit: NATIVE_PAGE_SIZE,
      cursor,
    });
    result.push(...reply.criteria);
    if (!reply.nextCursor) return result;
    if (stepCursorKey(reply.nextCursor) === stepCursorKey(cursor))
      throw new NativeApiLimitError("Execution.ListStepCriteria", "cursor did not advance");
    cursor = reply.nextCursor;
  }
  throw new NativeApiLimitError("Execution.ListStepCriteria", `more than ${NATIVE_MAX_PAGES} pages`);
}

async function loadNativeExecutionsForPlan(session: SessionIdentity, cycleId: string): Promise<TestExecution[]> {
  const summaries = await listNativeExecutions(session, cycleId);
  const cases = await listNativeCases(session, cycleId);
  const mappedCases = cases.map((item) => ({
    suite_id: item.suiteId,
    case_id: item.caseId,
    template_id: item.templateId,
  }));
  const cycle = await rpc.executionGetCycle(session, { cycleId });
  const executions = await mapBounded(summaries, (item) => loadNativeExecution(session, item.executionId));
  return executions.map((execution) => mapNativeExecution(execution, mappedCases, cycle));
}

function stepCursorKey(cursor: StepPageCursor): string {
  if (!cursor) return "";
  return `${cursor.executionId}:${cursor.stepId}:${cursor.kind?.HarmosVariant?.case ?? ""}:${cursor.nextOrdinal}`;
}

function mapPluginIdentity(value: WirePluginIdentity | undefined): ExecutionPluginIdentity {
  if (!value) throw new Error("Execution invalidation has no previous provider identity.");
  const implementation = value.implementation?.HarmosVariant;
  if (implementation?.case === "ContentDigest") {
    return {
      source: value.source,
      version: value.version,
      implementation: { kind: "bundle", content_digest: implementation.value.field0 },
    };
  }
  if (implementation?.case === "Generation") {
    return {
      source: value.source,
      version: value.version,
      implementation: { kind: "embedded", generation: implementation.value.field0 },
    };
  }
  throw new Error("Execution provider identity has no implementation.");
}
