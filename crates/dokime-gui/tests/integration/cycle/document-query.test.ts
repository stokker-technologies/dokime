// @vitest-environment happy-dom
/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Independent Document Card Queries
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { QueryClient, notifyManager } from "@tanstack/svelte-query";
import { act, render } from "@testing-library/svelte";
import { test, vi } from "vitest";

// Local Imports
import type { ConnectedSession } from "@/app/native-session/session";
import type { createScopedDocumentQuery } from "@/features/cycle/document-query.svelte";
import { invalidateRuntimeDocuments } from "@/features/cycle/server-state";
import { documentCard, documentView } from "@/lib/document-presentation";
import type { DocumentViewCard, DocumentViewPayload } from "@/lib/types";
import DocumentQueryProbe from "./DocumentQueryProbe.svelte";

/* === Re-Exports === */

/* === Constants === */

const backend = vi.hoisted(() => ({ fetch: vi.fn() }));
function connection(queryClient: QueryClient): ConnectedSession {
  return {
    identity: { generation: "19", isCurrent: () => true },
    own: () => () => {},
    snapshot: { connection: { backend: { epoch: "fixture.epoch" } } },
    queryClient,
  } as unknown as ConnectedSession;
}
vi.mock("@/lib/api/native-document-session", () => ({
  fetchNativeDocument: (...args: unknown[]) => backend.fetch(...args),
}));
vi.mock("@/lib/api/native-documents", () => ({
  mapNativeDocument: (capture: { view: DocumentViewPayload }) => capture.view,
  mapNativeDocumentCard: (capture: { card: DocumentViewCard }, id: string) => {
    assert.equal(capture.card.card_id, id);
    return capture.card;
  },
  nativeDocumentCardKey: (_capture: unknown, id: string) => id,
}));

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

function card(id: string, value?: number): DocumentViewCard {
  return documentCard(
    { identifier: id, label: id },
    "table",
    value === undefined
      ? { loading: true }
      : {
          rows: [{ row_id: id, cells: { value: String(value) } }],
        },
  );
}

function structure() {
  const cards = [card("fast"), card("slow"), card("large")];
  const view = documentView("default", { identifier: "default", label: "Fixture" }, cards, {
    nodes: cards.map((item) => ({ type: "card" as const, card_id: item.card_id, layout: item.layout })),
  });
  return { view, bindings: new Map(cards.map((item) => [item.card_id, { result: { kind: { case: "pending" } } }])) };
}

// A Refresh Reaches The Fast Card While The Slow Card Keeps Its Original Request.
test("Card Queries Refresh Independently", async () => {
  notifyManager.setScheduler(queueMicrotask);
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const slow = deferred<{ card: DocumentViewCard }>();
  const slowStarted = deferred<void>();
  const fastRefreshed = deferred<void>();
  let fastReads = 0;
  let slowReads = 0;
  let cancelled = 0;
  let latest: ReturnType<typeof createScopedDocumentQuery> | undefined;
  backend.fetch.mockImplementation(async (_session, request, signal, _previous, publish) => {
    const input = request as { structureOnly?: boolean; cardIds?: string[] };
    if (input.structureOnly) {
      const capture = structure();
      (publish as ((value: unknown) => void) | undefined)?.(capture);
      return capture;
    }
    if (input.cardIds?.[0] === "slow") {
      slowReads++;
      if (slowReads > 1) {
        return { card: card("slow", 4) };
      }
      (signal as AbortSignal).addEventListener("abort", () => {
        cancelled++;
      });
      slowStarted.resolve();
      return slow.promise;
    }
    if (input.cardIds?.[0] === "large") {
      return {
        card: {
          ...card("large"),
          loading: false,
          failure: {
            id: "fixture.budget",
            title: "Fixture",
            message: "materialized_bytes",
            reason: "materialized_bytes",
          },
        },
      };
    }
    fastReads++;
    return { card: card("fast", fastReads) };
  });
  const view = render(DocumentQueryProbe, {
    session: connection(client),
    cycleId: "fixture.cycle",
    executionId: "fixture.execution",
    ready: (query) => {
      latest = query;
    },
    onDocument: (document) => {
      const fast = document?.cards.find((item) => item.card_id === "fast");
      if (fast?.rows[0]?.cells.value === "2") {
        fastRefreshed.resolve();
      }
    },
  });
  try {
    await act(async () => {
      await slowStarted.promise;
    });
    const values = () => new Map(latest?.data?.cards.map((card) => [card.card_id, card]));
    assert.equal(values().get("fast")?.rows[0]?.cells.value, "1");
    assert.equal(values().get("slow")?.loading, true);
    assert.equal(values().get("large")?.failure?.reason, "materialized_bytes");
    let refresh!: Promise<void>;
    await act(async () => {
      refresh = invalidateRuntimeDocuments(client);
      await fastRefreshed.promise;
    });
    assert.equal(values().get("fast")?.rows[0]?.cells.value, "2");
    assert.equal(values().get("slow")?.loading, true);
    assert.equal(slowReads, 1);
    assert.equal(cancelled, 0);
    await act(async () => {
      slow.resolve({ card: card("slow", 3) });
      await refresh;
    });
    assert.equal(values().get("slow")?.rows[0]?.cells.value, "4");
    assert.equal(slowReads, 2);
    assert.equal(values().get("fast")?.rows[0]?.cells.value, "2");
    assert.equal(values().get("large")?.failure?.reason, "materialized_bytes");
  } finally {
    view.unmount();
    client.clear();
    notifyManager.setScheduler((callback) => {
      setTimeout(callback, 0);
    });
    backend.fetch.mockReset();
  }
});

