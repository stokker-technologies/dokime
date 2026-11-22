// @vitest-environment happy-dom
/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { create } from "@bufbuild/protobuf";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test, vi } from "vitest";

// Local Imports
import { Metric } from "@/components/Metric";
import StatsSummaryTable from "@/plots/StatsSummaryTable.svelte";
import { render } from "@testing-library/svelte";
import { DocumentCardContentView } from "@/components/document/DocumentSurfaceView";
import { DocumentViewRenderer } from "@/components/document/DocumentViewRenderer";
import { type CompletedDocument, mapNativeDocument } from "@/lib/api/native-documents";
import { NativeIntegerValue } from "@/lib/api/native-values";
import { nativeValueToJson } from "@/lib/host/values";
import {
  DocumentBindingSchema,
  DocumentCardKind,
  DocumentCardSchema,
  DocumentHeaderSchema,
  DocumentLayoutNodeSchema,
  DocumentSectionIdentitySchema,
  DocumentSummaryGroupSchema,
  DocumentSummaryTileSchema,
  ValueSchema,
} from "@generated/host/model";
import { documentSummaryTile } from "@/lib/document-presentation";
import { formatCompactDecimal } from "@/lib/format";
import { seriesLegendColumns, seriesLegendRows } from "@/lib/statistics";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/**
 * A thermal-shaped captured document: an Evidence section with exact
 * statistics and records, then a Plots section of plot cards that declare no
 * column span, one card with an explicit half span and a nested section.
 */
function thermalDocument(plotCount: number): CompletedDocument {
  const base = 1152921504606847975n;
  const layout: CompletedDocument["layout"] = [];
  const cards: CompletedDocument["cards"] = new Map();
  const bindings: CompletedDocument["bindings"] = new Map();
  const sections: CompletedDocument["sections"] = new Map();
  const section = (ordinal: number, nodeId: string, label: string, parent?: number) => {
    sections.set(nodeId, create(DocumentSectionIdentitySchema, { nodeId, metadata: { identifier: nodeId, label } }));
    layout.push(
      create(DocumentLayoutNodeSchema, {
        ordinal: BigInt(ordinal),
        parentOrdinal: parent === undefined ? undefined : BigInt(parent),
        node: { kind: { case: "section", value: { nodeId, metadata: { identifier: nodeId, label } } } },
      }),
    );
  };
  const place = (ordinal: number, parent: number, id: string, kind: "summary" | DocumentCardKind, columns?: number) => {
    layout.push(
      create(DocumentLayoutNodeSchema, {
        ordinal: BigInt(ordinal),
        parentOrdinal: BigInt(parent),
        node: {
          kind: { case: "placement", value: { cardId: id, rowSpan: 1, ...(columns ? { columnSpan: { base: columns } } : {}) } },
        },
      }),
    );
    cards.set(
      id,
      create(DocumentCardSchema, {
        metadata: { identifier: id, label: id },
        body: {
          kind: kind === "summary" ? { case: "summary", value: { groupCount: 1n } } : { case: "owner", value: { kind, plotId: id } },
        },
      }),
    );
    bindings.set(
      id,
      create(DocumentBindingSchema, {
        sourceId: id,
        metadata: { identifier: id, label: id },
        result: { kind: kind === "summary" ? { case: "summary", value: { groupCount: 1n } } : { case: "pending", value: true } },
      }),
    );
  };
  section(0, "evidence", "Evidence");
  place(1, 0, "statistics", "summary");
  place(2, 0, "records", DocumentCardKind.RECORDS);
  section(3, "plots", "Plots");
  for (let index = 0; index < plotCount; index++) {
    place(4 + index, 3, `plot-${index}`, DocumentCardKind.PLOT);
  }
  place(4 + plotCount, 3, "half", DocumentCardKind.PLOT, 6);
  section(5 + plotCount, "details", "Details", 3);
  place(6 + plotCount, 5 + plotCount, "detail", DocumentCardKind.PLOT);
  const names = ["Count", "Sum", "Mean", "Median", "Min", "Max", "First", "Last", "P05", "P95", "P99"];
  const tiles = names.map((name, ordinal) =>
    create(DocumentSummaryTileSchema, {
      cardId: "statistics",
      groupOrdinal: 0n,
      ordinal: BigInt(ordinal),
      metadata: { identifier: name.toLowerCase(), label: name },
      value: {
        kind:
          name === "Sum"
            ? { case: "exact", value: { numerator: String(base * 2500n), denominator: "1", negative: false } }
            : { case: "unsignedInteger", value: name === "Count" ? 2500n : base },
      },
    }),
  );
  return {
    header: create(DocumentHeaderSchema, {
      documentId: "thermal.report",
      metadata: { identifier: "thermal.report", label: "Thermal Report" },
    }),
    bindings,
    definitions: new Map(),
    definitionTiles: new Map(),
    repeats: new Map(),
    instances: new Map(),
    sections,
    layout,
    cards,
    rows: new Map(),
    groups: new Map([
      [
        "statistics",
        [
          create(DocumentSummaryGroupSchema, {
            cardId: "statistics",
            ordinal: 0n,
            metadata: { identifier: "exact", label: "Exact Statistics" },
            tileCount: 11n,
          }),
        ],
      ],
    ]),
    tiles: new Map([["statistics", new Map([[0n, tiles]])]]),
    warnings: [],
  };
}

