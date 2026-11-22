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

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/context";
import type { SessionIdentity } from "@/lib/rpc/transport";
import { deliverNativeArtifact } from "@/lib/api/native-delivery";
import {
  appendNativeCaseAttempt,
  cancelNativeTestExecution,
  deleteNativeTestCycleCases,
  fetchNativeCycleLabel,
  fetchNativeTestCycleDetail,
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
import { downloadNativeReport, type NativeReportSource } from "@/lib/api/native-reports";
import { reportFilename } from "@/features/cycle/cycle-workspace-model";
import { invalidateWorkflowResourceQueries } from "./server-state";
import { workflowQueryKeys } from "./workflow-query-keys";

/* === Private Functions === */

/** The identity a live read needs. */
function identityOf(session: ReturnType<typeof useNativeSessionOptional>) {
  if (!session) {
    throw new Error("A connected native backend is required.");
  }
  return session.identity;
}

/** Keys stay generation-scoped live, and constant while nothing is fetched. */
function generationOf(session: ReturnType<typeof useNativeSessionOptional>) {
  return session?.identity.generation ?? "disconnected";
}

/* === Query Hooks === */

/** The Cycle's Cases and Every Attempt, Read to Completion. */
export function cycleDetailQueryOptions(identity: SessionIdentity | undefined, cycleId: string) {
  return {
    queryFn: () => {
      if (!identity) {
        throw new Error("Connect to a backend before reading cycle details.");
      }
      return fetchNativeTestCycleDetail(identity, cycleId);
    },
    queryKey: [...workflowQueryKeys.cycleDetail(cycleId), identity?.generation ?? "disconnected"],
    staleTime: Number.POSITIVE_INFINITY,
  };
}

export function useCycleDetailQuery(cycleId: string, enabled = true) {
  const session = useNativeSessionOptional();
  return useQuery({
    ...cycleDetailQueryOptions(session?.identity, cycleId),
    enabled: enabled && session !== null,
  });
}

/** The live runtime as this cycle sees it, never another cycle's. */
export function useCycleRuntimeShellQuery(cycleId: string) {
  const session = useNativeSessionOptional();
  return useQuery({
    enabled: session !== null,
    queryFn: () => fetchNativeRuntimeShell(identityOf(session), cycleId),
    queryKey: [...workflowQueryKeys.runtimeShell(), cycleId, generationOf(session)],
  });
}

/** The label of the cycle this one reruns for its lineage link. */
export function useCycleLabelQuery(cycleId: string | null) {
  const session = useNativeSessionOptional();
  return useQuery({
    enabled: cycleId !== null && session !== null,
    queryFn: () => fetchNativeCycleLabel(identityOf(session), cycleId ?? ""),
    queryKey: [...workflowQueryKeys.cycleDetail(cycleId ?? ""), "label", generationOf(session)],
  });
}

export function useCyclePendingPromptsQuery(cycleId: string) {
  const session = useNativeSessionOptional();
  return useQuery({
    enabled: session !== null,
    queryFn: () => fetchNativePendingPrompts(identityOf(session), { cycleId }),
    queryKey: [...workflowQueryKeys.prompts(), cycleId, generationOf(session)],
  });
}

/* === Mutation Hooks === */

export function useCycleCommands(cycleId: string) {
  const connection = useNativeSessionOptional();
  const session = () => {
    if (!connection) {
      throw new Error("A connected native backend is required.");
    }
    return connection;
  };
  const queryClient = useQueryClient();
  /* Every command refreshes on settling rather than on success. A mutation
     that raised can still have committed a control transition — the contract
     says so of run admission in particular — and a workspace that refuses to
     re-read after an error is the one that keeps showing the state before it. */
  const invalidateAll = () => invalidateWorkflowResourceQueries(queryClient);
  const reportSource = (executionId: string | null): NativeReportSource =>
    executionId ? { kind: "execution_report", executionId } : { kind: "cycle_report", cycleId };

  return {
    exportEvidence: useMutation({
      mutationFn: ({ executionId }: { executionId: string | null }) =>
        deliverNativeArtifact(session().identity, {
          source: executionId ? { kind: "execution_evidence", executionId } : { kind: "cycle_evidence", cycleId },
          kind: "artifact",
          action: "save",
          filename: executionId ? `execution-${executionId}.zip` : `cycle-${cycleId}.zip`,
        }),
    }),
    archiveCycle: useMutation({
      mutationFn: (archived: boolean) => setNativeCycleArchived(session().identity, cycleId, archived),
      onSettled: invalidateAll,
    }),
    archiveExecution: useMutation({
      mutationFn: ({ archived, executionId }: { archived: boolean; executionId: string }) =>
        setNativeExecutionArchived(session().identity, executionId, archived),
      onSettled: invalidateAll,
    }),
    /* A rename refreshes on settling like every command: lists, tabs and the
       sidebar read the label the runtime now holds. */
    renameCycle: useMutation({
      mutationFn: (label: string) => renameNativeTestCycle(session().identity, cycleId, label),
      onSettled: invalidateAll,
    }),
    renameExecution: useMutation({
      mutationFn: ({ executionId, label }: { executionId: string; label: string }) =>
        renameNativeTestExecution(session().identity, executionId, label),
      onSettled: invalidateAll,
    }),
    rerunCycle: useMutation({
      mutationFn: () => rerunNativeTestCycle(session().identity, cycleId),
      onSettled: invalidateAll,
    }),
    deleteCase: useMutation({
      mutationFn: (caseId: string) => deleteNativeTestCycleCases(session().identity, cycleId, [caseId]),
      onSettled: invalidateAll,
    }),
    /* Create One Frozen Report, Then Save Its Retained File. */
    downloadReport: useMutation({
      mutationFn: async ({ executionId, label }: { executionId: string | null; label: string }) =>
        downloadNativeReport(session().identity, reportSource(executionId), reportFilename(label)),
    }),
    pauseExecution: useMutation({
      mutationFn: (executionId: string) => pauseNativeTestExecution(session().identity, executionId),
      onSettled: invalidateAll,
    }),
    resumeExecution: useMutation({
      mutationFn: (executionId: string) => resumeNativeTestExecution(session().identity, executionId),
      onSettled: invalidateAll,
    }),
    stopExecution: useMutation({
      mutationFn: (executionId: string) => stopNativeTestExecution(session().identity, executionId),
      onSettled: invalidateAll,
    }),
    cancelExecution: useMutation({
      mutationFn: (executionId: string) => cancelNativeTestExecution(session().identity, executionId),
      onSettled: invalidateAll,
    }),
    startExecution: useMutation({
      mutationFn: (executionId: string) => startNativeTestExecution(session().identity, executionId),
      onSettled: invalidateAll,
    }),
    appendAttempt: useMutation({
      mutationFn: (caseId: string) => appendNativeCaseAttempt(session().identity, cycleId, caseId),
      onSettled: invalidateAll,
    }),
    sealCycle: useMutation({
      mutationFn: () => sealNativeTestCycle(session().identity, cycleId),
      onSettled: invalidateAll,
    }),
    updatePolicy: useMutation({
      mutationFn: ({ continueAutomatically }: { continueAutomatically: boolean }) =>
        updateNativeTestCyclePolicy(session().identity, cycleId, continueAutomatically),
      onSettled: invalidateAll,
    }),
  };
}

/** Append A Fresh Attempt And Select It For Explicit Start */
export function useRerunCase(cycleId: string) {
  const commands = useCycleCommands(cycleId);
  return {
    pending: commands.appendAttempt.isPending,
    rerun: async (caseId: string): Promise<string | null> => {
      return commands.appendAttempt.mutateAsync(caseId);
    },
  };
}
