/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { type QueryClient, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

/* === Local Imports === */

import {
  useNativeSession,
  useNativeSessionOptional,
} from "@/app/native-session/context";
import {
  createNativeNote,
  editNativeNote,
  fetchNativeHealth,
  fetchNativeNotes,
} from "@/lib/api/native-annotations";
import { fetchNativeArtifacts, fetchNativeExecutionFiles } from "@/lib/api/native-content";
import { fetchNativeDocument, mapNativeDocument } from "@/lib/api/native-documents";
import { fetchNativeCycleRecords, fetchNativeRecords } from "@/lib/api/native-evidence";
import { fetchNativeTestExecutionDetail } from "@/lib/api/native-execution";
import { fetchNativeCycleHistory } from "@/lib/api/native-history";
import { fetchNativeDiagnostics, fetchNativeTraceability } from "@/lib/api/native-observation";
import { fetchNativePendingPrompts, respondNativePrompt } from "@/lib/api/native-prompts";
import type { RecordScope } from "@/lib/types";
import { documentRetry, documentRetryDelay } from "./document-query-model";
import { frozenQuery, useCycleSnapshotSource } from "./snapshot-source";

/* === Constants === */

/** How long an unobserved evidence read stays cached; records and documents are large. */
const EVIDENCE_GC_MS = 30_000;

/* === Types === */

type PromptResponseVariables = {
  action: string;
  correlationId: string;
  values: Record<string, unknown>;
};

type AddNoteVariables = {
  scope?: RecordScope | null;
  text: string;
};

type EditNoteVariables = {
  noteId: string;
  text: string;
};

/* === Query Keys === */

export const workflowQueryKeys = {
  all: ["workflow"] as const,
  artifacts: () => [...workflowQueryKeys.all, "artifacts"] as const,
  cycleDetails: () => [...workflowQueryKeys.all, "cycles", "detail"] as const,
  cycleDetail: (cycleId: string | null = null) =>
    [...workflowQueryKeys.all, "cycles", "detail", cycleId ?? "current"] as const,
  cycleRecords: (cycleId: string | null) =>
    [...workflowQueryKeys.all, "cycles", "records", cycleId ?? ""] as const,
  cycleHistory: () => [...workflowQueryKeys.all, "cycles", "history"] as const,
  diagnostics: () => [...workflowQueryKeys.all, "diagnostics"] as const,
  records: (limit = 500) => [...workflowQueryKeys.all, "records", { limit }] as const,
  executionDocument: ({
    cycleId,
    executionId,
  }: {
    cycleId: string | null;
    executionId: string | null;
  }) =>
    [...workflowQueryKeys.all, "documents", "execution", cycleId ?? "", executionId ?? ""] as const,
  executionDetails: () => [...workflowQueryKeys.all, "execution-detail"] as const,
  executionDetail: (executionId: string | null = null) =>
    [...workflowQueryKeys.all, "execution-detail", executionId ?? ""] as const,
  executionFiles: (executionId: string | null) =>
    [...workflowQueryKeys.all, "execution-files", executionId ?? ""] as const,
  health: () => [...workflowQueryKeys.all, "health"] as const,
  notes: () => [...workflowQueryKeys.all, "notes"] as const,
  prompts: () => [...workflowQueryKeys.all, "prompts"] as const,
  runtimeShell: () => [...workflowQueryKeys.all, "runtime-shell"] as const,
  traceability: (scope?: RecordScope | null) =>
    scope
      ? ([...workflowQueryKeys.all, "traceability", scope] as const)
      : ([...workflowQueryKeys.all, "traceability"] as const),
};

/* === Private Functions === */

/** The connection a live read needs. Unreachable while a snapshot stands in. */
function requireSession(session: ReturnType<typeof useNativeSessionOptional>) {
  if (!session) throw new Error("A connected native backend is required.");
  return session;
}

/** The identity a live read needs. Unreachable while a snapshot stands in. */
function identityOf(session: ReturnType<typeof useNativeSessionOptional>) {
  return requireSession(session).identity;
}

/** Keys stay generation-scoped live, and constant while nothing is fetched. */
function generationOf(session: ReturnType<typeof useNativeSessionOptional>) {
  return session?.identity.generation ?? "frozen";
}

/* === Query Hooks === */

export function useDiagnosticsQuery() {
  const session = useNativeSession();
  // Diagnostics only feeds slow-moving facts (server version, process
  // stats, startup doctor); live health arrives over the runtime stream.
  return useQuery({
    queryFn: () => fetchNativeDiagnostics(session.identity),
    queryKey: [...workflowQueryKeys.diagnostics(), session.identity.generation],
    refetchInterval: 30_000,
    staleTime: 25_000,
  });
}

export function useRecordsQuery(enabled: boolean, scope: RecordScope | null) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const query = useQuery({
    enabled: enabled && session !== null,
    queryFn: () => fetchNativeRecords(identityOf(session), { scope }),
    queryKey: [...workflowQueryKeys.records(), generationOf(session), scope],
    gcTime: EVIDENCE_GC_MS,
    staleTime: 1_000,
  });
  return snapshot ? frozenQuery(snapshot.records) : query;
}

