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

import * as rpc from "@/lib/host/client";
import { required } from "@/lib/rpc/canonical";
import { isHostReopen } from "@/lib/rpc/errors";
import { ArchiveScope, CyclePhase, ExecutionMode, ExecutionParameterSet } from "@generated/host/model";
import { type NativeExecution, mapNativeExecution } from "@/lib/api/native-execution-projection";

import type { SessionIdentity } from "@/lib/rpc/transport";
import type { StartTestCycleResponse, TestCyclePlan, TestExecution } from "@/lib/types";
import type { RerunNotices } from "@/stores/rerun-notice-store";
import {
  ensureNativeStartAccepted,
  fetchNativeTestCyclePlan,
  listNativeCases,
  listNativeExecutions,
  type NativeStartAdmission,
} from "./native-planning";
import { mapNativeExecutionSummary } from "@/lib/api/native-planning-projection";
import { mapBounded } from "@/lib/api/native-values";

/* === Types === */

/* === Public Functions === */

export async function fetchNativeTestCycleDetail(
  session: SessionIdentity,
  cycleId: string,
): Promise<{ executions: TestExecution[]; plan: TestCyclePlan | null }> {
  const plan = await fetchNativeTestCyclePlan(session, cycleId);
  if (!plan) {
    return { executions: [], plan: null };
  }
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
  const admitted = await rpc.cyclesSeal(session, { name: cycleId, requestId: crypto.randomUUID() });
  if (required(admitted.cycle, "Seal Cycle Admission").sealed) return;
  // A Long Seal Outlives One Watch: A Host Deadline Reopens It.
  for (;;) {
    const completion = await rpc.cyclesWatch(session, { selection: { cycleIds: [cycleId] }, filter: { archive: ArchiveScope.ALL } });
    try {
      for await (const frame of completion) {
        if (frame.resyncRequired) throw new Error(frame.resyncRequired.message);
        const cycle = frame.cycles.find((cycle) => cycle.cycleId === cycleId);
        if (!cycle) throw new Error("Seal Cycle: Reconnect and Inspect Its Completion");
        if (cycle.phase === CyclePhase.SEAL_FAILED)
          throw new Error(cycle.sealFailure || "Seal Cycle: Resolve Its Materialization Failure and Retry Seal");
        if (cycle.sealed && cycle.phase === CyclePhase.SEALED) return;
      }
      throw new Error("Seal Cycle: Reconnect and Inspect Its Completion");
    } catch (error) {
      if (!isHostReopen(error)) throw error;
    } finally {
      await completion.cancel();
    }
  }
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
  if (!plan) {
    throw new Error("The cycle disappeared after deleting cases. Reload Cycles and select an available cycle.");
  }
  return { plan };
}

export async function updateNativeTestCyclePolicy(
  session: SessionIdentity,
  cycleId: string,
  continueAutomatically: boolean,
): Promise<StartTestCycleResponse> {
  await replaceCycles(session, cycleId, { continueAutomatically: continueAutomatically });
  const plan = await fetchNativeTestCyclePlan(session, cycleId);
  if (!plan) {
    throw new Error("The cycle disappeared after updating its policy. Reload Cycles and select an available cycle.");
  }
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
  const inputs = (await rpc.executionsParameters(session, { name: executionId, set: ExecutionParameterSet.INPUTS })).parameters;
  const outputs = (await rpc.executionsParameters(session, { name: executionId, set: ExecutionParameterSet.OUTPUTS })).parameters;
  const steps = (await rpc.executionsSteps(session, { name: executionId })).steps;
  return { detail, inputs, outputs, steps: steps.map((step) => ({ step, criteria: step.criteria })) };
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
