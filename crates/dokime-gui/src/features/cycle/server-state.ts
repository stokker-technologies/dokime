/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import type { Wire } from "@/lib/rpc/canonical";
import type { EvidenceScope } from "@generated/host/model";
import { useQuery, useQueryClient, type Query, type QueryClient } from "@tanstack/react-query";
import { useEffect, useMemo } from "react";

/* === Local Imports === */

import { createRuntimeProjectionRefreshCoordinator, type RuntimeProjectionRefreshCoordinator } from "@/app/live/runtime-refresh";
import { useNativeSession, useNativeSessionOptional } from "@/app/native-session/context";
import { fetchNativeHealth, fetchNativeNotes } from "@/lib/api/native-annotations";
import { fetchNativeExecutionFiles } from "@/lib/api/native-content";
import { mapNativeDocument } from "@/lib/api/native-documents";
import {
  RECORD_TAIL,
  appendRecordTail,
  fetchNativeRecordTail,
  openNativeRecordStream,
  type NativeEvidenceSession,
  type RecordTail,
} from "@/lib/api/native-evidence";
import { fetchNativeTraceability } from "@/lib/api/native-observation";
import type { DocumentViewCard, DocumentViewPayload } from "@/lib/types";
import type { NativeDocumentCapture } from "../../lib/api/native-document-session";
import { mergeDocumentProgress } from "@/features/cycle/document-query-model";
import { workflowQueryKeys } from "./workflow-query-keys";

/* === Constants === */

/** How Long an Unobserved Evidence Read Stays Cached; Records and Documents Are Large. */
const GUI_EVIDENCE_GC_MS = 30_000;
const GUI_RECONNECT_MS = 1_000;

/* === Types === */

export type RootDocument = {
  completed?: NativeDocumentCapture;
  current: NativeDocumentCapture;
  mapped: DocumentViewPayload;
};

export type CardRead = { document: NativeDocumentCapture; card: DocumentViewCard };

/* === Private Functions === */

/** The connection a live read needs. */
function requireSession(session: ReturnType<typeof useNativeSessionOptional>) {
  if (!session) {
    throw new Error("Connect to a backend before reading cycle evidence.");
  }
  return session;
}

/** The identity a live read needs. */
function identityOf(session: ReturnType<typeof useNativeSessionOptional>) {
  return requireSession(session).identity;
}

/** Keys stay generation-scoped live, and constant while nothing is fetched. */
function generationOf(session: ReturnType<typeof useNativeSessionOptional>) {
  return session?.identity.generation ?? "disconnected";
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
export function useRecordsQuery(enabled: boolean, scope: Wire<EvidenceScope> | null) {
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
    enabled: enabled && session !== null,
    queryFn: () => fetchNativeRecordTail(identityOf(session), stableScope),
    queryKey,
    gcTime: GUI_EVIDENCE_GC_MS,
    select: selectRecords,
    staleTime: Number.POSITIVE_INFINITY,
  });
  const loaded = query.isSuccess;
  useEffect(() => {
    if (!enabled || !session || !loaded) {
      return;
    }
    return holdRecordTail(queryClient, { ...session.identity, own: session.own }, queryKey, stableScope);
  }, [enabled, loaded, queryClient, queryKey, session, stableScope]);
  return query;
}

export function useNotesQuery(enabled: boolean, scope: Wire<EvidenceScope>) {
  const session = useNativeSessionOptional();
  return useQuery({
    enabled: enabled && session !== null && !!scope.cycleId,
    queryFn: () => fetchNativeNotes(identityOf(session), scope),
    queryKey: [...workflowQueryKeys.notes(), generationOf(session), scope],
  });
}

export function useHealthQuery(enabled: boolean, scope: Wire<EvidenceScope>) {
  const session = useNativeSessionOptional();
  return useQuery({
    enabled: enabled && session !== null && !!scope.cycleId,
    queryFn: () => fetchNativeHealth(identityOf(session), scope),
    queryKey: [...workflowQueryKeys.health(), generationOf(session), scope],
  });
}

export function useTraceabilityQuery(enabled: boolean, scope: Wire<EvidenceScope> | null) {
  const session = useNativeSessionOptional();
  return useQuery({
    enabled: enabled && session !== null,
    queryFn: () => fetchNativeTraceability(identityOf(session), scope),
    queryKey: [...workflowQueryKeys.traceability(scope), generationOf(session)],
    gcTime: GUI_EVIDENCE_GC_MS,
  });
}

