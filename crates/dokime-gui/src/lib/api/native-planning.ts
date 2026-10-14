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

import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type {
  CaseActions,
  ExecutionQueueItem,
  PreviewTestCycleRequest,
  PreviewTestCycleResponse,
  StartTestCycleRequest,
  StartTestCycleResponse,
  StepExecutionRecord,
  SuiteTemplateEntry,
  TestCase,
  TestCyclePlan,
  TestExecution,
  TestSuiteEntry,
  TestTemplateMetadata,
} from "@/lib/types";
import type { Row19 as GeneratedValue } from "../../../.generated/dokime/v1/CatalogMessages/GetSuite/messages_pb";
import type { Row2 as GeneratedCaseDetail } from "../../../.generated/dokime/v1/ExecutionMessages/GetCase/messages_pb";
import type {
  Row4 as GeneratedCycleStatus,
  Row2 as GeneratedCycleSummary,
} from "../../../.generated/dokime/v1/ExecutionMessages/GetCycle/messages_pb";
import type { Row3 as GeneratedParameterCursor } from "../../../.generated/dokime/v1/ExecutionMessages/ListCaseParameters/messages_pb";
import type {
  Row6 as GeneratedCaseStatus,
  Row4 as GeneratedCaseSummary,
  Row8 as GeneratedNativeCaseActions,
} from "../../../.generated/dokime/v1/ExecutionMessages/ListCases/messages_pb";
import type { Row3 as GeneratedCursor } from "../../../.generated/dokime/v1/ExecutionMessages/ListCycles/messages_pb";
import type { Row8 as GeneratedStepStatus } from "../../../.generated/dokime/v1/ExecutionMessages/ListExecutionSteps/messages_pb";
import type {
  Row5 as GeneratedExecutionStatus,
  Row4 as GeneratedExecutionSummary,
} from "../../../.generated/dokime/v1/ExecutionMessages/ListExecutions/messages_pb";
import type { Row8 as GeneratedCriterionStatus } from "../../../.generated/dokime/v1/ExecutionMessages/ListStepCriteria/messages_pb";
import { fetchNativeSuite, fetchNativeSuites } from "./native-catalog";
import {
  durationSeconds,
  jsonToNativeValue,
  mapBounded,
  metadataJson,
  NATIVE_CASE_LIMIT,
  NATIVE_MAX_PAGES,
  NATIVE_PAGE_SIZE,
  NativeApiLimitError,
  nativeValueObjectToJson,
  nativeValueToJson,
  nowIso,
  safeNumber,
  timestampIso,
} from "./native-values";

type Cursor = Wire<GeneratedCursor>;
type CaseDetail = Wire<GeneratedCaseDetail>;
type CaseSummary = Wire<GeneratedCaseSummary>;
type ExecutionSummary = Wire<GeneratedExecutionSummary>;
type CycleSummary = Wire<GeneratedCycleSummary>;
type NativeCaseActions = Wire<GeneratedNativeCaseActions>;
type ParameterCursor = Wire<GeneratedParameterCursor>;
type Value = Wire<GeneratedValue>;
type CycleStatus = Wire<GeneratedCycleStatus>;
type CaseStatus = Wire<GeneratedCaseStatus>;
type ExecutionStatus = Wire<GeneratedExecutionStatus>;
type StepStatus = Wire<GeneratedStepStatus>;
type CriterionStatus = Wire<GeneratedCriterionStatus>;

/* === Constants === */

/** Match the original bounded activity observation after a run admission. */
const NATIVE_RUN_ACTIVITY_ATTEMPTS = 80;
const NATIVE_RUN_ACTIVITY_DELAY_MS = 125;

/* === Planning and execution === */