export function usePendingPromptsQuery(enabled: boolean) {
  const session = useNativeSession();
  return useQuery({
    enabled,
    queryFn: () => fetchNativePendingPrompts(session.identity),
    queryKey: [...workflowQueryKeys.prompts(), session.identity.generation],
    staleTime: 500,
  });
}

export function useNotesQuery(enabled: boolean, scope: RecordScope) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const query = useQuery({
    enabled: enabled && session !== null && !!scope.test_cycle_id,
    queryFn: () => fetchNativeNotes(identityOf(session), scope),
    queryKey: [...workflowQueryKeys.notes(), generationOf(session), scope],
    staleTime: 1_000,
  });
  return snapshot ? frozenQuery(snapshot.notes) : query;
}

export function useHealthQuery(enabled: boolean, scope: RecordScope) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const query = useQuery({
    enabled: enabled && session !== null && !!scope.test_cycle_id,
    queryFn: () => fetchNativeHealth(identityOf(session), scope),
    queryKey: [...workflowQueryKeys.health(), generationOf(session), scope],
    staleTime: 1_000,
  });
  return snapshot ? frozenQuery(snapshot.health) : query;
}

export function useArtifactsQuery(enabled: boolean) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const query = useQuery({
    enabled: enabled && session !== null,
    queryFn: () => fetchNativeArtifacts(identityOf(session)),
    queryKey: [...workflowQueryKeys.artifacts(), generationOf(session)],
    staleTime: 1_000,
  });
  return snapshot ? frozenQuery(snapshot.artifacts) : query;
}

export function useTraceabilityQuery(enabled: boolean, scope: RecordScope | null) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const query = useQuery({
    enabled: enabled && session !== null,
    queryFn: () => fetchNativeTraceability(identityOf(session), scope),
    queryKey: [...workflowQueryKeys.traceability(scope), generationOf(session)],
    gcTime: EVIDENCE_GC_MS,
    staleTime: 1_000,
  });
  return snapshot ? frozenQuery(snapshot.traceability(scope?.test_execution_id ?? null)) : query;
}

export function useCycleHistoryQuery() {
  const session = useNativeSession();
  return useQuery({
    queryFn: () => fetchNativeCycleHistory(session.identity),
    queryKey: [...workflowQueryKeys.cycleHistory(), session.identity.generation],
    staleTime: 15_000,
  });
}

export function useExecutionDetailQuery(executionId: string | null, enabled: boolean) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const query = useQuery({
    enabled: Boolean(executionId) && enabled && session !== null,
    queryFn: () => fetchNativeTestExecutionDetail(identityOf(session), executionId ?? ""),
    queryKey: [...workflowQueryKeys.executionDetail(executionId), generationOf(session)],
    staleTime: 1_000,
  });
  return snapshot
    ? frozenQuery(
        snapshot.executions.find((item) => item.test_execution_id === executionId) ?? undefined,
      )
    : query;
}

export function useCycleRecordsQuery(cycleId: string | null, enabled = true) {
  const session = useNativeSession();
  return useQuery({
    enabled: Boolean(cycleId) && enabled,
    queryFn: () => fetchNativeCycleRecords(session.identity, cycleId ?? ""),
    queryKey: [...workflowQueryKeys.cycleRecords(cycleId), session.identity.generation],
    gcTime: EVIDENCE_GC_MS,
    staleTime: 30_000,
  });
}

export function useExecutionFilesQuery(executionId: string | null, enabled: boolean) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  // The execution_files topic is a hot counter (it ticks with every support
  // log append), so key on the slow artifacts revision and refresh the file
  // listing on a gentle interval instead of once per pushed frame.
  const query = useQuery({
    enabled: Boolean(executionId) && enabled && session !== null,
    queryFn: () => fetchNativeExecutionFiles(identityOf(session), executionId ?? ""),
    queryKey: [...workflowQueryKeys.executionFiles(executionId), generationOf(session)],
    refetchInterval: 5_000,
    staleTime: 4_000,
  });
  return snapshot ? frozenQuery(snapshot.executionFiles(executionId)) : query;
}

export function useExecutionDocumentQuery({
  cycleId,
  executionId,
}: {
  cycleId: string | null;
  executionId: string | null;
}) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const query = useQuery({
    enabled: Boolean(cycleId && executionId) && session !== null,
    queryFn: ({ signal }) =>
      fetchNativeDocument(
        nativeDocumentSession(requireSession(session)),
        {
          cycleId: cycleId ?? "",
          documentId: "default",
          executionId: executionId ?? "",
          epoch: nativeEpoch(requireSession(session)),
        },
        signal,
      ).then((document) => mapNativeDocument(document)),
    queryKey: [
      ...workflowQueryKeys.executionDocument({ cycleId, executionId }),
      generationOf(session),
    ],
    gcTime: EVIDENCE_GC_MS,
    retry: documentRetry,
    retryDelay: documentRetryDelay,
    staleTime: 1_000,
  });
  return snapshot && executionId ? frozenQuery(snapshot.document(executionId) ?? undefined) : query;
}