export function useExecutionFilesQuery(executionId: string | null, enabled: boolean) {
  const session = useNativeSessionOptional();
  // Runtime Changes Refresh Files Alongside Documents Through The Shared Throttle.
  return useQuery({
    enabled: Boolean(executionId) && enabled && session !== null,
    queryFn: () => fetchNativeExecutionFiles(identityOf(session), executionId ?? ""),
    queryKey: [...workflowQueryKeys.executionFiles(executionId), generationOf(session)],
    staleTime: Infinity,
  });
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
  const queries = queryClient.getQueryCache().findAll({ queryKey: [...workflowQueryKeys.all, "documents"] });
  for (const cards of [false, true]) {
    await Promise.all(
      queries
        .filter((query) => (query.queryKey.at(-3) === "cards") === cards)
        .map((query) => {
          let refresh = documentRefreshes.get(query);
          if (!refresh) {
            refresh = createRuntimeProjectionRefreshCoordinator(() =>
              queryClient.invalidateQueries({ predicate: (candidate) => candidate === query }, { cancelRefetch: false }),
            );
            documentRefreshes.set(query, refresh);
          }
          return refresh.request({ ensureTrailing: query.isActive() && query.state.fetchStatus === "fetching" });
        }),
    );
  }
}

export async function invalidateWorkflowDocuments(queryClient: QueryClient) {
  await Promise.all([
    invalidateRuntimeDocuments(queryClient),
    queryClient.invalidateQueries({ queryKey: [...workflowQueryKeys.all, "execution-files"] }),
  ]);
}

export function nativeEpoch(session: ReturnType<typeof useNativeSession>): string {
  const epoch = session.snapshot.connection.backend?.epoch;
  if (!epoch) {
    throw new Error("The connected native backend has no process epoch.");
  }
  return epoch;
}

export function nativeDocumentSession(session: ReturnType<typeof useNativeSession>) {
  return { ...session.identity, own: session.own };
}

/* === Captured Documents === */

const documentRefreshes = new WeakMap<Query, RuntimeProjectionRefreshCoordinator>();

export function selectDocument(document: RootDocument): DocumentViewPayload {
  return document.mapped;
}

/** Root query data owns native captures and their stable display projection. */
export async function capturedDocument(
  queryClient: QueryClient,
  queryKey: readonly unknown[],
  capture: (previous: NativeDocumentCapture | null, publish: (document: NativeDocumentCapture) => void) => Promise<NativeDocumentCapture>,
): Promise<RootDocument> {
  const held = queryClient.getQueryData<RootDocument>(queryKey);
  let progress = held?.mapped;
  const completed = await capture(held?.completed ?? null, (document) => {
    progress = mergeDocumentProgress(mapNativeDocument(document), progress);
    queryClient.setQueryData<RootDocument>(queryKey, { completed: held?.completed, current: document, mapped: progress });
  });
  if (held && completed === held.completed) {
    return held;
  }
  return { completed, current: completed, mapped: mapNativeDocument(completed) };
}

/* === Record Tails === */

/** One live record stream per scope, shared by every holder of that scope. */
const recordTails = new Map<string, { holders: number; stop: () => void }>();

export function selectRecords(tail: RecordTail) {
  return tail.records;
}

/**
 * Keep one scope's tail appended from its record stream while anything holds
 * it. Returns the release; the last release cancels the stream.
 */
export function holdRecordTail(
  queryClient: QueryClient,
  session: NativeEvidenceSession,
  queryKey: readonly unknown[],
  scope: Wire<EvidenceScope> | null,
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
    if (stopped || !session.isCurrent()) {
      return;
    }
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
        stream = await openNativeRecordStream(session, scope, cursor ?? tail?.cursor);
        if (stopped) {
          await stream.cancel();
          return;
        }
        for await (const page of stream) {
          if (stopped) {
            break;
          }
          for (const record of page.records) {
            pending.set(record.record_id, record);
            if (pending.size > RECORD_TAIL) {
              const oldest = pending.keys().next().value;
              if (oldest !== undefined) pending.delete(oldest);
            }
          }
          cursor = page.cursor ?? cursor;
          if (pending.size > 0) {
            frame ??= window.requestAnimationFrame(flush);
          }
        }
      } catch {
        // The stream failed: the tail is re-read below and followed again.
      }
      stream = undefined;
      if (stopped || !session.isCurrent()) {
        return;
      }
      flush();
      await queryClient.invalidateQueries({ queryKey, exact: true });
      await new Promise((resume) => window.setTimeout(resume, GUI_RECONNECT_MS));
    }
  };
  recordTails.set(key, {
    holders: 1,
    stop: () => {
      if (pending.size === 0 && cursor) {
        flush();
      }
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
  if (!held) {
    return;
  }
  held.holders -= 1;
  if (held.holders > 0) {
    return;
  }
  recordTails.delete(key);
  held.stop();
}
