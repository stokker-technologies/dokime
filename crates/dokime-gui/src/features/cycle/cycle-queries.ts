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

import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import {
  downloadNativeCycleBundle,
  fetchNativeCycleFolderPath,
  nativeContentSession,
} from "@/lib/api/native-content";
import {
  clearNativeTestCycle,
  deleteNativeTestCycleCases,
  fetchNativeTestCycleDetail,
  pauseNativeTestCycle,
  pauseNativeTestExecution,
  reopenNativeTestCycle,
  rerunNativeTestCase,
  resumeNativeTestCycle,
  resumeNativeTestExecution,
  runNativeQueuedTestExecution,
  runNativeQueuedTestCycle,
  stopNativeTestCycle,
  stopNativeTestExecution,
  updateNativeTestCyclePolicy,
} from "@/lib/api/native-execution";
import { fetchNativeRuntimeShell } from "@/lib/api/native-observation";
import { fetchNativePendingPrompts } from "@/lib/api/native-prompts";
import { downloadNativeCycleReport, downloadNativeReport } from "@/lib/api/native-reports";
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

  return {
    clearCycle: useMutation({
      mutationFn: () => clearNativeTestCycle(session().identity, cycleId),
      onSettled: invalidateAll,
    }),
    copyCycle: useMutation({
      mutationFn: () => reopenNativeTestCycle(session().identity, cycleId),
      onSettled: invalidateAll,
    }),
    deleteCase: useMutation({
      mutationFn: (caseId: string) =>
        deleteNativeTestCycleCases(session().identity, cycleId, [caseId]),
      onSettled: invalidateAll,
    }),
    downloadBundle: useMutation({
      mutationFn: () =>
        downloadNativeCycleBundle(nativeContentSession(session().identity, session().own), cycleId),
    }),
    openFolder: useMutation({
      mutationFn: () => fetchNativeCycleFolderPath(session().identity, cycleId),
    }),
    pauseExecution: useMutation({
      mutationFn: (executionId: string) =>
        pauseNativeTestExecution(session().identity, executionId),
      onSettled: invalidateAll,
    }),
    resumeExecution: useMutation({
      mutationFn: (executionId: string) =>
        resumeNativeTestExecution(session().identity, executionId),
      onSettled: invalidateAll,
    }),
    stopExecution: useMutation({
      mutationFn: (executionId: string) => stopNativeTestExecution(session().identity, executionId),
      onSettled: invalidateAll,
    }),
    pauseCycle: useMutation({
      mutationFn: () => pauseNativeTestCycle(session().identity, cycleId),
      onSettled: invalidateAll,
    }),
    resumeCycle: useMutation({
      mutationFn: () => resumeNativeTestCycle(session().identity, cycleId),
      onSettled: invalidateAll,
    }),
    runCase: useMutation({
      mutationFn: (executionId: string) =>
        runNativeQueuedTestExecution(session().identity, executionId),
      onSettled: invalidateAll,
    }),
    runQueued: useMutation({
      mutationFn: () => runNativeQueuedTestCycle(session().identity, cycleId),
      onSettled: invalidateAll,
    }),
    scheduleRerun: useMutation({
      mutationFn: (caseId: string) => rerunNativeTestCase(session().identity, cycleId, caseId),
      onSettled: invalidateAll,
    }),
    stopCycle: useMutation({
      mutationFn: () => stopNativeTestCycle(session().identity, cycleId),
      onSettled: invalidateAll,
    }),
    updatePolicy: useMutation({
      mutationFn: (continueOnFailure: boolean) =>
        updateNativeTestCyclePolicy(session().identity, cycleId, continueOnFailure),
      onSettled: invalidateAll,
    }),
    writeReport: useMutation({
      mutationFn: async () => {
        const resource = await downloadNativeCycleReport(
          { ...session().identity, own: session().own },
          cycleId,
          "cycle.summary",
        );
        return downloadNativeReport(
          resource,
          `${resource.report.identifier}.html`,
          session().identity,
        );
      },
    }),
  };
}

export type CycleCommands = ReturnType<typeof useCycleCommands>;
