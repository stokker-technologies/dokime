/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The cycle workspace's reads and commands, every one naming its cycle.
 *
 * These hooks reuse the query keys and invalidation of `server-state.ts` so the
 * runtime stream keeps both the routed workspace and this one fresh. They add
 * no projection: the `lib/api` adapters still own the protobuf boundary.
 */

/* === External Imports === */

import { createMutation, type QueryClient } from "@tanstack/svelte-query";

/* === Local Imports === */

import type { ConnectedSession } from "@/app/native-session/session";
import type { SessionIdentity } from "@/lib/rpc/transport";
import { deliverNativeArtifact } from "@/lib/api/native-delivery";
import {
  appendNativeCaseAttempt,
  cancelNativeTestExecution,
  deleteNativeTestCycleCases,
  fetchNativeCycleLabel,
  fetchNativeTestCycleDetail,
  fetchNativeTestExecutionDetail,
  pauseNativeTestExecution,
  renameNativeTestCycle,
  renameNativeTestExecution,
  rerunNativeTestCycle,
  resumeNativeTestExecution,
  sealNativeTestCycle,
  setNativeCycleArchived,
  setNativeExecutionArchived,
  startNativeTestExecution,
  stopNativeTestExecution,
  updateNativeTestCyclePolicy,
} from "@/lib/api/native-execution";
import { fetchNativeRuntimeShell } from "@/lib/api/native-observation";
import { fetchNativePendingPrompts } from "@/lib/api/native-prompts";
import { downloadNativeReport } from "@/lib/api/native-reports";
import { reportFilename } from "@/features/cycle/cycle-workspace-model";
import { invalidateWorkflowResourceQueries } from "./server-state";
import { workflowQueryKeys } from "./workflow-query-keys";

/* === Private Functions === */

/** The Cycle's Cases and Every Attempt, Read to Completion. */
export function cycleDetailQueryOptions(identity: SessionIdentity, cycleId: string) {
  return {
    queryFn: () => fetchNativeTestCycleDetail(identity, cycleId),
    queryKey: [...workflowQueryKeys.cycleDetail(cycleId), identity.generation],
    staleTime: Number.POSITIVE_INFINITY,
  };
}

export function executionDetailQueryOptions(identity: SessionIdentity, executionId: string | null) {
  return {
    enabled: executionId !== null,
    queryFn: () => fetchNativeTestExecutionDetail(identity, executionId ?? ""),
    queryKey: [...workflowQueryKeys.executionDetail(executionId), identity.generation],
    staleTime: Number.POSITIVE_INFINITY,
  };
}

/** The live runtime as this cycle sees it, never another cycle's. */
export function cycleRuntimeQueryOptions(identity: SessionIdentity, cycleId: string) {
  return {
    queryFn: () => fetchNativeRuntimeShell(identity, cycleId),
    queryKey: [...workflowQueryKeys.runtimeShell(), cycleId, identity.generation],
  };
}

/** The label of the cycle this one reruns for its lineage link. */
export function cycleLabelQueryOptions(identity: SessionIdentity, cycleId: string | null) {
  return {
    enabled: cycleId !== null,
    queryFn: () => fetchNativeCycleLabel(identity, cycleId ?? ""),
    queryKey: [...workflowQueryKeys.cycleDetail(cycleId ?? ""), "label", identity.generation],
  };
}

export function cyclePromptQueryOptions(identity: SessionIdentity, cycleId: string) {
  return {
    queryFn: () => fetchNativePendingPrompts(identity, { cycleId }),
    queryKey: [...workflowQueryKeys.prompts(), cycleId, identity.generation],
  };
}

/* === Mutation Hooks === */

export function createCycleCommands(session: () => ConnectedSession, cycleId: () => string) {
  const client = () => session().queryClient;
  const options = () => {
    const { identity, queryClient } = session();
    return cycleCommandOptions(() => identity, queryClient, cycleId());
  };
  return {
    exportEvidence: createMutation(() => options().exportEvidence, client),
    archiveCycle: createMutation(() => options().archiveCycle, client),
    archiveExecution: createMutation(() => options().archiveExecution, client),
    renameCycle: createMutation(() => options().renameCycle, client),
    renameExecution: createMutation(() => options().renameExecution, client),
    rerunCycle: createMutation(() => options().rerunCycle, client),
    deleteCase: createMutation(() => options().deleteCase, client),
    downloadReport: createMutation(() => options().downloadReport, client),
    pauseExecution: createMutation(() => options().pauseExecution, client),
    resumeExecution: createMutation(() => options().resumeExecution, client),
    stopExecution: createMutation(() => options().stopExecution, client),
    cancelExecution: createMutation(() => options().cancelExecution, client),
    startExecution: createMutation(() => options().startExecution, client),
    appendAttempt: createMutation(() => options().appendAttempt, client),
    sealCycle: createMutation(() => options().sealCycle, client),
    updatePolicy: createMutation(() => options().updatePolicy, client),
  };
}

