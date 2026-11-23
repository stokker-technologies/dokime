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
 * Live Updates Leave The Document's Queries Untouched
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { notifyManager, QueriesObserver, QueryClient, QueryObserver } from "@tanstack/svelte-query";
import { render } from "@testing-library/svelte";
import { tick } from "svelte";
import { test, vi } from "vitest";

// Local Imports
import type { ConnectedSession } from "@/app/native-session/session";
import { createPlotWindow } from "@/features/cycle/plot-window-state";
import { documentCard, documentView } from "@/lib/document-presentation";
import type { DocumentViewPayload, HubLiveState } from "@/lib/types";
import LiveSurfaceProbe from "./LiveSurfaceProbe.svelte";

/* === Constants === */

const backend = vi.hoisted(() => ({ fetch: vi.fn() }));
vi.mock("@/lib/api/native-document-session", () => ({ fetchNativeDocument: (...args: unknown[]) => backend.fetch(...args) }));
vi.mock("@/lib/api/native-documents", () => ({
  mapNativeDocument: (capture: { view: DocumentViewPayload }) => capture.view,
  mapNativeDocumentCard: () => undefined,
  nativeDocumentCardKey: (_capture: unknown, id: string) => id,
}));
vi.mock("@/lib/api/native-annotations", () => ({ fetchNativeNotes: async () => [], fetchNativeHealth: async () => [] }));
vi.mock("@/lib/api/native-content", () => ({ fetchNativeExecutionFiles: async () => [] }));
vi.mock("@/lib/api/native-observation", () => ({ fetchNativeTraceability: async () => null }));

/* === Entry Point === */

// A Refresh Status, A Live Snapshot, An Equal Scope Or An Unchanged Structure Never Re-Runs A Query's Options.
test("Live Updates Keep Every Query's Options", async () => {
  notifyManager.setScheduler(queueMicrotask);
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const session = {
    identity: { generation: "1", isCurrent: () => true },
    own: () => () => {},
    snapshot: { connection: { backend: { epoch: "epoch" } } },
    queryClient,
  } as unknown as ConnectedSession;
  const cards = ["notes", "health", "traceability", "artifacts", "records"].map((kind) =>
    documentCard({ identifier: kind, label: kind }, kind as "notes", {}),
  );
  const view = documentView("default", { identifier: "default", label: "Fixture" }, cards, {
    nodes: cards.map((card) => ({ type: "card" as const, card_id: card.card_id, layout: card.layout })),
  });
  backend.fetch.mockImplementation(async (_session, _request, _signal, _previous, publish) => {
    const capture = { view: structuredClone(view), bindings: new Map() };
    publish?.(capture);
    return capture;
  });
  const setOptions = vi.spyOn(QueryObserver.prototype, "setOptions");
  const setQueries = vi.spyOn(QueriesObserver.prototype, "setQueries");
  const plotWindow = createPlotWindow();
  const probe = render(LiveSurfaceProbe, { session, plotWindow });
  await vi.waitFor(() =>
    assert.ok(
      queryClient
        .getQueryCache()
        .getAll()
        .every((query) => query.state.status === "success"),
    ),
  );
  await tick();
  const queries = queryClient.getQueryCache().getAll().length;
  assert.equal(queries, 6, "the document and its five evidence queries");
  setOptions.mockClear();
  setQueries.mockClear();
  const stream = { name: "pulse" } as unknown as HubLiveState["telemetry"][string];
  const sections = probe.component.sections();
  const changes = {
    refresh: async () => plotWindow.getState().refresh.onChange({ refreshing: false, updatedAt: 1 }),
    snapshot: async () => probe.component.setSnapshot({ telemetry: { pulse: stream } } as unknown as HubLiveState),
    scope: async () => probe.component.setScope({ cycleId: "cycle", executionId: "execution" }),
    structure: () => queryClient.refetchQueries({ queryKey: ["workflow", "documents"] }),
  };
  for (const [name, change] of Object.entries(changes)) {
    await change();
    await tick();
    assert.deepEqual(
      setOptions.mock.calls.map(([options]) => options.queryKey),
      [],
      `${name} keeps every query's options`,
    );
    assert.equal(setQueries.mock.calls.length, 0, `${name} keeps the card queries`);
  }
  assert.deepEqual(probe.component.streams(), [stream]);
  assert.equal(probe.component.sections(), sections, "an unchanged structure keeps its sections");
  assert.ok(backend.fetch.mock.calls.length > 1);
  assert.equal(queryClient.getQueryCache().getAll().length, queries);
  probe.unmount();
});
