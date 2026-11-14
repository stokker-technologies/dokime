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
import { nativeValueToJson } from "@/lib/host/values";
import { create } from "@bufbuild/protobuf";
import { ValueSchema } from "@host/dokime/v1/scalar_pb.js";
import { documentSummaryTile } from "@/lib/document-presentation";
import { seriesLegendColumns } from "@/lib/statistics";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

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
    host.innerHTML = renderToStaticMarkup(createElement(Metric, { metricId: tile.metric_id, label: tile.label, value: tile.formatted }));
    assert.equal(host.querySelector(`[data-document-metric="${name}"] strong`)?.textContent, String(expected));
  }
});