test("Repeat Pages Query Their Own Cards and Return to Live", async () => {
  notifyManager.setScheduler(queueMicrotask);
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const first = deferred<void>();
  const second = deferred<void>();
  const requests: { id: string; page?: string }[] = [];
  let refreshed: ReturnType<typeof deferred<void>> | undefined;
  let latest: ReturnType<typeof createScopedDocumentQuery> | undefined;
  backend.fetch.mockImplementation(async (_session, input, _signal, _previous, publish) => {
    const request = input as { structureOnly?: boolean; cardIds?: string[]; page?: string };
    const later = request.page === "fixture.second";
    const id = later ? "fixture.second.card" : "fixture.first.card";
    if (request.structureOnly) {
      const pending = card(id);
      const view = documentView("default", { identifier: "default", label: "Fixture" }, [pending], {
        nodes: [{ type: "card", card_id: id, layout: pending.layout }],
      });
      const capture = {
        view,
        page: { current: later ? "fixture.second" : "fixture.first", next: later ? undefined : "fixture.second" },
        bindings: new Map([[id, { result: { kind: { case: "pending" } } }]]),
      };
      (publish as ((value: unknown) => void) | undefined)?.(capture);
      return capture;
    }
    requests.push({ id: request.cardIds?.[0] ?? "", page: request.page });
    (later ? second : first).resolve();
    refreshed?.resolve();
    return { card: card(id, requests.length) };
  });
  const view = render(DocumentQueryProbe, {
    session: connection(client),
    cycleId: "fixture.paging",
    executionId: "fixture.execution",
    ready: (query) => {
      latest = query;
    },
  });
  try {
    await act(async () => first.promise);
    await act(async () => latest?.navigation.next());
    await act(async () => second.promise);
    assert.deepEqual(requests.slice(0, 2), [
      { id: "fixture.first.card", page: "fixture.first" },
      { id: "fixture.second.card", page: "fixture.second" },
    ]);
    assert.deepEqual(
      latest?.data?.cards.map((card) => card.card_id),
      ["fixture.second.card"],
    );
    await act(async () => latest?.navigation.previous());
    assert.deepEqual(
      latest?.data?.cards.map((card) => card.card_id),
      ["fixture.first.card"],
    );
    const before = requests.length;
    refreshed = deferred<void>();
    await act(async () => latest?.navigation.latest());
    await act(async () => refreshed?.promise);
    assert.ok(requests.length > before);
    assert.equal(requests.at(-1)?.page, "fixture.first");
    assert.equal(latest?.navigation.fixed, false);
  } finally {
    view.unmount();
    client.clear();
    notifyManager.setScheduler((callback) => setTimeout(callback, 0));
    backend.fetch.mockReset();
  }
});

/* === Entry Point === */
