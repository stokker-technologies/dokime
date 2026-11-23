// Dokime — SPDX-License-Identifier: Apache-2.0
import type { EvidenceScope } from "@generated/host/model";
import { createQuery } from "@tanstack/svelte-query";
import type { ConnectedSession } from "../../app/native-session/session";
import { fetchNativeHealth, fetchNativeNotes } from "../../lib/api/native-annotations";
import { fetchNativeExecutionFiles } from "../../lib/api/native-content";
import { fetchNativeRecordTail } from "../../lib/api/native-evidence";
import { fetchNativeTraceability } from "../../lib/api/native-observation";
import { orderedDocumentSections } from "../../lib/document-layout";
import type { Wire } from "../../lib/rpc/canonical";
import { scopeVisibleIn } from "../../lib/scope";
import {
  type DocumentSurface,
  type DocumentSurfaceOptions,
  executionRecords,
  scopedNotes,
  surfaceArtifacts,
  surfaceCards,
  surfacePlots,
} from "./document-view-model";
import { mergeParameterContexts, plotContextTelemetrySnapshot, uniqueArtifacts } from "./model-document";
import { holdRecordTail, type RecordTailDelivery, selectRecords } from "./server-state";
import { workflowQueryKeys } from "./workflow-query-keys";

const GUI_EVIDENCE_GC_MS = 30_000;

type RecordTailSource = RecordTailDelivery & { fetch?: typeof fetchNativeRecordTail };

export function createRecordsQuery(
  session: () => ConnectedSession,
  read: () => { enabled: boolean; scope: Wire<EvidenceScope> | null },
  source?: RecordTailSource,
) {
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
      queryFn: () => (source?.fetch ?? fetchNativeRecordTail)(identity, held),
      gcTime: GUI_EVIDENCE_GC_MS,
      select: selectRecords,
      staleTime: Number.POSITIVE_INFINITY,
    };
  }, client);
  const loaded = $derived(query.isSuccess);
  $effect(() => {
    if (enabled && loaded) {
      const { identity, own } = session();
      return holdRecordTail(client(), { ...identity, own }, queryKey, scope, source);
    }
  });
  return query;
}

/**
 * The Live Evidence Surface. `options` Reads Through Getters And Every Field
 * Of The Result Is Its Own Derivation, So A Snapshot, Record Or Document Change
 * Reaches Only Its Readers And Query Options Follow Only Their Own Inputs.
 */
