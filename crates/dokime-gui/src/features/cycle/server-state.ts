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
import { type Query, type QueryClient, replaceEqualDeep } from "@tanstack/svelte-query";

/* === Local Imports === */

import { createRuntimeProjectionRefreshCoordinator, type RuntimeProjectionRefreshCoordinator } from "@/app/live/runtime-refresh";
import type { ConnectedSession } from "@/app/native-session/session";
import { mapNativeDocument } from "@/lib/api/native-documents";
import {
  RECORD_TAIL,
  appendRecordTail,
  openNativeRecordStream,
  type NativeEvidenceSession,
  type RecordTail,
} from "@/lib/api/native-evidence";
import type { DocumentViewCard, DocumentViewPayload } from "@/lib/types";
import type { NativeDocumentCapture } from "../../lib/api/native-document-session";
import { isHostReopen } from "../../lib/rpc/errors";
import { mergeDocumentProgress } from "@/features/cycle/document-query-model";
import { workflowQueryKeys } from "./workflow-query-keys";

/* === Constants === */

const GUI_RECONNECT_MS = 1_000;

/* === Types === */

export type RootDocument = {
  completed?: NativeDocumentCapture;
  current: NativeDocumentCapture;
  mapped: DocumentViewPayload;
};

export type CardRead = { document: NativeDocumentCapture; card: DocumentViewCard };

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

export function nativeEpoch(session: ConnectedSession): string {
  const epoch = session.snapshot.connection.backend?.epoch;
  if (!epoch) {
    throw new Error("The connected native backend has no process epoch.");
  }
  return epoch;
}

export function nativeDocumentSession(session: ConnectedSession) {
  return { ...session.identity, own: session.own };
}

/* === Captured Documents === */

const documentRefreshes = new WeakMap<Query, RuntimeProjectionRefreshCoordinator>();

export function selectDocument(document: RootDocument): DocumentViewPayload {
  return document.mapped;
}

/**
 * Root query data owns native captures and their stable display projection:
 * a refresh keeps every unchanged card and plot of the projection as it was.
 */
export async function capturedDocument(
  queryClient: QueryClient,
  queryKey: readonly unknown[],
  capture: (previous: NativeDocumentCapture | null, publish: (document: NativeDocumentCapture) => void) => Promise<NativeDocumentCapture>,
): Promise<RootDocument> {
  const held = queryClient.getQueryData<RootDocument>(queryKey);
  let progress = held?.mapped;
  const completed = await capture(held?.completed ?? null, (document) => {
    progress = replaceEqualDeep(progress, mergeDocumentProgress(mapNativeDocument(document), progress));
    queryClient.setQueryData<RootDocument>(queryKey, { completed: held?.completed, current: document, mapped: progress });
  });
  if (held && completed === held.completed) {
    return held;
  }
  return { completed, current: completed, mapped: replaceEqualDeep(progress, mapNativeDocument(completed)) };
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
export type RecordTailDelivery = {
  openStream?: typeof openNativeRecordStream;
  schedule?: (publish: () => void) => number | null;
};

export function holdRecordTail(
  queryClient: QueryClient,
  session: NativeEvidenceSession,
  queryKey: readonly unknown[],
  scope: Wire<EvidenceScope> | null,
  { openStream = openNativeRecordStream, schedule = (publish) => window.requestAnimationFrame(publish) }: RecordTailDelivery = {},
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
      let reopen = false;
      try {
        stream = await openStream(session, scope, cursor ?? tail?.cursor);
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
            frame ??= schedule(flush);
          }
        }
      } catch (error) {
        // A host deadline resumes from the cursor at once; any other end re-reads the tail below.
        reopen = isHostReopen(error);
      }
      stream = undefined;
      if (stopped || !session.isCurrent()) {
        return;
      }
      if (reopen) {
        continue;
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
