/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native planning adapters and the summary-shaped plan projection. */

/* === External Imports === */

/* === Local Imports === */

import {
  CaseStatus,
  type CaseDetail as HostCaseDetail,
  type CaseSummary as HostCaseSummary,
  type CaseActions as HostCaseActions,
} from "@host/dokime/client/v1/cases_pb.js";
import {
  CriterionStatus,
  LiveExecutionState,
  StepStatus,
  type ExecutionSummary as HostExecutionSummary,
  type ExecutionControls,
} from "@host/dokime/client/v1/executions_pb.js";
import { CyclePhase, CycleStatus, type CycleSummary as HostCycleSummary } from "@host/dokime/v1/cycles_pb.js";
import { CycleOrderKey } from "@host/dokime/client/v1/cycles_pb.js";
import { ExecutionState, ResultStatus, type ExecutionOutcome } from "@host/dokime/v1/execution_pb.js";
import type { ReadCut } from "@host/harmos/v1/types_pb.js";
import { required, cutKey } from "@/lib/rpc/canonical";
import { Termination } from "@host/dokime/v1/execution_pb.js";
import type { Value as HostValue } from "@host/dokime/v1/scalar_pb.js";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/host/client";
import { jsonToNativeValue, metadataJson, nativeValuesToJson, nativeValueToJson } from "@/lib/host/values";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type {
  CaseActions,
  ExecutionQueueItem,
  PreviewTestCycleRequest,
  PreviewTestCycleResponse,
  StartTestCycleResponse,
  StepExecutionRecord,
  SuiteTemplateEntry,
  TestCase,
  TestCycle,
  TestCyclePlan,
  TestCycleSelection,
  TestExecution,
  TestTemplateMetadata,
} from "@/lib/types";
import { fetchNativeSuite, fetchNativeSuites } from "@/lib/host/suites";
import {
  durationSeconds,
  mapBounded,
  NATIVE_CASE_LIMIT,
  NATIVE_MAX_PAGES,
  NATIVE_PAGE_SIZE,
  NativeApiLimitError,
  nowIso,
  safeNumber,
  timestampIso,
} from "./native-values";

type Cursor = Wire<ReadCut>;
type CaseDetail = Wire<HostCaseDetail>;
type CaseSummary = Wire<HostCaseSummary>;
type ExecutionSummary = Wire<HostExecutionSummary>;
export type CycleSummary = Wire<HostCycleSummary>;
type NativeCaseActions = Wire<HostCaseActions>;
type Value = Wire<HostValue>;

/* === Planning and execution === */

export async function previewNativeTestCycle(
  session: SessionIdentity,
  request: PreviewTestCycleRequest,
): Promise<PreviewTestCycleResponse> {
  const plan = await nativeSelectionPlan(session, request, request.cycle_id ?? "");
  const reply = await rpc.casesCreate(session, { plan, validateOnly: true });
  return {
    case_count: reply.cases.length,
    cases: reply.cases.map((item, _index) => ({
      suite_id: item.suiteId,
      case_id: item.caseId,
      template_id: item.templateId,
      parameters: nativeValuesToJson(item.origin?.kind.case === "pluginRun" ? item.origin.kind.value.testCase?.parameters : undefined),
      generation_index: safeNumber(item.ordinal, "planned case ordinal"),
      label: item.metadata?.label || item.caseId,
      metadata: metadataJson(item.metadata),
    })),
    // Native planning enforces NATIVE_CASE_LIMIT before returning this reply;
    // there is no separate over-cap result in the protobuf contract.
    exceeds_default_cap: false,
    default_cap: NATIVE_CASE_LIMIT,
  };
}

/** What the operator states about a new cycle; the backend derives its identifier. */
export type NewCycleRequest = {
  label: string;
  description?: string;
  continueAutomatically: boolean;
};

/**
 * Create an empty Active cycle.
 *
 * Cases materialize and append separately. Creation never starts hardware.
 */
export async function createNativeTestCycle(session: SessionIdentity, request: NewCycleRequest): Promise<{ cycleId: string }> {
  const created = await rpc.cyclesCreate(session, {
    metadata: {
      identifier: "",
      label: request.label,
      description: request.description,
      tags: [],
      attributes: {},
    },
    continueAutomatically: request.continueAutomatically,
    requestId: crypto.randomUUID(),
  });
  return { cycleId: required(created.cycle, "Create Cycle").cycleId };
}

