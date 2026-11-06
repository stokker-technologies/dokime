/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { type QueryClient, useMutation, useQueries, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useMemo, useState } from "react";

/* === Local Imports === */

import { useNativeSession, useNativeSessionOptional } from "@/app/native-session/context";
import { createNativeNote, editNativeNote, fetchNativeHealth, fetchNativeNotes } from "@/lib/api/native-annotations";
import { fetchNativeExecutionFiles } from "@/lib/api/native-content";
import {
  type CompletedDocument,
  type NativeDocumentRequest,
  type NativeDocumentSession,
  fetchNativeDocument,
  mapNativeDocument,
  mapNativeDocumentCard,
  nativeDocumentCardKey,
} from "@/lib/api/native-documents";
import {
  appendRecordTail,
  fetchNativeRecordTail,
  type NativeEvidenceSession,
  openNativeRecordStream,
  RECORD_TAIL,
  type RecordTail,
} from "@/lib/api/native-evidence";
import { fetchNativeTestExecutionDetail } from "@/lib/api/native-execution";
import { fetchNativeTraceability } from "@/lib/api/native-observation";
import { respondNativePrompt } from "@/lib/api/native-prompts";
import type { DocumentViewCard, DocumentViewPayload, RecordScope } from "@/lib/types";
import { failureOf } from "@/lib/notification-details";
import { documentRetry, documentRetryDelay, mergeDocumentProgress } from "./document-query-model";
import { frozenQuery, useCycleSnapshotSource } from "./snapshot-source";

/* === Constants === */

const GUI_CAPTURED_DOCUMENTS = 16;
const GUI_EVIDENCE_GC_MS = 30_000;
const GUI_PAGE_ITEMS = 500;
const GUI_RECONNECT_MS = 1_000;

/** How long an unobserved evidence read stays cached; records and documents are large. */
const EVIDENCE_GC_MS = GUI_EVIDENCE_GC_MS;

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
  cycleDetail: (cycleId: string | null = null) => [...workflowQueryKeys.all, "cycles", "detail", cycleId ?? "current"] as const,
  cycleHistory: () => [...workflowQueryKeys.all, "cycles", "history"] as const,
  records: (limit = GUI_PAGE_ITEMS) => [...workflowQueryKeys.all, "records", { limit }] as const,
  executionDocument: ({ cycleId, executionId }: { cycleId: string | null; executionId: string | null }) =>
    [...workflowQueryKeys.all, "documents", "execution", cycleId ?? "", executionId ?? ""] as const,
  executionDetails: () => [...workflowQueryKeys.all, "execution-detail"] as const,
  executionDetail: (executionId: string | null = null) => [...workflowQueryKeys.all, "execution-detail", executionId ?? ""] as const,
  executionFiles: (executionId: string | null) => [...workflowQueryKeys.all, "execution-files", executionId ?? ""] as const,
  health: () => [...workflowQueryKeys.all, "health"] as const,
  notes: () => [...workflowQueryKeys.all, "notes"] as const,
  prompts: () => [...workflowQueryKeys.all, "prompts"] as const,
  runtimeShell: () => [...workflowQueryKeys.all, "runtime-shell"] as const,
  traceability: (scope?: RecordScope | null) =>
    scope ? ([...workflowQueryKeys.all, "traceability", scope] as const) : ([...workflowQueryKeys.all, "traceability"] as const),
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

/**
 * A scope's latest records: the tail read once, then appended from the
 * scope's record stream.
 *
 * The runtime stream no longer invalidates records, so an open cycle never
 * replays its history on a cursor. Every holder of one scope shares one
 * record stream (`holdRecordTail`); a stream that fails re-reads the tail and
 * continues from its new cursor.
 */
