// Dokime — SPDX-License-Identifier: Apache-2.0
import { createQueries, createQuery } from "@tanstack/svelte-query";
import { untrack } from "svelte";
import type { ConnectedSession } from "../../app/native-session/session";
import { fetchNativeDocument, type NativeDocumentRequest } from "../../lib/api/native-document-session";
import { mapNativeDocumentCard, nativeDocumentCardKey } from "../../lib/api/native-documents";
import { failureOf } from "../../lib/notification-details";
import { documentRetry, documentRetryDelay } from "./document-query-model";
import { type CardRead, capturedDocument, nativeDocumentSession, nativeEpoch, type RootDocument, selectDocument } from "./server-state";
import { workflowQueryKeys } from "./workflow-query-keys";

type Input = { cycleId: string | null; executionId: string | null; enabled: boolean };
type Navigation = { owner: string; token?: string; previous: string[]; revision: number };

const GUI_EVIDENCE_GC_MS = 30_000;

export function createScopedDocumentQuery(session: () => ConnectedSession, input: () => Input) {
  const connected = $derived(session());
  const { cycleId, executionId, enabled: requested } = $derived(input());
  const client = () => connected.queryClient;
  const cardKey = $derived([...workflowQueryKeys.executionDocument({ cycleId, executionId }), connected.identity.generation]);
  const owner = $derived(JSON.stringify(cardKey));
  let heldNavigation = $state.raw<Navigation>();
  const navigation = $derived<Navigation>(heldNavigation?.owner === owner ? heldNavigation : { owner, previous: [], revision: 0 });
  const token = $derived(navigation.token);
  const captureKey = $derived([...cardKey, "pages", token ?? ""]);
  const connection = $derived(nativeDocumentSession(connected));
  const request = $derived<NativeDocumentRequest>({
    cycleId: cycleId ?? "",
    executionId: executionId ?? "",
    documentId: "default",
    epoch: nativeEpoch(connected),
  });
  const enabled = $derived(requested && Boolean(cycleId));
  const query = createQuery(() => {
    const key = captureKey;
    const source = connection;
    const document = { ...request, structureOnly: true, page: token };
    const queryClient = client();
    return {
      enabled,
      queryKey: key,
      queryFn: ({ signal }: { signal: AbortSignal }) =>
        capturedDocument(queryClient, key, (previous, publish) => fetchNativeDocument(source, document, signal, previous, publish)),
      select: selectDocument,
      structuralSharing: false,
      gcTime: GUI_EVIDENCE_GC_MS,
      retry: documentRetry,
      retryDelay: documentRetryDelay,
    };
  }, client);
  const root = $derived(query.data ? client().getQueryData<RootDocument>(captureKey) : undefined);
  const candidates = $derived.by(() =>
    (query.data?.cards ?? []).flatMap((card) => {
      const id = card.card_id;
      const source = root?.current.bindings.has(id) ? root.current : root?.completed;
      if (source?.bindings.get(id)?.result?.kind.case !== "pending") {
        return [];
      }
      const key = [...cardKey, "view", navigation.revision, "cards", id, nativeDocumentCardKey(source, id)];
      return [{ id, key, page: source.page?.current }];
    }),
  );
  const candidateIdentity = $derived(JSON.stringify(candidates));
  const cards = createQueries(() => {
    // Structure Refreshes Can Republish the Same Card Cuts While a Card Is Still Reading.
    void candidateIdentity;
    const pendingCards = untrack(() => candidates);
    const source = connection;
    const rootKey = captureKey;
    const rootRequest = request;
    const queryClient = client();
    return {
      queries: pendingCards.map(({ id, key, page }) => ({
        queryKey: key,
        enabled,
        gcTime: GUI_EVIDENCE_GC_MS,
        retry: documentRetry,
        retryDelay: documentRetryDelay,
        queryFn: async ({ signal }: { signal: AbortSignal }): Promise<CardRead> => {
          const latest = queryClient.getQueryData<RootDocument>(rootKey);
          const capture = latest?.current.bindings.has(id) ? latest.current : latest?.completed;
          const previous = queryClient.getQueryData<CardRead>(key);
          const document = await fetchNativeDocument(
            source,
            { ...rootRequest, cardIds: [id], at: capture?.resolvedAt, page },
            signal,
            previous?.document,
          );
          if (previous && document === previous.document) {
            return previous;
          }
          return { document, card: mapNativeDocumentCard(document, id) };
        },
      })),
    };
  }, client);
  const document = $derived.by(() => {
    const mapped = query.data;
    if (!mapped) {
      return undefined;
    }
    const results = new Map(candidates.map(({ id }, index) => [id, cards[index]]));
    return {
      ...mapped,
      cards: mapped.cards.map((card) => {
        const result = results.get(card.card_id);
        const value = result?.data?.card ?? card;
        if (result?.error) {
          return { ...value, loading: false, failure: failureOf(result.error, card.title) };
        }
        return value;
      }),
    };
  });

  async function retryCard(id?: string) {
    const index = candidates.findIndex((card) => card.id === id);
    if (index >= 0) {
      await cards[index]?.refetch();
      return;
    }
    await query.refetch();
    await Promise.all(cards.filter((card) => card.error || card.data?.card.failure).map((card) => card.refetch()));
  }

  return {
    get data() {
      return document;
    },
    get error() {
      return query.error;
    },
    get failureReason() {
      return query.failureReason;
    },
    get isPending() {
      return query.isPending;
    },
    get navigation() {
      const page = root?.current.page;
      return {
        hasPrevious: navigation.previous.length > 0,
        hasNext: Boolean(page?.next),
        fixed: Boolean(navigation.token),
        busy: query.isFetching,
        previous: () => {
          const previous = [...navigation.previous];
          const token = previous.pop();
          heldNavigation = { ...navigation, token, previous };
        },
        next: () => {
          if (page?.next) {
            heldNavigation = { ...navigation, token: page.next, previous: [...navigation.previous, page.current] };
          }
        },
        latest: () => {
          heldNavigation = { owner, previous: [], revision: navigation.revision + 1 };
        },
      };
    },
    retryCard,
  };
}