export async function previewNativeTestCycle(
  session: SessionIdentity,
  request: PreviewTestCycleRequest,
): Promise<PreviewTestCycleResponse> {
  const plan = await nativeSelectionPlan(session, request);
  const reply = await rpc.planningPreviewSelection(session, { plan });
  return {
    case_count: reply.cases.length,
    cases: reply.cases.map((item, _index) => ({
      suite_id: item.suiteId,
      case_id: item.caseId,
      template_id: item.templateId,
      parameters: nativeValueObjectToJson(item.parameters),
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

/**
 * Create the cycle and stop there.
 *
 * Execution.CreateCycle only: the cycle lands Ready and waits for the operator
 * to admit its run, so nothing here observes run activity.
 */
export async function createNativeTestCycle(
  session: SessionIdentity,
  request: StartTestCycleRequest,
): Promise<{ cycleId: string }> {
  const plan = await nativeSelectionPlan(session, request);
  const firstSuite = await firstSelectedSuite(session, request);
  const registered = await rpc.planningRegisterSelection(session, {
    plan,
    idempotencyKey: crypto.randomUUID(),
  });
  const created = await rpc.executionCreateCycle(session, {
    plan: { caseIds: registered.cases.map((item) => item.caseId) },
    metadata: {
      identifier: `cycle.${firstSuite.suite_id.replaceAll(".", "_")}`,
      label: request.title ?? firstSuite.name,
      description: firstSuite.description ?? undefined,
    },
    continueOnFailure: request.continue_on_failure ?? firstSuite.continue_on_failure,
  });
  return { cycleId: created.cycleId };
}

export async function startNativeTestCycle(
  session: SessionIdentity,
  request: StartTestCycleRequest,
): Promise<StartTestCycleResponse> {
  const { cycleId } = await createNativeTestCycle(session, request);
  await ensureNativeRunAccepted(
    await rpc.executionRunCycle(session, {
      cycleId,
      idempotencyKey: crypto.randomUUID(),
    }),
  );
  await waitForNativeCycleRunActivity(session, cycleId);
  const result = await fetchNativeTestCyclePlan(session, cycleId);
  if (!result) throw new Error("Native CreateCycle did not produce a cycle.");
  return { plan: result };
}

export async function appendNativeTestCycle(
  session: SessionIdentity,
  cycleId: string,
  request: StartTestCycleRequest,
): Promise<StartTestCycleResponse> {
  const plan = await nativeSelectionPlan(session, request);
  const registered = await rpc.planningRegisterSelection(session, {
    plan,
    idempotencyKey: crypto.randomUUID(),
  });
  await rpc.executionAppendPlannedCases(session, {
    cycleId,
    plan: { caseIds: registered.cases.map((item) => item.caseId) },
    context: { correlationId: crypto.randomUUID() },
  });
  const next = await fetchNativeTestCyclePlan(session, cycleId);
  if (!next) throw new Error("The cycle disappeared after appending planned cases.");
  return { plan: next };
}

/** The cycle's plan, or `null` when the backend no longer holds that cycle. */
export async function fetchNativeTestCyclePlan(
  session: SessionIdentity,
  cycleId: string,
): Promise<TestCyclePlan | null> {
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
async function assembleNativeCyclePlan(
  session: SessionIdentity,
  summary: CycleSummary,
): Promise<TestCyclePlan> {
  const cases = await listNativeCases(session, summary.cycleId);
  if (summary.caseCount > BigInt(cases.length))
    throw new NativeApiLimitError(
      "Execution.ListCases",
      `cycle reports ${summary.caseCount} cases but the bounded read returned ${cases.length}`,
    );

  const caseDetails = await mapBounded(cases, async (item) => {
    const detail = await rpc.executionGetCase(session, {
      cycleId: summary.cycleId,
      caseId: item.caseId,
    });
    const parameters = await listNativeCaseParameters(session, summary.cycleId, item.caseId);
    return { item, detail, parameters };
  });
  const suiteId = caseDetails.find((item) => item.detail.suite?.identifier)?.detail.suite
    ?.identifier;
  const suite = suiteId ? await getNativeSuiteById(session, suiteId) : null;
  const mappedCases = caseDetails.map((entry) =>
    mapNativeCase(entry.item, entry.detail, entry.parameters),
  );
  const queue = caseDetails.map((entry, index) =>
    mapQueueItem(summary, entry.detail.summary ?? entry.item, index),
  );
  /* The case the cycle is on, as the case projection states it: exactly one
     case can be RUNNING, and its own latest attempt is the live one. */
  const active = queue.find((item) => item.status === "running") ?? null;
  const templates = suite?.members.map((template) => mapTemplateMetadata(template)) ?? [];
  const generatedAt =
    timestampIso(summary.createdAt) ?? timestampIso(summary.updatedAt) ?? nowIso();
  return {
    test_cycle: {
      test_cycle_id: summary.cycleId,
      title: summary.metadata?.label || summary.cycleId,
      suite_id: suiteId ?? null,
      suite_name: suite?.name ?? null,
      status: mapCycleStatus(summary.status),
      case_count: mappedCases.length,
      queue_count: mappedCases.length,
      current_test_execution_id: active?.current_test_execution_id ?? null,
      current_queue_item_id: active?.case_id ?? null,
      continue_on_failure: summary.continueOnFailure,
      started_at: timestampIso(summary.startedAt),
      finished_at: isFinishedCycle(summary.status) ? timestampIso(summary.updatedAt) : null,
      duration_s: durationSeconds(summary.startedAt, summary.updatedAt),
      duration_delta_s: null,
      metadata: metadataJson(summary.metadata),
      generated_at: generatedAt,
      archived: summary.archived,
    },
    templates,
    cases: mappedCases,
    queue,
    generated_at: generatedAt,
  };
}

function mapNativeCase(
  item: CaseSummary,
  detail: CaseDetail,
  parameters: { name: string; value?: Value }[],
): TestCase {
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

export function mapNativeExecutionSummary(
  item: ExecutionSummary,
  cases: TestCase[],
  cycle: CycleSummary,
): TestExecution {
  const testCase = cases.find((candidate) => candidate.case_id === item.caseId);
  return {
    archived: item.archived,
    test_execution_id: item.executionId,
    test_cycle_id: cycle.cycleId,
    queue_item_id: item.caseId,
    case_id: item.caseId,
    template_id: testCase?.template_id ?? "",
    attempt_index: safeNumber(item.ordinal, "execution ordinal"),
    invalidation: null,
    status: projectExecutionStatus(item.status, null, cycle.status),
    started_at: null,
    finished_at: null,
    parameters: null,
    metadata: {},
    steps: [],
  };
}

function mapTemplateMetadata(template: SuiteTemplateEntry): TestTemplateMetadata {
  // Keep the original cycle-plan projection: the HTTP runtime mapper also
  // supplied no authored metadata here. Native Catalog carries the same
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
function mapQueueItem(
  summary: CycleSummary,
  item: CaseSummary,
  ordinal: number,
): ExecutionQueueItem {
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
      item.latestExecutionStatus === undefined
        ? null
        : mapExecutionStatus(item.latestExecutionStatus),
    attempt_count: item.attemptCount,
    rerun_scheduled: item.rerunScheduled,
    actions: mapCaseActions(item.actions),
    blocked_reason: null,
  };
}

/** Action eligibility, or `null` where the projection reports none at all. */
function mapCaseActions(actions: NativeCaseActions | undefined): CaseActions | null {
  if (!actions) return null;
  return {
    run: actions.run,
    schedule_rerun: actions.scheduleRerun,
    delete: actions.delete,
    unavailable_reasons: [...actions.unavailableReasons],
  };
}

async function firstSelectedSuite(
  session: SessionIdentity,
  request: StartTestCycleRequest,
): Promise<TestSuiteEntry> {
  const suites = await fetchNativeSuites(session);
  const suiteId = request.selections?.[0]?.suite_id ?? request.suite_id;
  const suite = suites.find((item) => item.suite_id === suiteId) ?? suites[0];
  if (!suite) throw new Error("No registered suite is available.");
  return suite;
}

async function nativeSelectionPlan(
  session: SessionIdentity,
  request: PreviewTestCycleRequest | StartTestCycleRequest,
): Promise<Wire<NonNullable<Parameters<typeof rpc.planningPreviewSelection>[1]["plan"]>>> {
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
      (candidate) =>
        candidate.template_id === selection.template_id ||
        candidate.metadata.identifier === selection.template_id,
    );
    if (!template)
      throw new Error(
        `Template ${selection.template_id} is no longer registered in ${suite.name}.`,
      );
    return {
      suiteId: suite.suite_id,
      templateId: template.template_id,
      parameters: Object.entries(selection.parameters ?? {}).flatMap(([key, values]) =>
        values.length ? [{ key, value: { values: values.map(jsonToNativeValue) } }] : [],
      ),
      files: Object.entries(selection.files ?? {}).map(([key, value]) => ({
        key,
        value: {
          kind: {
            HarmosVariant:
              value === null
                ? { case: "Clear" as const, value: {} }
                : { case: "Value" as const, value: { field0: value } },
          },
        },
      })),
    };
  });
  const maxCases = request.max_cases ?? NATIVE_CASE_LIMIT;
  if (maxCases < 1 || maxCases > NATIVE_CASE_LIMIT)
    throw new NativeApiLimitError("Planning.SelectionPlan", `max_cases ${maxCases}`);
  return {
    selections: nativeSelections,
    maxCases,
  };
}

/** What an admitted run answers with: the durable receipt identity, not a run. */
export type NativeRunAdmission = { commandId: string; jobId: string | null };

/**
 * The admission, or the runtime's own refusal as an error.
 *
 * A domain rejection carries the runtime's code, message and validation
 * issues; raising it verbatim is what lets the workspace tell the operator
 * why, rather than leaving a control that did nothing.
 */
export function ensureNativeRunAccepted(
  reply: Wire<Awaited<ReturnType<typeof rpc.executionRunCycle>>>,
): NativeRunAdmission {
  if (reply.outcome?.HarmosVariant.case === "Accepted") {
    const accepted = reply.outcome.HarmosVariant.value.field0;
    if (!accepted) throw new Error("Native run admission has no accepted payload.");
    return { commandId: accepted.commandId, jobId: accepted.jobId ?? null };
  }
  if (reply.outcome?.HarmosVariant.case === "Rejected") {
    const detail = reply.outcome.HarmosVariant.value.field0;
    if (!detail) throw new Error("Native run admission has no rejection payload.");
    const issues = detail.issues.map((issue) => `${issue.path}: ${issue.message}`).join("; ");
    /* The code travels with the message. It is the runtime's own vocabulary
       for the refusal — `conflict`, `runtime_unavailable`,
       `provider_identity_changed` — and without it the operator, and anyone
       reading a gate log, has prose and no way to tell two refusals apart. */
    const message = detail.code ? `${detail.code}: ${detail.message}` : detail.message;
    throw new Error(issues ? `${message} (${issues})` : message);
  }
  throw new Error("The native backend returned no run admission outcome.");
}

async function waitForNativeCycleRunActivity(
  session: SessionIdentity,
  cycleId: string,
): Promise<CycleSummary> {
  let latest: CycleSummary | undefined;
  for (let attempt = 0; attempt < NATIVE_RUN_ACTIVITY_ATTEMPTS; attempt++) {
    latest = await rpc.executionGetCycle(session, { cycleId });
    if (hasNativeCycleRunActivity(latest)) return latest;
    if (attempt + 1 < NATIVE_RUN_ACTIVITY_ATTEMPTS)
      await new Promise((resolve) => setTimeout(resolve, NATIVE_RUN_ACTIVITY_DELAY_MS));
  }
  // Match the original route behavior: return the latest authoritative cycle
  // snapshot after the bounded wait so the caller can render its current plan.
  if (!latest) throw new Error("Native CreateCycle did not produce a cycle.");
  return latest;
}

function hasNativeCycleRunActivity(cycle: CycleSummary): boolean {
  return (
    cycle.executionCount > 0n ||
    ["Running", "Paused", "Stopping", "Completed", "Failed", "Stopped", "Blocked"].includes(
      cycle.status?.HarmosVariant.case ?? "",
    )
  );
}

async function getNativeSuiteById(
  session: SessionIdentity,
  suiteId: string,
): Promise<TestSuiteEntry | null> {
  return fetchNativeSuite(session, suiteId);
}

/**
 * Server-side selection and ordering for the live cycle list.
 *
 * `text` matches a cycle's label, its identifier and its admitted suite label;
 * an empty string selects everything. `statuses` is disjunctive and an empty
 * list is no constraint, so hiding one lifecycle means naming the others.
 * `descending` is what makes UPDATED_AT newest-first: the server's own default
 * is ascending.
 */
export type NativeCycleSelection = {
  filter?: NonNullable<Parameters<typeof rpc.executionListCycles>[1]["filter"]>;
  order?: NonNullable<Parameters<typeof rpc.executionListCycles>[1]["order"]>;
};

/** Every page of Execution.ListCycles under one unchanged selection. */
export async function listNativeCycles(
  session: SessionIdentity,
  selection: NativeCycleSelection = {},
): Promise<CycleSummary[]> {
  const result: CycleSummary[] = [];
  let afterCycleId = "";
  let cursor: Cursor | undefined;
  /* The contract rejects a continuation whose selection or order changed
     rather than silently restarting it, so both are built once and repeated
     by reference on every page of this read. */
  const filter = selection.filter ?? { text: "", statuses: [], suiteIds: [] };
  const order = selection.order ?? {
    key: { HarmosVariant: { case: "UpdatedAt", value: {} } },
    descending: true,
  };
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.executionListCycles(session, {
      limit: NATIVE_PAGE_SIZE,
      afterCycleId,
      cursor,
      filter,
      order,
    });
    result.push(...reply.cycles);
    const next = reply.nextAfterCycleId;
    if (!next) return result;
    if (next === afterCycleId)
      throw new NativeApiLimitError("Execution.ListCycles", `cursor did not advance at ${next}`);
    afterCycleId = next;
    cursor = reply.cursor;
  }
  throw new NativeApiLimitError("Execution.ListCycles", `more than ${NATIVE_MAX_PAGES} pages`);
}

export async function listNativeCases(
  session: SessionIdentity,
  cycleId: string,
): Promise<CaseSummary[]> {
  const result: CaseSummary[] = [];
  let afterCaseId = "";
  let cursor: Cursor | undefined;
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.executionListCases(session, {
      cycleId,
      limit: NATIVE_PAGE_SIZE,
      afterCaseId,
      cursor,
    });
    result.push(...reply.cases);
    const next = reply.nextAfterCaseId;
    if (!next) return result;
    if (next === afterCaseId)
      throw new NativeApiLimitError("Execution.ListCases", `cursor did not advance at ${next}`);
    afterCaseId = next;
    cursor = reply.cursor;
  }
  throw new NativeApiLimitError("Execution.ListCases", `more than ${NATIVE_MAX_PAGES} pages`);
}

export async function listNativeExecutions(
  session: SessionIdentity,
  cycleId: string,
): Promise<ExecutionSummary[]> {
  const result: ExecutionSummary[] = [];
  let afterExecutionId = "";
  let cursor: Cursor | undefined;
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.executionListExecutions(session, {
      cycleId,
      limit: NATIVE_PAGE_SIZE,
      afterExecutionId,
      cursor,
    });
    result.push(...reply.executions);
    const next = reply.nextAfterExecutionId;
    if (!next) return result;
    if (next === afterExecutionId)
      throw new NativeApiLimitError(
        "Execution.ListExecutions",
        `cursor did not advance at ${next}`,
      );
    afterExecutionId = next;
    cursor = reply.cursor;
  }
  throw new NativeApiLimitError("Execution.ListExecutions", `more than ${NATIVE_MAX_PAGES} pages`);
}

export async function listNativeCaseParameters(
  session: SessionIdentity,
  cycleId: string,
  caseId: string,
) {
  const result: { name: string; value?: Value }[] = [];
  let afterName = "";
  let cursor: ParameterCursor | undefined;
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.executionListCaseParameters(session, {
      cycleId,
      caseId,
      limit: NATIVE_PAGE_SIZE,
      afterName,
      cursor,
    });
    result.push(...reply.parameters);
    const next = reply.nextAfterName;
    if (!next) return result;
    if (next === afterName)
      throw new NativeApiLimitError(
        "Execution.ListCaseParameters",
        `cursor did not advance at ${next}`,
      );
    afterName = next;
    cursor = reply.cursor;
  }
  throw new NativeApiLimitError(
    "Execution.ListCaseParameters",
    `more than ${NATIVE_MAX_PAGES} pages`,
  );
}