/* === Entry Point === */

test("Report Unit Labels", () => {
  const tile = documentSummaryTile({
    metadata: { identifier: "fixture", label: "fixture" },
    value: 12,
    unit: "Mbps",
    stat: "latest",
    format: "number",
    presentation: "value",
    state: null,
  });
  assert.match(tile.label, / \[Mbps\]$/);
  const metric = renderToStaticMarkup(createElement(Metric, { label: tile.label, value: tile.formatted }));
  assert.match(metric, /<span[^>]*class="[^"]*truncate[^"]*"[^>]*title="[^"<>]* \[Mbps\]"/);
  const { container, unmount } = render(StatsSummaryTable, {
    legend: { columns: seriesLegendColumns("Mbps"), rows: [], pending: false },
  });
  const headings = container.querySelector("thead")?.outerHTML ?? "";
  unmount();
  assert.match(headings, /title="[^"<>]* \[Mbps\]"/);
  assert.doesNotMatch(headings, /whitespace-normal|overflow-wrap:anywhere|<br/);
});

test("Saved Report Browser", async () => {
  const opened: string[] = [];
  let refuseOpen = false;
  const saved: string[] = [];
  const created: unknown[] = [];
  const published: unknown[] = [];
  vi.resetModules();
  vi.doMock("@/lib/surface-environment", async (original) => ({
    ...(await original<object>()),
    isTauriDesktop: () => true,
  }));
  vi.doMock("@/lib/host/client", () => ({
    reportsCreate: async (_session: unknown, request: unknown) => {
      created.push(request);
      return { file: { metadata: { id: "prepared-report/one" }, sha256: new Uint8Array([1]) } };
    },
    reportsPublish: async (_session: unknown, request: unknown) => {
      published.push(request);
    },
  }));
  vi.doMock("@/lib/desktop/commands", () => ({
    commands: {
      desktopArtifactDeliver: async (_generation: string, document: { filename: string; source: unknown; action: string }) => {
        assert.deepEqual(document.source, { kind: "cycle_report", cycleId: "cycle-1", name: "prepared-report/one" });
        assert.equal("bytesBase64" in document, false);
        assert.equal(document.action, "save");
        saved.push(document.filename);
        return document.filename === "cancelled.html" ? null : { path: `/home/operator/${document.filename}`, origin: "sealed" };
      },
      desktopOpenPath: async (path: string) => {
        if (refuseOpen) {
          throw new Error("no default browser");
        }
        opened.push(path);
        return null;
      },
    },
  }));
  try {
    const { downloadNativeReport } = await import("@/lib/api/native-reports");
    const session = { generation: "generation-1", isCurrent: () => true } as never;
    const source = { kind: "cycle_report", cycleId: "cycle-1" } as const;

    // The Saved Path Is the One the Browser Opens
    assert.deepEqual(await downloadNativeReport(session, source, "bench.html"), { path: "/home/operator/bench.html", openFailure: null });
    assert.deepEqual(opened, ["/home/operator/bench.html"]);

    // A Cancelled Save Opens Nothing
    assert.equal(await downloadNativeReport(session, source, "cancelled.html"), null);
    assert.deepEqual(opened, ["/home/operator/bench.html"]);

    // A Failed Open Keeps the Save Successful and Says Why
    refuseOpen = true;
    const delivered = await downloadNativeReport(session, source, "night.html");
    assert.equal(delivered?.path, "/home/operator/night.html");
    assert.equal(delivered?.openFailure?.message, "no default browser");
    assert.deepEqual(saved, ["bench.html", "cancelled.html", "night.html"]);
    assert.equal(created.length, 3);
    assert.equal(published.length, 0);
  } finally {
    vi.doUnmock("@/lib/host/client");
    vi.doUnmock("@/lib/surface-environment");
    vi.doUnmock("@/lib/desktop/commands");
    vi.resetModules();
  }
});

