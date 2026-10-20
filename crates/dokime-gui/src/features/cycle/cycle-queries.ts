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
 *
 * Each read consults `useCycleSnapshotSource` first. A frozen report mounts
 * the same workspace with `live={false}` and a snapshot in that context, so
 * these hooks answer from the snapshot and never reach for a backend that is
 * not there. The return shape is identical either way.
 */

/* === External Imports === */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/context";
import { deliverNativeArtifact } from "@/lib/api/native-delivery";
import {
  appendNativeCaseAttempt,
  cancelNativeTestExecution,
  deleteNativeTestCycleCases,
  fetchNativeTestCycleDetail,
  pauseNativeTestExecution,
  renameNativeTestCycle,
  renameNativeTestExecution,
  reopenNativeTestCycle,
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
import {
  downloadNativeCycleReport,
  downloadNativeExecutionReport,
  downloadNativeReport,
  type NativeReportHtml,
  openNativeReport,
} from "@/lib/api/native-reports";
import { reportFilename } from "./cycle-workspace-model";
import { invalidateWorkflowResourceQueries, workflowQueryKeys } from "./server-state";
import { frozenQuery, useCycleSnapshotSource } from "./snapshot-source";

/* === Constants === */

/** One identity, so a frozen prompt list never re-renders the banner. */
const EMPTY_PROMPTS: Awaited<ReturnType<typeof fetchNativePendingPrompts>> = [];

/* === Private Functions === */

/** The identity a live read needs. Unreachable while a snapshot stands in. */
function identityOf(session: ReturnType<typeof useNativeSessionOptional>) {
  if (!session) throw new Error("A connected native backend is required.");
  return session.identity;
}

/** Keys stay generation-scoped live, and constant while nothing is fetched. */
function generationOf(session: ReturnType<typeof useNativeSessionOptional>) {
  return session?.identity.generation ?? "frozen";
}

/* === Query Hooks === */

/** The cycle's cases and every attempt of each, read to completion. */
export function useCycleDetailQuery(cycleId: string, enabled = true) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const query = useQuery({
    enabled: enabled && session !== null,
    queryFn: () => fetchNativeTestCycleDetail(identityOf(session), cycleId),
    queryKey: [...workflowQueryKeys.cycleDetail(cycleId), generationOf(session)],
    staleTime: 1_000,
  });
  return snapshot ? frozenQuery({ plan: snapshot.plan, executions: snapshot.executions }) : query;
}

/** The live runtime as this cycle sees it, never another cycle's. */
export function useCycleRuntimeShellQuery(cycleId: string) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const query = useQuery({
    enabled: session !== null,
    queryFn: () => fetchNativeRuntimeShell(identityOf(session), cycleId),
    queryKey: [...workflowQueryKeys.runtimeShell(), cycleId, generationOf(session)],
    staleTime: 1_000,
  });
  return snapshot ? frozenQuery(snapshot.runtimeShell) : query;
}

/* A frozen cycle has no pending prompt by construction: generation waits for
   the cycle to settle, and nobody can answer a question in a static file. */
export function useCyclePendingPromptsQuery(cycleId: string) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const query = useQuery({
    enabled: session !== null,
    queryFn: () => fetchNativePendingPrompts(identityOf(session), { cycleId }),
    queryKey: [...workflowQueryKeys.prompts(), cycleId, generationOf(session)],
    staleTime: 500,
  });
  return snapshot ? frozenQuery(EMPTY_PROMPTS) : query;
}

/* === Mutation Hooks === */

export function useCycleCommands(cycleId: string) {
  /* A frozen report renders no control that fires one of these, so the set is
     built without a connection and asks for one only when a command runs. */
  const connection = useNativeSessionOptional();
  const session = () => {
    if (!connection) throw new Error("A connected native backend is required.");
    return connection;
  };
  const queryClient = useQueryClient();
  /* Every command refreshes on settling rather than on success. A mutation
     that raised can still have committed a control transition — the contract
     says so of run admission in particular — and a workspace that refuses to
     re-read after an error is the one that keeps showing the state before it. */
  const invalidateAll = () => invalidateWorkflowResourceQueries(queryClient);
  const readReport = (executionId: string | null): Promise<NativeReportHtml> => {
    const owner = { ...session().identity, own: session().own };
    return executionId ? downloadNativeExecutionReport(owner, executionId) : downloadNativeCycleReport(owner, cycleId);
  };

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
    copyCycle: useMutation({
      mutationFn: () => reopenNativeTestCycle(session().identity, cycleId),
      onSettled: invalidateAll,
    }),
    deleteCase: useMutation({
      mutationFn: (caseId: string) => deleteNativeTestCycleCases(session().identity, cycleId, [caseId]),
      onSettled: invalidateAll,
    }),
    /* Saves the report Open Report would open, named after its node. */
    downloadReport: useMutation({
      mutationFn: async ({ executionId, label }: { executionId: string | null; label: string }) =>
        downloadNativeReport(await readReport(executionId), reportFilename(label), session().identity),
    }),
    /* The report the cycle, or one execution, has now: the runtime renders
       one from the run so far, reads the sealed one, or renders the sealed
       one again when its file is gone. Opening it never fails for want of a
       seal. */
    openReport: useMutation({
      mutationFn: async (executionId: string | null) => {
        const resource = await readReport(executionId);
        await openNativeReport(resource, session().identity);
        return resource.report.origin?.HarmosVariant.case;
      },
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
      mutationFn: ({ continueAutomatically, continueOnFailure }: { continueAutomatically: boolean; continueOnFailure: boolean }) =>
        updateNativeTestCyclePolicy(session().identity, cycleId, continueAutomatically, continueOnFailure),
      onSettled: invalidateAll,
    }),
  };
}

export type CycleCommands = ReturnType<typeof useCycleCommands>;

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
