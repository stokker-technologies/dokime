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
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test, vi } from "vitest";

// Local Imports
import { Metric } from "@/components/Metric";
import { StatsSummaryTable } from "@/components/StatsSummaryTable";
import { DocumentCardContentView } from "@/components/document/DocumentSurfaceView";
import { DocumentViewRenderer } from "@/components/document/DocumentViewRenderer";
import { type CompletedDocument, mapNativeDocument } from "@/lib/api/native-documents";
import { nativeValueToJson } from "@/lib/host/values";
import { create } from "@bufbuild/protobuf";
import {
  DocumentBindingSchema,
  DocumentCardKind,
  DocumentCardSchema,
  DocumentHeaderSchema,
  DocumentLayoutNodeSchema,
  DocumentSectionIdentitySchema,
  DocumentSummaryGroupSchema,
  DocumentSummaryTileSchema,
} from "@host/dokime/client/v1/documents_pb.js";
import { ValueSchema } from "@host/dokime/v1/scalar_pb.js";
import { documentSummaryTile } from "@/lib/document-presentation";
import { formatCompactDecimal } from "@/lib/format";
import { seriesLegendColumns } from "@/lib/statistics";

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
  for (let index = 0; index < plotCount; index++) place(4 + index, 3, `plot-${index}`, DocumentCardKind.PLOT);
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
  const table = renderToStaticMarkup(
    createElement(StatsSummaryTable, {
      legend: { columns: seriesLegendColumns("Mbps"), rows: [], pending: false },
    }),
  );
  const headings = table.match(/<thead\b[\s\S]*?<\/thead>/)?.[0] ?? "";
  assert.match(headings, /title="[^"<>]* \[Mbps\]"/);
  assert.doesNotMatch(headings, /whitespace-normal|overflow-wrap:anywhere|<br/);
});

test("Saved Report Browser", async () => {
  const opened: string[] = [];
  let refuseOpen = false;
  const saved: string[] = [];
  vi.resetModules();
  vi.doMock("@/lib/desktop-open", async (original) => ({ ...(await original<object>()), isTauriDesktop: () => true }));
  vi.doMock("@/lib/desktop/commands", () => ({
    commands: {
      desktopArtifactDeliver: async (_generation: string, document: { filename: string; source: unknown; action: string }) => {
        assert.deepEqual(document.source, { kind: "cycle_report", cycleId: "cycle-1" });
        assert.equal("bytesBase64" in document, false);
        if (document.action === "open") return { path: "/native/cache/report.html", origin: "current" };
        saved.push(document.filename);
        return document.filename === "cancelled.html" ? null : { path: `/home/operator/${document.filename}`, origin: "sealed" };
      },
      desktopOpenPath: async (path: string) => {
        if (refuseOpen) throw new Error("no default browser");
        opened.push(path);
        return null;
      },
    },
  }));
  try {
    const { downloadNativeReport, openNativeReport } = await import("@/lib/api/native-reports");
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
    assert.ok(delivered?.openFailure?.stack);
    assert.deepEqual(saved, ["bench.html", "cancelled.html", "night.html"]);

    // Current Report Origin Survives the Native Pull Without an HTML Payload
    assert.equal(await openNativeReport(session, source), "Current");
  } finally {
    vi.doUnmock("@/lib/desktop-open");
    vi.doUnmock("@/lib/desktop/commands");
    vi.resetModules();
  }
});

test("Every Integer Statistic Reaches the Summary Tile Without Rounding", () => {
  const base = 1152921504606847975n;
  const values = {
    min: base,
    max: base + 10000n,
    first: base,
    last: base + 10000n,
    median: base + 5000n,
    p05: base + 500n,
    p95: base + 9500n,
    p99: base + 9900n,
    mean: base + 5000n,
    sum: (base + 5000n) * 101n,
  };
  for (const [name, expected] of Object.entries(values)) {
    const wire = create(ValueSchema, {
      kind:
        expected <= 18446744073709551615n
          ? { case: "unsignedInteger", value: expected }
          : { case: "exact", value: { numerator: String(expected), denominator: "1", negative: false } },
    });
    const value = nativeValueToJson(wire);
    const tile = documentSummaryTile({
      metadata: { identifier: name, label: name },
      value,
      stat: name as never,
      format: "number",
      presentation: "value",
      state: null,
    });
    const host = document.createElement("div");
    host.innerHTML = renderToStaticMarkup(
      createElement(Metric, { metricId: tile.metric_id, label: tile.label, value: tile.formatted, exactValue: tile.exact ?? undefined }),
    );
    // The tile shows six significant digits; its title and copy keep every digit.
    const shown = host.querySelector(`[data-document-metric="${name}"] strong [data-exact-value]`);
    assert.equal(shown?.getAttribute("data-exact-value"), String(expected));
    assert.equal(shown?.getAttribute("title"), String(expected));
    assert.match(shown?.textContent ?? "", /^\d\.\d{1,5}e\+(18|20)$/);
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
  assert.equal(sum?.textContent, "2.8823e+21");
  const count = statistics?.querySelector('[data-document-metric="count"] strong [data-exact-value]');
  assert.ok(count?.textContent);
  assert.equal(count.textContent, count.getAttribute("data-exact-value"));
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
