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

import { cutKey, required } from "@/lib/rpc/canonical";
import type { ReadCut } from "@host/model";
import { CycleOrderKey, CyclePhase } from "@host/model";
import { CaseSummary, CycleSummary, ExecutionSummary, mapNativeCyclePlan } from "./native-planning-projection";

import * as rpc from "@/lib/host/client";
import { jsonToNativeValue, metadataJson, nativeValuesToJson } from "@/lib/host/values";
import type { Wire } from "@/lib/rpc/canonical";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type {
  PreviewTestCycleRequest,
  PreviewTestCycleResponse,
  StartTestCycleResponse,
  TestCyclePlan,
  TestCycleSelection,
} from "@/lib/types";
import { fetchNativeSuite, fetchNativeSuites } from "../host/suite-session";
import { NATIVE_CASE_LIMIT, NATIVE_MAX_PAGES, NATIVE_PAGE_SIZE, NativeApiLimitError, mapBounded, safeNumber } from "./native-values";

type Cursor = Wire<ReadCut>;

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
  return mapNativeCyclePlan(summary, caseDetails, suites);
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