export function useRecordsQuery(enabled: boolean, scope: RecordScope | null) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const queryClient = useQueryClient();
  /* Callers build their scope inline, so the key and the stream follow its
     content rather than its identity. */
  const scopeKey = JSON.stringify(scope);
  const generation = generationOf(session);
  // biome-ignore lint/correctness/useExhaustiveDependencies: `scopeKey` is `scope`'s content; the scope follows it, not the object.
  const stableScope = useMemo(() => scope, [scopeKey]);
  const queryKey = useMemo(() => [...workflowQueryKeys.records(), generation, stableScope] as const, [generation, stableScope]);
  const query = useQuery({
    enabled: enabled && session !== null && !snapshot,
    queryFn: () => fetchNativeRecordTail(identityOf(session), stableScope),
    queryKey,
    gcTime: EVIDENCE_GC_MS,
    select: selectRecords,
    staleTime: Number.POSITIVE_INFINITY,
  });
  const loaded = query.isSuccess;
  useEffect(() => {
    if (!enabled || !session || snapshot || !loaded) return undefined;
    return holdRecordTail(queryClient, { ...session.identity, own: session.own }, queryKey, stableScope);
  }, [enabled, loaded, queryClient, queryKey, session, snapshot, stableScope]);
  return snapshot ? frozenQuery(snapshot.records) : query;
}

export function useNotesQuery(enabled: boolean, scope: RecordScope) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const query = useQuery({
    enabled: enabled && session !== null && !!scope.test_cycle_id,
    queryFn: () => fetchNativeNotes(identityOf(session), scope),
    queryKey: [...workflowQueryKeys.notes(), generationOf(session), scope],
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
  });
  return snapshot ? frozenQuery(snapshot.health) : query;
}

export function useTraceabilityQuery(enabled: boolean, scope: RecordScope | null) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const query = useQuery({
    enabled: enabled && session !== null,
    queryFn: () => fetchNativeTraceability(identityOf(session), scope),
    queryKey: [...workflowQueryKeys.traceability(scope), generationOf(session)],
    gcTime: EVIDENCE_GC_MS,
  });
  return snapshot ? frozenQuery(snapshot.traceability(scope?.test_execution_id ?? null)) : query;
}

export function useExecutionDetailQuery(executionId: string | null, enabled: boolean) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const query = useQuery({
    enabled: Boolean(executionId) && enabled && session !== null,
    queryFn: () => fetchNativeTestExecutionDetail(identityOf(session), executionId ?? ""),
    queryKey: [...workflowQueryKeys.executionDetail(executionId), generationOf(session)],
  });
  return snapshot ? frozenQuery(snapshot.executions.find((item) => item.test_execution_id === executionId) ?? undefined) : query;
}

export function useExecutionFilesQuery(executionId: string | null, enabled: boolean) {
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  // Runtime Changes Refresh Files Alongside Documents Through The Shared Throttle.
  const query = useQuery({
    enabled: Boolean(executionId) && enabled && session !== null,
    queryFn: () => fetchNativeExecutionFiles(identityOf(session), executionId ?? ""),
    queryKey: [...workflowQueryKeys.executionFiles(executionId), generationOf(session)],
    staleTime: Infinity,
  });
  return snapshot ? frozenQuery(snapshot.executionFiles(executionId)) : query;
}

export function useExecutionDocumentQuery({ cycleId, executionId }: { cycleId: string | null; executionId: string | null }) {
  const queryClient = useQueryClient();
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const cardKey = [...workflowQueryKeys.executionDocument({ cycleId, executionId }), generationOf(session)];
  const navigation = useDocumentNavigation(JSON.stringify(cardKey));
  const captureKey = [...cardKey, "pages", navigation.token ?? ""];
  const query = useQuery({
    enabled: Boolean(cycleId && executionId) && session !== null && !snapshot,
    queryFn: ({ queryKey, signal }) =>
      capturedDocument(
        queryKey,
        (previous, publish) =>
          fetchNativeDocument(
            nativeDocumentSession(requireSession(session)),
            {
              cycleId: cycleId ?? "",
              documentId: "default",
              executionId: executionId ?? "",
              epoch: nativeEpoch(requireSession(session)),
              structureOnly: true,
              page: navigation.token,
            },
            signal,
            previous,
            publish,
          ),
        (document) => queryClient.setQueryData(queryKey, document),
      ),
    queryKey: captureKey,
    gcTime: EVIDENCE_GC_MS,
    retry: documentRetry,
    retryDelay: documentRetryDelay,
  });
  const cards = useDocumentCards({
    document: query.data,
    cardKey: [...cardKey, "view", navigation.revision],
    queryKey: captureKey,
    request: { cycleId: cycleId ?? "", executionId: executionId ?? "", documentId: "default", epoch: session ? nativeEpoch(session) : "" },
    session: session ? nativeDocumentSession(session) : null,
    retryRoot: () => query.refetch(),
    enabled: Boolean(cycleId && executionId) && !snapshot,
  });
  return snapshot && executionId
    ? {
        ...frozenQuery(snapshot.document(executionId) ?? undefined),
        retryCard: cards.retryCard,
        navigation: navigation.controls(undefined, false),
      }
    : { ...query, data: cards.data, retryCard: cards.retryCard, navigation: navigation.controls(cards.page, query.isFetching) };
}