export function mapCycleStatus(
  status: CycleStatus | undefined,
): TestCyclePlan["test_cycle"]["status"] {
  switch (status?.HarmosVariant.case) {
    case "Running":
      return "running";
    case "Paused":
      return "paused";
    case "Stopping":
      return "stopping";
    case "Completed":
      return "completed";
    case "Failed":
    case "Blocked":
      return "failed";
    case "Stopped":
      return "stopped";
    default:
      return "planned";
  }
}

export function mapExecutionStatus(status: ExecutionStatus | undefined): TestExecution["status"] {
  switch (status?.HarmosVariant.case) {
    case "Skipped":
      return "skipped";
    case "Running":
      return "running";
    case "Passed":
      return "passed";
    case "Failed":
      return "failed";
    case "Blocked":
      return "failed";
    case "Cancelled":
      return "stopped";
    case "Interrupted":
      return "interrupted";
    case "Invalidated":
      return "invalidated";
    default:
      return "created";
  }
}

/**
 * The status an attempt has, once `SKIPPED` is read for what it means.
 *
 * `ExecutionStatus::Skipped` is documented as "Not run. Before sealing this may
 * still be selected for execution.", and CreateCycle gives every planned case
 * one attempt in exactly that state. Projected literally, a cycle nobody has
 * run yet arrives with every case finished and skipped, which is the opposite
 * of the truth: the attempt is planned, and running the cycle is the one thing
 * left to do. An attempt that did start, or one whose cycle has been sealed, is
 * a real skip — the runtime passed over it — and keeps `skipped`.
 *
 * `ExecutionSummary` carries no start instant, so the bounded summary decides
 * on the cycle's lifecycle alone; the execution detail, which does carry one,
 * passes it in.
 */
