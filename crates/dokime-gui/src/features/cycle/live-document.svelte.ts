// Dokime — SPDX-License-Identifier: Apache-2.0
import type { EvidenceScope } from "@generated/host/model";
import { createQuery } from "@tanstack/svelte-query";
import type { ConnectedSession } from "../../app/native-session/session";
import { isDocumentSurfaceCard } from "../../components/document/document-surface-policy";
import { fetchNativeHealth, fetchNativeNotes } from "../../lib/api/native-annotations";
import { fetchNativeExecutionFiles } from "../../lib/api/native-content";
import { fetchNativeRecordTail } from "../../lib/api/native-evidence";
import { fetchNativeTraceability } from "../../lib/api/native-observation";
import { orderedDocumentSections } from "../../lib/document-layout";
import type { Wire } from "../../lib/rpc/canonical";
import { documentSurface, documentView, type DocumentSurfaceOptions } from "./document-view-model";
import { holdRecordTail, selectRecords } from "./server-state";
import { workflowQueryKeys } from "./workflow-query-keys";

const GUI_EVIDENCE_GC_MS = 30_000;

export function createRecordsQuery(session: () => ConnectedSession, read: () => { enabled: boolean; scope: Wire<EvidenceScope> | null }) {
  // A New Object With the Same Scope Keeps Its Stream.
  const scopeKey = $derived(JSON.stringify(read().scope));
  const scope = $derived(JSON.parse(scopeKey) as Wire<EvidenceScope> | null);
  const enabled = $derived(read().enabled);
  const queryKey = $derived([...workflowQueryKeys.records(), session().identity.generation, scope]);
  const client = () => session().queryClient;
  const query = createQuery(() => {
    const { identity } = session();
    const held = scope;
    return {
      enabled,
      queryKey,
      queryFn: () => fetchNativeRecordTail(identity, held),
      gcTime: GUI_EVIDENCE_GC_MS,
      select: selectRecords,
      staleTime: Number.POSITIVE_INFINITY,
    };
  }, client);
  const loaded = $derived(query.isSuccess);
  $effect(() => {
    if (enabled && loaded) {
      const { identity, own } = session();
      return holdRecordTail(client(), { ...identity, own }, queryKey, scope);
    }
  });
  return query;
}

export function createLiveDocumentSurface(session: () => ConnectedSession, read: () => DocumentSurfaceOptions) {
  const options = $derived(read());
  const scope = $derived(options.scope);
  const executionId = $derived(scope.executionId ?? null);
  const kinds = $derived.by(() => {
    const sections = orderedDocumentSections(options.document);
    const cards = sections.flatMap((section) => section.cards).filter(isDocumentSurfaceCard);
    return new Set(cards.map((card) => card.kind));
  });
  const client = () => session().queryClient;
  const files = createQuery(() => {
    const { identity } = session();
    const id = executionId;
    return {
      enabled: Boolean(id && (kinds.has("artifacts") || kinds.has("media"))),
      queryFn: () => fetchNativeExecutionFiles(identity, id ?? ""),
      queryKey: [...workflowQueryKeys.executionFiles(id), identity.generation],
      staleTime: Infinity,
    };
  }, client);
  const notes = createQuery(() => {
    const { identity } = session();
    const at = scope;
    return {
      enabled: kinds.has("notes") && Boolean(at.cycleId),
      queryFn: () => fetchNativeNotes(identity, at),
      queryKey: [...workflowQueryKeys.notes(), identity.generation, at],
    };
  }, client);
  const health = createQuery(() => {
    const { identity } = session();
    const at = scope;
    return {
      enabled: kinds.has("health") && Boolean(at.cycleId),
      queryFn: () => fetchNativeHealth(identity, at),
      queryKey: [...workflowQueryKeys.health(), identity.generation, at],
    };
  }, client);
  const traceability = createQuery(() => {
    const { identity } = session();
    const at = scope;
    return {
      enabled: kinds.has("traceability"),
      queryFn: () => fetchNativeTraceability(identity, at),
      queryKey: [...workflowQueryKeys.traceability(at), identity.generation],
      gcTime: GUI_EVIDENCE_GC_MS,
    };
  }, client);
  const records = createRecordsQuery(session, () => ({
    enabled: kinds.has("records") || kinds.has("plots"),
    scope,
  }));
  const view = $derived(
    documentView(
      { ...options, document: options.document ?? null, liveSnapshot: options.snapshot },
      {
        artifacts: files.data ?? [],
        notes: notes.data ?? [],
        health: health.data ?? [],
        records: records.data ?? [],
        traceability: traceability.data ?? null,
        reportPayloads: null,
      },
    ),
  );
  const surface = $derived(documentSurface(options, view, true));
  return () => surface;
}