/**
 * The cycle's own document: the host-composed records, artifacts, media and
 * cycle-level plots, read through the same document route with no execution
 * in its scope. A frozen report answers from its embedded cycle capture.
 */
export function useCycleDocumentQuery(cycleId: string | null, enabled: boolean) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const query = useQuery({
    enabled: Boolean(cycleId) && enabled && session !== null,
    queryFn: ({ signal }) =>
      fetchNativeDocument(
        nativeDocumentSession(requireSession(session)),
        {
          cycleId: cycleId ?? "",
          documentId: "default",
          executionId: "",
          epoch: nativeEpoch(requireSession(session)),
        },
        signal,
      ).then((document) => mapNativeDocument(document)),
    queryKey: [
      ...workflowQueryKeys.executionDocument({ cycleId, executionId: null }),
      generationOf(session),
    ],
    gcTime: EVIDENCE_GC_MS,
    retry: documentRetry,
    retryDelay: documentRetryDelay,
    staleTime: 1_000,
  });
  return snapshot && enabled ? frozenQuery(snapshot.document(null) ?? undefined) : query;
}

/* === Invalidation === */

export async function invalidateWorkflowResourceQueries(queryClient: QueryClient) {
  await Promise.all([
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.runtimeShell() }),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.cycleDetails() }),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.executionDetails() }),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.records() }),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.prompts() }),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.notes() }),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.health() }),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.artifacts() }),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.traceability() }),
  ]);
}

export async function invalidateRuntimeProjectionQueries(queryClient: QueryClient) {
  const keepActiveRefetch = { cancelRefetch: false } as const;
  await Promise.all([
    queryClient.invalidateQueries(
      { queryKey: workflowQueryKeys.cycleDetails() },
      keepActiveRefetch,
    ),
    queryClient.invalidateQueries(
      { queryKey: workflowQueryKeys.executionDetails() },
      keepActiveRefetch,
    ),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.records() }, keepActiveRefetch),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.prompts() }, keepActiveRefetch),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.notes() }, keepActiveRefetch),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.health() }, keepActiveRefetch),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.artifacts() }, keepActiveRefetch),
    queryClient.invalidateQueries(
      { queryKey: workflowQueryKeys.traceability() },
      keepActiveRefetch,
    ),
  ]);
}

/**
 * Re-read the open documents. Each read pages every record of its scope and
 * reassembles the document, so a running cycle refreshes them on a throttle
 * rather than on every runtime cursor.
 */
export async function invalidateRuntimeDocuments(queryClient: QueryClient) {
  await queryClient.invalidateQueries(
    { queryKey: [...workflowQueryKeys.all, "documents"] },
    { cancelRefetch: false },
  );
}

export async function invalidateWorkflowDocuments(queryClient: QueryClient) {
  await Promise.all([
    queryClient.invalidateQueries({ queryKey: [...workflowQueryKeys.all, "documents"] }),
    queryClient.invalidateQueries({ queryKey: [...workflowQueryKeys.all, "execution-files"] }),
  ]);
}

/* === Mutation Hooks === */

export function usePromptResponseMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ action, correlationId, values }: PromptResponseVariables) =>
      respondNativePrompt(session.identity, correlationId, action, values),
    onSuccess: async (_data, variables) => {
      queryClient.setQueryData(workflowQueryKeys.prompts(), (current: unknown) =>
        Array.isArray(current)
          ? current.filter((item) => item?.correlation_id !== variables.correlationId)
          : current,
      );
      await queryClient.invalidateQueries({ queryKey: workflowQueryKeys.prompts() });
    },
  });
}

export function useAddNoteMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ scope, text }: AddNoteVariables) =>
      createNativeNote(session.identity, nativeEpoch(session), text, scope),
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: workflowQueryKeys.notes() }),
        queryClient.invalidateQueries({ queryKey: workflowQueryKeys.runtimeShell() }),
      ]);
    },
  });
}

export function useEditNoteMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ noteId, text }: EditNoteVariables) =>
      editNativeNote(session.identity, nativeEpoch(session), noteId, text),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: workflowQueryKeys.notes() });
    },
  });
}

function nativeEpoch(session: ReturnType<typeof useNativeSession>): string {
  const epoch = session.snapshot.connection.backend?.epoch;
  if (!epoch) throw new Error("The connected native backend has no process epoch.");
  return epoch;
}

function nativeDocumentSession(session: ReturnType<typeof useNativeSession>) {
  return { ...session.identity, own: session.own };
}