test("Execution Report Save Creates Once and Downloads the Published File", async () => {
  const actions: string[] = [];
  const digest = new Uint8Array([1, 2, 3]);
  vi.resetModules();
  vi.doMock("@/lib/host/client", () => ({
    reportsCreate: async () => {
      actions.push("create");
      return { file: { metadata: { id: "prepared-report/one" }, sha256: digest } };
    },
    reportsPublish: async (_session: unknown, request: { name: string; expectedRevision: Uint8Array }) => {
      actions.push("publish");
      assert.equal(request.name, "prepared-report/one");
      assert.deepEqual(request.expectedRevision, digest);
      return { report: { identifier: "published" } };
    },
  }));
  vi.doMock("@/lib/api/native-delivery", () => ({
    deliverNativeContent: async (_session: unknown, request: { source: unknown }) => {
      actions.push("download");
      assert.deepEqual(request.source, { kind: "execution_report", executionId: "execution", name: "published" });
      return { path: "/saved/report.html" };
    },
  }));
  vi.doMock("@/lib/desktop/commands", () => ({
    commands: {
      desktopOpenPath: async () => {
        actions.push("open");
      },
    },
  }));
  try {
    const { downloadNativeReport } = await import("@/lib/api/native-reports");
    const session = { generation: "generation", isCurrent: () => true } as never;
    const result = await downloadNativeReport(session, { kind: "execution_report", executionId: "execution" }, "report.html");
    assert.deepEqual(result, { path: "/saved/report.html", openFailure: null });
    assert.deepEqual(actions, ["create", "publish", "download", "open"]);
  } finally {
    vi.doUnmock("@/lib/host/client");
    vi.doUnmock("@/lib/api/native-delivery");
    vi.doUnmock("@/lib/desktop/commands");
    vi.resetModules();
  }
});

test("Desktop and Report Summary Values Share Ungrouped Decimal Formatting", () => {
  for (const [value, expected] of [
    [1234567, "1234567"],
    [1234.5, "1234.5"],
    [-1234.5678, "-1234.568"],
    [1152921504606848000, "1152921504606848000"],
  ] as const) {
    const tile = documentSummaryTile({
      metadata: { identifier: "mean", label: "Mean" },
      value,
      format: "number",
      presentation: "value",
      unit: "V",
    });
    assert.equal(tile.exact, expected);
    const host = document.createElement("div");
    host.innerHTML = renderToStaticMarkup(
      createElement(Metric, { metricId: tile.metric_id, label: tile.label, value: tile.formatted, exactValue: tile.exact ?? undefined }),
    );
    const shown = host.querySelector('[data-document-metric="mean"] strong [data-exact-value]');
    assert.equal(shown?.getAttribute("data-exact-value"), expected);
    assert.equal(tile.label, "Mean [V]");
  }
});