export async function appendNativeTestCycle(
  session: SessionIdentity,
  cycleId: string,
  selections: TestCycleSelection[],
): Promise<StartTestCycleResponse> {
  const destination = required((await rpc.cyclesGet(session, { name: cycleId })).cycle, "Get Cycle");
  if (destination.phase !== CyclePhase.ACTIVE) throw new Error("Choose an Active cycle before adding cases.");
  /* Materialize Reusable Cases Before Registering Explicit Attempts */
  const plan = await nativeSelectionPlan(session, { parameters: {}, selections }, cycleId);
  const registered = await rpc.casesCreate(session, { plan, requestId: crypto.randomUUID() });
  const materialized = registered.cases;
  const executions = materialized.map((item) => ({ executionId: crypto.randomUUID(), caseId: item.caseId }));
  if (executions.length > NATIVE_CASE_LIMIT)
    throw new NativeApiLimitError("Execution.AppendExecutions", `more than ${NATIVE_CASE_LIMIT} cases`);
  await rpc.executionsCreate(session, { parent: cycleId, executions, requestId: crypto.randomUUID() });
  const next = await fetchNativeTestCyclePlan(session, cycleId);
  if (!next) throw new Error("The cycle disappeared after appending planned cases.");
  return { plan: next };
}

/** The cycle's plan, or `null` when the backend no longer holds that cycle. */
export async function fetchNativeTestCyclePlan(session: SessionIdentity, cycleId: string): Promise<TestCyclePlan | null> {
  const summaries = await listNativeCycles(session);
  const summary = summaries.find((candidate) => candidate.cycleId === cycleId);
  if (!summary) return null;
  return assembleNativeCyclePlan(session, summary);
}

/* === Private cycle assembly === */

/**
 * The cycle's plan, read from the two RPCs that own it.
 *
 * `CaseSummary` now projects each case's own state — its status, its latest
 * attempt and that attempt's status, how many attempts it holds, whether a
 * rerun is scheduled, and which mutations the runtime would admit — so nothing
 * here joins `ListExecutions` to learn what a case is doing. The attempt
 * vector is still a separate read, for the surfaces that list attempts
 * themselves.
 */