/**
 * The cycle's own document: the host-composed records, artifacts, media and
 * cycle-level plots, read through the same document route with no execution
 * in its scope. A frozen report answers from its embedded cycle capture.
 */
export function useCycleDocumentQuery(cycleId: string | null, enabled: boolean) {
  const queryClient = useQueryClient();
  const snapshot = useCycleSnapshotSource();
  const session = useNativeSessionOptional();
  const cardKey = [...workflowQueryKeys.executionDocument({ cycleId, executionId: null }), generationOf(session)];
  const navigation = useDocumentNavigation(JSON.stringify(cardKey));
  const captureKey = [...cardKey, "pages", navigation.token ?? ""];
  const query = useQuery({
    enabled: Boolean(cycleId) && enabled && session !== null && !snapshot,
    queryFn: ({ queryKey, signal }) =>
      capturedDocument(
        queryKey,
        (previous, publish) =>
          fetchNativeDocument(
            nativeDocumentSession(requireSession(session)),
            {
              cycleId: cycleId ?? "",
              documentId: "default",
              executionId: "",
              epoch: nativeEpoch(requireSession(session)),
              structureOnly: true,
              page: navigation.token,
            },
            signal,
            previous,
            publish,
          ),
        (document) => queryClient.setQueryData(queryKey, document),
      ),
    queryKey: captureKey,
    gcTime: EVIDENCE_GC_MS,
    retry: documentRetry,
    retryDelay: documentRetryDelay,
  });
  const cards = useDocumentCards({
    document: query.data,
    cardKey: [...cardKey, "view", navigation.revision],
    queryKey: captureKey,
    request: { cycleId: cycleId ?? "", executionId: "", documentId: "default", epoch: session ? nativeEpoch(session) : "" },
    session: session ? nativeDocumentSession(session) : null,
    retryRoot: () => query.refetch(),
    enabled: Boolean(cycleId) && enabled && !snapshot,
  });
  return snapshot && enabled
    ? {
        ...frozenQuery(snapshot.document(null) ?? undefined),
        retryCard: cards.retryCard,
        navigation: navigation.controls(undefined, false),
      }
    : { ...query, data: cards.data, retryCard: cards.retryCard, navigation: navigation.controls(cards.page, query.isFetching) };
}

/* === Invalidation === */

export async function invalidateWorkflowResourceQueries(queryClient: QueryClient) {
  await Promise.all([
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.runtimeShell() }),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.cycleDetails() }),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.executionDetails() }),
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
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.cycleDetails() }, keepActiveRefetch),
    // A sealed execution never changes again: its detail is not re-read.
    queryClient.invalidateQueries(
      {
        predicate: (query) => !(query.state.data as { sealed?: boolean } | undefined)?.sealed,
        queryKey: workflowQueryKeys.executionDetails(),
      },
      keepActiveRefetch,
    ),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.prompts() }, keepActiveRefetch),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.notes() }, keepActiveRefetch),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.health() }, keepActiveRefetch),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.artifacts() }, keepActiveRefetch),
    queryClient.invalidateQueries({ queryKey: workflowQueryKeys.traceability() }, keepActiveRefetch),
  ]);
}

/**
 * Refresh structure and card queries independently. Active card requests keep
 * their cut while completed cards can refresh at the latest durable cut.
 */
export async function invalidateRuntimeDocuments(queryClient: QueryClient) {
  await queryClient.invalidateQueries({ queryKey: [...workflowQueryKeys.all, "documents"] }, { cancelRefetch: false });
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
        Array.isArray(current) ? current.filter((item) => item?.correlation_id !== variables.correlationId) : current,
      );
      await queryClient.invalidateQueries({ queryKey: workflowQueryKeys.prompts() });
    },
  });
}