function cycleCommandOptions(identity: () => SessionIdentity, queryClient: QueryClient, cycleId: string) {
  /* Every command refreshes on settling rather than on success. A mutation
     that raised can still have committed a control transition — the contract
     says so of run admission in particular — and a workspace that refuses to
     re-read after an error is the one that keeps showing the state before it. */
  const invalidateAll = () => invalidateWorkflowResourceQueries(queryClient);
  return {
    exportEvidence: {
      mutationFn: ({ executionId }: { executionId: string | null }) =>
        deliverNativeArtifact(identity(), {
          source: executionId ? { kind: "execution_evidence", executionId } : { kind: "cycle_evidence", cycleId },
          kind: "artifact",
          action: "save",
          filename: executionId ? `execution-${executionId}.zip` : `cycle-${cycleId}.zip`,
        }),
    },
    archiveCycle: {
      mutationFn: (archived: boolean) => setNativeCycleArchived(identity(), cycleId, archived),
      onSettled: invalidateAll,
    },
    archiveExecution: {
      mutationFn: ({ archived, executionId }: { archived: boolean; executionId: string }) =>
        setNativeExecutionArchived(identity(), executionId, archived),
      onSettled: invalidateAll,
    },
    /* A rename refreshes on settling like every command: lists, tabs and the
       sidebar read the label the runtime now holds. */
    renameCycle: {
      mutationFn: (label: string) => renameNativeTestCycle(identity(), cycleId, label),
      onSettled: invalidateAll,
    },
    renameExecution: {
      mutationFn: ({ executionId, label }: { executionId: string; label: string }) =>
        renameNativeTestExecution(identity(), executionId, label),
      onSettled: invalidateAll,
    },
    rerunCycle: {
      mutationFn: () => rerunNativeTestCycle(identity(), cycleId),
      onSettled: invalidateAll,
    },
    deleteCase: {
      mutationFn: (caseId: string) => deleteNativeTestCycleCases(identity(), cycleId, [caseId]),
      onSettled: invalidateAll,
    },
    downloadReport: downloadReportOptions(identity, cycleId),
    pauseExecution: {
      mutationFn: (executionId: string) => pauseNativeTestExecution(identity(), executionId),
      onSettled: invalidateAll,
    },
    resumeExecution: {
      mutationFn: (executionId: string) => resumeNativeTestExecution(identity(), executionId),
      onSettled: invalidateAll,
    },
    stopExecution: {
      mutationFn: (executionId: string) => stopNativeTestExecution(identity(), executionId),
      onSettled: invalidateAll,
    },
    cancelExecution: {
      mutationFn: (executionId: string) => cancelNativeTestExecution(identity(), executionId),
      onSettled: invalidateAll,
    },
    startExecution: {
      mutationFn: (executionId: string) => startNativeTestExecution(identity(), executionId),
      onSettled: invalidateAll,
    },
    appendAttempt: appendCaseAttemptOptions(identity, queryClient, cycleId),
    sealCycle: {
      mutationFn: () => sealNativeTestCycle(identity(), cycleId),
      onSettled: invalidateAll,
    },
    updatePolicy: {
      mutationFn: ({ continueAutomatically }: { continueAutomatically: boolean }) =>
        updateNativeTestCyclePolicy(identity(), cycleId, continueAutomatically),
      onSettled: invalidateAll,
    },
  };
}

export function appendCaseAttemptOptions(identity: () => SessionIdentity, client: QueryClient, cycleId: string) {
  return {
    mutationFn: (caseId: string) => appendNativeCaseAttempt(identity(), cycleId, caseId),
    onSettled: () => invalidateWorkflowResourceQueries(client),
  };
}

/** Create One Frozen Report, Then Save Its Retained File. */
export function downloadReportOptions(identity: () => SessionIdentity, cycleId: string) {
  return {
    mutationFn: ({ executionId, label }: { executionId: string | null; label: string }) =>
      downloadNativeReport(
        identity(),
        executionId ? { kind: "execution_report", executionId } : { kind: "cycle_report", cycleId },
        reportFilename(label),
      ),
  };
}