async function assembleNativeCyclePlan(session: SessionIdentity, summary: CycleSummary): Promise<TestCyclePlan> {
  const cases = await listNativeCases(session, summary.cycleId);
  if (summary.caseCount > BigInt(cases.length))
    throw new NativeApiLimitError(
      "Execution.ListCases",
      `cycle reports ${summary.caseCount} cases but the bounded read returned ${cases.length}`,
    );

  const caseDetails = await mapBounded(cases, async (item) => {
    const detail = required((await rpc.casesGet(session, { parent: summary.cycleId, name: item.caseId })).value, "Get Case");
    const parameters = await listNativeCaseParameters(session, summary.cycleId, item.caseId);
    return { item, detail, parameters };
  });
  /* A cycle has no suite of its own: its cases may come from any number of
     them, and each case's template is read from the suite it came from. */
  const suiteIds = [...new Set(caseDetails.flatMap((item) => item.detail.suite?.identifier || []))];
  const suites = await mapBounded(suiteIds, (suiteId) => fetchNativeSuite(session, suiteId));
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

export function mapNativeExecutionSummary(item: ExecutionSummary, cases: TestCase[], cycle: CycleSummary): TestExecution {
  const testCase = cases.find((candidate) => candidate.case_id === item.caseId);
  return {
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
  if (!actions) return null;
  return {
    delete: actions.delete,
    unavailable_reasons: [...actions.unavailableReasons],
  };
}

async function nativeSelectionPlan(
  session: SessionIdentity,
  request: PreviewTestCycleRequest,
  cycleId = "",
): Promise<Wire<NonNullable<Parameters<typeof rpc.casesCreate>[1]["plan"]>>> {
  const selections =
    request.selections && request.selections.length > 0
      ? request.selections
      : request.suite_id && request.template_id
        ? [
            {
              suite_id: request.suite_id,
              template_id: request.template_id,
              parameters: request.parameters,
              files: request.files,
            },
          ]
        : [];
  if (!selections.length) throw new Error("Select at least one registered template.");
  const suites = await fetchNativeSuites(session);
  const nativeSelections = selections.map((selection) => {
    const suite = suites.find((item) => item.suite_id === selection.suite_id);
    if (!suite) throw new Error(`Suite ${selection.suite_id} is no longer registered.`);
    const template = suite.members.find(
      (candidate) => candidate.template_id === selection.template_id || candidate.metadata.identifier === selection.template_id,
    );
    if (!template) throw new Error(`Template ${selection.template_id} is no longer registered in ${suite.name}.`);
    return {
      suiteId: suite.suite_id,
      templateId: template.template_id,
      parameters: Object.entries(selection.parameters ?? {}).flatMap(([key, values]) =>
        values.length ? [{ key, value: { values: values.map(jsonToNativeValue) } }] : [],
      ),
      files: Object.entries(selection.files ?? {}).map(([key, value]) => ({
        key,
        value: {
          kind: { kind: value === null ? { case: "clear" as const, value: {} } : { case: "value" as const, value } },
        },
      })),
    };
  });
  const maxCases = request.max_cases ?? NATIVE_CASE_LIMIT;
  if (maxCases < 1 || maxCases > NATIVE_CASE_LIMIT) throw new NativeApiLimitError("Cases.Create", `max_cases ${maxCases}`);
  return {
    selections: nativeSelections,
    maxCases,
    cycleId,
  };
}

/** A Durable Admission For One Execution */
export type NativeStartAdmission = { commandId: string; jobId: string | null };

/**
 * The admission, or the runtime's own refusal as an error.
 *
 * A domain rejection carries the runtime's code, message and validation
 * issues; raising it verbatim is what lets the workspace tell the operator
 * why, rather than leaving a control that did nothing.
 */
export function ensureNativeStartAccepted(reply: Wire<Awaited<ReturnType<typeof rpc.executionsStart>>>): NativeStartAdmission {
  const execution = required(reply.execution, "Start Execution");
  const commandId = required(execution.controls?.requestId, "Start Execution Admission");
  return { commandId, jobId: null };
}

/**
 * Server-side selection and ordering for the live cycle list.
 *
 * `text` matches a cycle's label and its identifier; an empty string selects
 * everything. `statuses` is disjunctive and an empty
 * list is no constraint, so hiding one lifecycle means naming the others.
 * `descending` is what makes UPDATED_AT newest-first: the server's own default
 * is ascending.
 */
export type NativeCycleSelection = {
  filter?: NonNullable<Parameters<typeof rpc.cyclesList>[1]["filter"]>;
  order?: NonNullable<Parameters<typeof rpc.cyclesList>[1]["order"]>;
};

/** Every page of Execution.ListCycles under one unchanged selection. */
export async function listNativeCycles(session: SessionIdentity, selection: NativeCycleSelection = {}): Promise<CycleSummary[]> {
  const result: CycleSummary[] = [];
  let pageToken = "";
  let at: Cursor | undefined;
  const filter = selection.filter ?? { text: "", statuses: [] };
  const order = selection.order ?? { key: CycleOrderKey.UPDATED_AT, descending: true };
  const tokens = new Set<string>();
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.cyclesList(session, { filter, order, at, page: { pageSize: NATIVE_PAGE_SIZE, pageToken } });
    const cut = required(reply.resolvedAt, "List Cycles Cut");
    if (at && cutKey(at) !== cutKey(cut)) throw new Error("List Cycles: Retained Cut Changed");
    at = cut;
    result.push(...reply.cycles);
    if (!reply.nextPageToken) return result;
    if (tokens.has(reply.nextPageToken)) throw new NativeApiLimitError("Cycles.List", "Continuation Did Not Advance");
    pageToken = reply.nextPageToken;
    tokens.add(pageToken);
  }
  throw new NativeApiLimitError("Cycles.List", "Page Budget Reached");
}

export async function listNativeCases(session: SessionIdentity, cycleId: string): Promise<CaseSummary[]> {
  return (await rpc.casesList(session, { parent: cycleId })).cases;
}

export async function listNativeExecutions(session: SessionIdentity, cycleId: string): Promise<ExecutionSummary[]> {
  const result: ExecutionSummary[] = [];
  let pageToken = "";
  let at: Cursor | undefined;
  const tokens = new Set<string>();
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.executionsList(session, { parent: cycleId, at, page: { pageSize: NATIVE_PAGE_SIZE, pageToken } });
    const cut = required(reply.resolvedAt, "List Executions Cut");
    if (at && cutKey(at) !== cutKey(cut)) throw new Error("List Executions: Retained Cut Changed");
    at = cut;
    result.push(...reply.executions.map((item) => required(item.summary, "List Execution Summary")));
    if (!reply.nextPageToken) return result;
    if (tokens.has(reply.nextPageToken)) throw new NativeApiLimitError("Executions.List", "Continuation Did Not Advance");
    pageToken = reply.nextPageToken;
    tokens.add(pageToken);
  }
  throw new NativeApiLimitError("Executions.List", "Page Budget Reached");
}

export async function listNativeCaseParameters(session: SessionIdentity, cycleId: string, caseId: string) {
  return (await rpc.casesParameters(session, { parent: cycleId, name: caseId })).parameters;
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
    if (liveState === LiveExecutionState.CLEANING_UP) return "finalizing";
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
      case Termination.FAILED:
        return "failed";
      case Termination.CANCELLED:
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