export function useAddNoteMutation() {
  const session = useNativeSession();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ scope, text }: AddNoteVariables) => createNativeNote(session.identity, nativeEpoch(session), text, scope),
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
    mutationFn: ({ noteId, text }: EditNoteVariables) => editNativeNote(session.identity, nativeEpoch(session), noteId, text),
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

/* === Captured Documents === */

/** The documents last captured, by query: a capture whose frontiers are unchanged answers these. */
const capturedDocuments = new Map<string, { completed?: CompletedDocument; current: CompletedDocument; mapped: DocumentViewPayload }>();

/** The most open documents remembered at once; the oldest is forgotten first. */
const CAPTURED_DOCUMENT_LIMIT = GUI_CAPTURED_DOCUMENTS;

/**
 * Capture a document, handing the capture the one held for this query: an
 * unchanged capture returns the held document, and the held projection is
 * answered unmapped, so nothing downstream re-renders.
 */
async function capturedDocument(
  queryKey: readonly unknown[],
  capture: (previous: CompletedDocument | null, publish: (document: CompletedDocument) => void) => Promise<CompletedDocument>,
  publish: (document: DocumentViewPayload) => void,
): Promise<DocumentViewPayload> {
  const key = JSON.stringify(queryKey);
  const held = capturedDocuments.get(key);
  let progress = held?.mapped;
  const completed = await capture(held?.completed ?? null, (document) => {
    progress = mergeDocumentProgress(mapNativeDocument(document), progress);
    capturedDocuments.set(key, { completed: held?.completed, current: document, mapped: progress });
    publish(progress);
  });
  if (held && completed === held.completed) return held.mapped;
  const mapped = mapNativeDocument(completed);
  capturedDocuments.delete(key);
  capturedDocuments.set(key, { completed, current: completed, mapped });
  for (const oldest of capturedDocuments.keys()) {
    if (capturedDocuments.size <= CAPTURED_DOCUMENT_LIMIT) break;
    capturedDocuments.delete(oldest);
  }
  return mapped;
}

/* === Record Tails === */

function useDocumentNavigation(owner: string) {
  const [state, setState] = useState<{ owner: string; token?: string; previous: string[]; revision: number }>({
    owner,
    previous: [],
    revision: 0,
  });
  const current = state.owner === owner ? state : { owner, previous: [] as string[], token: undefined, revision: 0 };
  return {
    token: current.token,
    revision: current.revision,
    controls: (page: CompletedDocument["page"], busy: boolean) => ({
      hasPrevious: current.previous.length > 0,
      hasNext: Boolean(page?.next),
      fixed: Boolean(current.token),
      busy,
      previous: () => {
        const previous = [...current.previous];
        const token = previous.pop();
        setState({ ...current, token, previous });
      },
      next: () => {
        if (page?.next) setState({ ...current, token: page.next, previous: [...current.previous, page.current] });
      },
      latest: () => setState({ owner, previous: [], revision: current.revision + 1 }),
    }),
  };
}

type CardRead = { document: CompletedDocument; card: DocumentViewCard };