export function createLiveDocumentSurface(
  session: () => ConnectedSession,
  options: DocumentSurfaceOptions,
  source?: RecordTailSource,
): DocumentSurface {
  // A New Object With the Same Scope Keeps Its Queries.
  const scopeKey = $derived(JSON.stringify(options.scope));
  const scope = $derived(JSON.parse(scopeKey) as Wire<EvidenceScope>);
  const executionId = $derived(scope.executionId ?? null);
  const sections = $derived(orderedDocumentSections(options.document));
  const cards = $derived(surfaceCards(sections));
  const shows = (...kinds: string[]) => cards.some((card) => kinds.includes(card.kind));
  const filesShown = $derived(shows("artifacts", "media"));
  const notesShown = $derived(shows("notes"));
  const healthShown = $derived(shows("health"));
  const traceabilityShown = $derived(shows("traceability"));
  const recordsShown = $derived(shows("records", "plots"));
  const client = () => session().queryClient;
  const files = createQuery(() => {
    const { identity } = session();
    const id = executionId;
    return {
      enabled: Boolean(id && filesShown),
      queryFn: () => fetchNativeExecutionFiles(identity, id ?? ""),
      queryKey: [...workflowQueryKeys.executionFiles(id), identity.generation],
      staleTime: Infinity,
    };
  }, client);
  const notes = createQuery(() => {
    const { identity } = session();
    const at = scope;
    return {
      enabled: notesShown && Boolean(at.cycleId),
      queryFn: () => fetchNativeNotes(identity, at),
      queryKey: [...workflowQueryKeys.notes(), identity.generation, at],
    };
  }, client);
  const health = createQuery(() => {
    const { identity } = session();
    const at = scope;
    return {
      enabled: healthShown && Boolean(at.cycleId),
      queryFn: () => fetchNativeHealth(identity, at),
      queryKey: [...workflowQueryKeys.health(), identity.generation, at],
    };
  }, client);
  const traceability = createQuery(() => {
    const { identity } = session();
    const at = scope;
    return {
      enabled: traceabilityShown,
      queryFn: () => fetchNativeTraceability(identity, at),
      queryKey: [...workflowQueryKeys.traceability(at), identity.generation],
      gcTime: GUI_EVIDENCE_GC_MS,
    };
  }, client);
  const records = createRecordsQuery(session, () => ({ enabled: recordsShown, scope }), source);

  const artifacts = $derived(uniqueArtifacts(files.data ?? []));
  const linked = $derived(surfaceArtifacts(artifacts, true));
  const visibleNotes = $derived(scopedNotes(notes.data ?? [], scope, options.noteElapsedStart));
  const visibleHealth = $derived((health.data ?? []).filter((item) => scopeVisibleIn(item.scope, scope)));
  const visibleRecords = $derived((records.data ?? []).filter((record) => scopeVisibleIn(record.scope, scope)));
  const stepRecords = $derived(executionRecords(options.execution, "step"));
  const criterionRecords = $derived(executionRecords(options.execution, "criterion"));
  const streams = $derived(options.snapshot ? Object.values(options.snapshot.telemetry) : []);
  // Each Scope's Parameters Keep Their Identity While Other Execution Fields Change.
  const caseParameters = $derived(options.parameters.case);
  const executionParameters = $derived(options.parameters.execution);
  const parameterContext = $derived(mergeParameterContexts(caseParameters, executionParameters));
  const plots = $derived(surfacePlots(cards));
  // A Snapshot That Leaves The Plotted Streams Unchanged Keeps Their Telemetry.
  let heldTelemetry: ReturnType<typeof plotContextTelemetrySnapshot> = null;
  const plotContextTelemetry = $derived.by(() => {
    const next = plotContextTelemetrySnapshot(options.snapshot, plots);
    const held = heldTelemetry;
    const same =
      next === held ||
      (next !== null &&
        held !== null &&
        Object.keys(next).length === Object.keys(held).length &&
        Object.entries(next).every(([stream, telemetry]) => held[stream] === telemetry));
    heldTelemetry = same ? held : next;
    return heldTelemetry;
  });
  const trace = $derived(traceability.data ?? null);

  return {
    options,
    view: {
      get artifacts() {
        return artifacts;
      },
      get criterionRecords() {
        return criterionRecords;
      },
      get health() {
        return visibleHealth;
      },
      get records() {
        return visibleRecords;
      },
      reportPayloads: null,
      get stepRecords() {
        return stepRecords;
      },
      get streams() {
        return streams;
      },
      get traceability() {
        return trace;
      },
      get notes() {
        return visibleNotes;
      },
      get parameterContext() {
        return parameterContext;
      },
      get plotContextTelemetry() {
        return plotContextTelemetry;
      },
      get sections() {
        return sections;
      },
    },
    shared: {
      get artifacts() {
        return linked.artifacts;
      },
      get mediaArtifacts() {
        return linked.mediaArtifacts;
      },
      get streams() {
        return streams;
      },
      get criterionRecords() {
        return criterionRecords;
      },
      get document() {
        return options.document;
      },
      get notes() {
        return visibleNotes;
      },
      get plotParameterContext() {
        return parameterContext;
      },
      get records() {
        return visibleRecords;
      },
      get health() {
        return visibleHealth;
      },
      get sections() {
        return sections;
      },
      get stepRecords() {
        return stepRecords;
      },
      get traceability() {
        return trace;
      },
      get traceabilityExportName() {
        return options.traceabilityExportName;
      },
    },
  };
}