test("Plot Cards Without a Declared Span Stack One per Row", () => {
  const view = mapNativeDocument(thermalDocument(100));
  const host = document.createElement("div");
  host.innerHTML = renderToStaticMarkup(
    createElement(DocumentViewRenderer, {
      document: view,
      renderCard: (card) =>
        card.kind === "overview"
          ? createElement(DocumentCardContentView, { card, documentId: view.document_id, renderMode: "live" } as never)
          : createElement("p", null, card.title),
    }),
  );
  const span = (element: Element | null | undefined) =>
    ["base", "sm", "md", "lg", "xl"].map((size) => element?.getAttribute(`data-col-span-${size}`));
  const children = (sectionId: string) => [...(host.querySelector(`[data-section-id="${sectionId}"] > .document-grid`)?.children ?? [])];

  // Every Plot Card Without a Span Is a Direct Grid Card Spanning All Twelve Columns.
  const plots = children("plots");
  const plotCards = plots.filter((child) => /^plot-\d+$/.test(child.textContent ?? ""));
  assert.equal(plotCards.length, 100);
  for (const card of plotCards) {
    assert.ok(card.classList.contains("document-card"));
    assert.deepEqual(span(card), ["12", "12", "12", "12", "12"]);
  }

  // An Explicit Span Is Honoured, and a Nested Section Fills Its Row.
  const half = plots.find((child) => child.textContent === "half");
  assert.deepEqual(span(half), ["6", "6", "6", "6", "6"]);
  const nested = plots.find((child) => child.getAttribute("data-section-id") === "details");
  assert.ok(nested?.classList.contains("col-span-full"));
  assert.ok(!nested?.classList.contains("document-card"));

  // The Statistics Card Fills Its Row and Keeps All Tiles in One Wrapping Grid.
  const statistics = children("evidence").find((child) => child.querySelector("[data-document-metric]"));
  assert.ok(statistics?.classList.contains("document-card"));
  assert.deepEqual(span(statistics), ["12", "12", "12", "12", "12"]);
  const tiles = [...(statistics?.querySelectorAll("[data-document-metric]") ?? [])];
  assert.equal(tiles.length, 11);
  assert.ok(tiles.every((tile) => tile.parentElement === tiles[0].parentElement));

  // Long Exact Values Are Abbreviated in the Tile and Kept Whole in Its Title.
  const sum = statistics?.querySelector('[data-document-metric="sum"] strong [data-exact-value]');
  assert.equal(sum?.getAttribute("data-exact-value"), String(1152921504606847975n * 2500n));
  const count = statistics?.querySelector('[data-document-metric="count"] strong [data-exact-value]');
  assert.ok(count?.textContent);
  assert.equal(count.textContent, count.getAttribute("data-exact-value"));
});

test("Plot Legend Statistics Abbreviate and Keep Every Digit", () => {
  const exact = new NativeIntegerValue(1152921504606847975n, true);
  const rows = seriesLegendRows({
    hidden: new Set(),
    summaries: [{ series_id: "counter", count: 4487, min: exact, max: exact, mean: exact, median: exact }],
    traces: [{ key: "counter", label: "Counter", color: "#000000" }],
  });
  const { container: host, unmount } = render(StatsSummaryTable, { legend: { columns: seriesLegendColumns(null), rows, pending: false } });
  const values = [...host.querySelectorAll("tbody [data-exact-value]")].filter(
    (cell) => cell.getAttribute("data-exact-value") === exact.decimal,
  );
  unmount();
  assert.equal(values.length, 4);
  for (const value of values) {
    assert.equal(value.textContent, "1.15292e+18");
    assert.equal(value.getAttribute("title"), exact.decimal);
  }
});

test("Long Tile Values Abbreviate from Their Exact Digits", () => {
  for (const [exact, shown] of [
    ["1152921504606847975", "1.15292e+18"],
    ["-116445071965292150475", "-1.16445e+20"],
    ["9999995000000000000", "1e+19"],
    ["3458764513820540928/3", "1.15292e+18"],
    ["1/3", "1/3"],
    ["2/300000000000000", "6.66667e-15"],
    ["100000000000000/300000000", "333333"],
    ["12345.678901234", "12345.7"],
    ["0.000123456789012", "0.000123457"],
    ["123456789012", "123456789012"],
    ["Unavailable Reading", "Unavailable Reading"],
  ]) {
    assert.equal(formatCompactDecimal(exact), shown, exact);
  }
});

test("Integer Statistics Show Every Digit on Desktop and Report Cards", () => {
  for (const integer of [1152921504606847975n, 18446744073709551615n]) {
    for (const kind of ["integer", "unsignedInteger"] as const) {
      if (kind === "integer" && integer > 9223372036854775807n) {
        continue;
      }
      const signed = kind === "integer" ? -integer : integer;
      const value = nativeValueToJson(create(ValueSchema, { kind: { case: kind, value: signed } }));
      const tile = documentSummaryTile({
        metadata: { identifier: "integer", label: "Integer" },
        value,
        format: "number",
        presentation: "value",
      });
      assert.equal(tile.formatted, String(signed));
      const host = document.createElement("div");
      host.innerHTML = renderToStaticMarkup(
        createElement(Metric, { metricId: tile.metric_id, label: tile.label, value: tile.formatted, exactValue: tile.exact ?? undefined }),
      );
      assert.equal(host.querySelector("[data-exact-value]")?.textContent, String(signed));
    }
  }
});