export function projectExecutionStatus(
  status: ExecutionStatus | undefined,
  startedAt: string | null,
  cycleStatus: CycleStatus | undefined,
): TestExecution["status"] {
  if (status?.HarmosVariant.case === "Skipped" && !startedAt && !isFinishedCycle(cycleStatus)) {
    return "planned";
  }
  return mapExecutionStatus(status);
}

/**
 * A case's own state, as `CaseSummary.status` projects its latest attempt.
 *
 * This is the whole reason the client join is gone: PLANNED and QUEUED are the
 * two pre-dispatch states the runtime distinguishes — no attempt at all, and a
 * materialized attempt nobody has dispatched — and neither is an outcome.
 * BLOCKED keeps its own word here because a case list has room to say it,
 * where the attempt projection folds it into `failed`. UNSPECIFIED is the
 * historical projection: a captured cycle grouping retains the plan without
 * its attempt records and so states nothing about a case's state.
 */
export function mapCaseStatus(status: CaseStatus | undefined): ExecutionQueueItem["status"] {
  switch (status?.HarmosVariant.case) {
    case "Planned":
      return "planned";
    case "Queued":
      return "queued";
    case "Running":
      return "running";
    case "Passed":
      return "passed";
    case "Failed":
      return "failed";
    case "Blocked":
      return "blocked";
    case "Cancelled":
      return "stopped";
    case "Interrupted":
      return "interrupted";
    case "Skipped":
      return "skipped";
    case "Invalidated":
      return "invalidated";
    default:
      return "unknown";
  }
}

export function mapStepStatus(status: StepStatus | undefined): StepExecutionRecord["status"] {
  switch (status?.HarmosVariant.case) {
    case "Running":
      return "running";
    case "Passed":
      return "passed";
    case "Failed":
      return "failed";
    case "Skipped":
      return "skipped";
    case "Cancelled":
      return "stopped";
    default:
      return "pending";
  }
}

export function mapCriterionStatus(
  status: CriterionStatus | undefined,
): StepExecutionRecord["status"] {
  switch (status?.HarmosVariant.case) {
    case "Passed":
      return "passed";
    case "Failed":
      return "failed";
    case "Indeterminate":
      return "indeterminate";
    case "Skipped":
      return "skipped";
    default:
      return "pending";
  }
}

export function isFinishedCycle(status: CycleStatus | undefined): boolean {
  return ["Completed", "Failed", "Stopped"].includes(status?.HarmosVariant.case ?? "");
}