function useDocumentCards({
  document,
  queryKey,
  cardKey,
  request,
  session,
  retryRoot,
  enabled,
}: {
  document: DocumentViewPayload | undefined;
  queryKey: readonly unknown[];
  cardKey: readonly unknown[];
  request: NativeDocumentRequest;
  session: NativeDocumentSession | null;
  retryRoot: () => Promise<unknown>;
  enabled: boolean;
}) {
  const queryClient = useQueryClient();
  const held = capturedDocuments.get(JSON.stringify(queryKey));
  const candidates = (document?.cards ?? []).flatMap((card) => {
    const source = held?.current.bindings.has(card.card_id) ? held.current : held?.completed;
    if (source?.bindings.get(card.card_id)?.result?.HarmosVariant.case !== "Pending") return [];
    const key = [...cardKey, "cards", card.card_id, nativeDocumentCardKey(source, card.card_id)];
    return [{ id: card.card_id, key, page: source.page?.current }];
  });
  const results = useQueries({
    queries: candidates.map(({ id, key, page }) => ({
      queryKey: key,
      enabled: enabled && session !== null,
      gcTime: EVIDENCE_GC_MS,
      retry: documentRetry,
      retryDelay: documentRetryDelay,
      queryFn: async ({ signal }: { signal: AbortSignal }): Promise<CardRead> => {
        if (!session) throw new Error("A connected native backend is required.");
        const previous = queryClient.getQueryData<CardRead>(key);
        const captured = await fetchNativeDocument(session, { ...request, cardIds: [id], page }, signal, previous?.document);
        if (previous && captured === previous.document) return previous;
        return { document: captured, card: mapNativeDocumentCard(captured, id) };
      },
    })),
  });
  const values = new Map(
    results.map((query, index) => {
      const id = candidates[index]?.id;
      const original = document?.cards.find((card) => card.card_id === id);
      if (!original) return [id, undefined] as const;
      if (query.error)
        return [id, { ...(query.data?.card ?? original), loading: false, failure: failureOf(query.error, original.title) }] as const;
      return [id, query.data?.card] as const;
    }),
  );
  const data = document ? { ...document, cards: document.cards.map((card) => values.get(card.card_id) ?? card) } : undefined;
  const retryCard = async (id?: string) => {
    const index = id ? candidates.findIndex((card) => card.id === id) : -1;
    if (index >= 0) {
      await results[index]?.refetch();
      return;
    }
    await Promise.all([retryRoot(), ...results.filter((query) => query.error || query.data?.card.failure).map((query) => query.refetch())]);
  };
  return { data, retryCard, page: held?.current.page };
}

/** One live record stream per scope, shared by every holder of that scope. */
const recordTails = new Map<string, { holders: number; stop: () => void }>();

function selectRecords(tail: RecordTail) {
  return tail.records;
}

/**
 * Keep one scope's tail appended from its record stream while anything holds
 * it. Returns the release; the last release cancels the stream.
 */
function holdRecordTail(
  queryClient: QueryClient,
  session: NativeEvidenceSession,
  queryKey: readonly unknown[],
  scope: RecordScope | null,
): () => void {
  const key = JSON.stringify(queryKey);
  const held = recordTails.get(key);
  if (held) {
    held.holders += 1;
    return () => releaseRecordTail(key);
  }
  let stopped = false;
  let stream: Awaited<ReturnType<typeof openNativeRecordStream>> | undefined;
  let frame: number | null = null;
  const pending = new Map<string, RecordTail["records"][number]>();
  let cursor: RecordTail["cursor"] = null;
  const flush = () => {
    if (frame !== null) window.cancelAnimationFrame(frame);
    frame = null;
    if (stopped || !session.isCurrent()) return;
    const records = [...pending.values()];
    pending.clear();
    const through = cursor;
    cursor = null;
    queryClient.setQueryData<RecordTail>(queryKey, (current) => (current ? appendRecordTail(current, records, through) : current));
  };
  const follow = async () => {
    while (!stopped && session.isCurrent()) {
      const tail = queryClient.getQueryData<RecordTail>(queryKey);
      try {
        stream = await openNativeRecordStream(session, scope, tail?.cursor);
        if (stopped) {
          await stream.cancel();
          return;
        }
        for await (const page of stream) {
          if (stopped) break;
          for (const record of page.records) {
            pending.set(record.record_id, record);
            if (pending.size > RECORD_TAIL) {
              const oldest = pending.keys().next().value;
              if (oldest !== undefined) pending.delete(oldest);
            }
          }
          cursor = page.cursor ?? cursor;
          frame ??= window.requestAnimationFrame(flush);
        }
      } catch {
        // The stream failed: the tail is re-read below and followed again.
      }
      stream = undefined;
      if (stopped || !session.isCurrent()) return;
      flush();
      await queryClient.invalidateQueries({ queryKey, exact: true });
      await new Promise((resume) => window.setTimeout(resume, GUI_RECONNECT_MS));
    }
  };
  recordTails.set(key, {
    holders: 1,
    stop: () => {
      stopped = true;
      if (frame !== null) window.cancelAnimationFrame(frame);
      pending.clear();
      void stream?.cancel().catch(() => undefined);
    },
  });
  void follow();
  return () => releaseRecordTail(key);
}

function releaseRecordTail(key: string) {
  const held = recordTails.get(key);
  if (!held) return;
  held.holders -= 1;
  if (held.holders > 0) return;
  recordTails.delete(key);
  held.stop();
}
