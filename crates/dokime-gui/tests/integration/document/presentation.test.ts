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
import { test } from "vitest";

// Local Imports
import { LivePlotGroupGrid } from "@/components/document/PlotGroupGrid";
import type { DocumentViewCard, DocumentViewPayload, DocumentViewPlot } from "@/lib/types";
import { DocumentViewRenderer } from "../../../src/components/document/DocumentViewRenderer";
import { ExecutionSectionCard } from "../../../src/features/cycle/ExecutionSectionCard";

/* === Re-Exports === */

/* === Constants === */

const layout = { col_span: { base: 12 }, row_span: 1, order: 0, density: "normal", rail: "main" } as const;
const cards: DocumentViewCard[] = ["Signal", "Position"].map((title) => ({
  card_id: title.toLowerCase(),
  kind: "plots",
  title,
  layout,
  region: "main",
  groups: [],
  plots: [],
  rows: [],
  show_report_outcome: false,
  config: {},
}));
const documentView: DocumentViewPayload = {
  document_id: "default",
  metadata: { identifier: "default", label: "Document" },
  title: "Document",
  desktop_sidebar_min_px: 0,
  desktop_sidebar_max_px: 0,
  cards,
  layout: {
    nodes: cards.map((card, index) => ({
      type: "section",
      node_id: `section-${index}`,
      metadata: { identifier: `section-${index}`, label: index ? "Motion" : "Host" },
      children: [{ type: "card", card_id: card.card_id, layout }],
    })),
  },
};

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Document Sections Share One Heading Scale Across Every Chrome", () => {
  const host = document.createElement("div");
  for (const sectionChrome of ["plain", "grouped", "card"] as const) {
    host.innerHTML = renderToStaticMarkup(
      createElement(DocumentViewRenderer, {
        document: documentView,
        sectionChrome,
        renderCard: (card) =>
          createElement(LivePlotGroupGrid, {
            documentPlots: [
              {
                plot_id: card.card_id,
                title: `${card.title} Plot`,
                plot_type: "time_series",
                stream: "readings",
                y: ["value"],
              } as DocumentViewPlot,
            ],
            title: card.title,
            showGroupHeaders: false,
            viewportMode: "fit",
          }),
      }),
    );
    const headings = host.querySelectorAll(
      "[data-section-id] > header .document-section-title, [data-section-id] > [data-slot=card-header] .document-section-title",
    );
    assert.equal(headings.length, 2);
    assert.equal(headings[0].tagName, "H4");
    assert.equal(host.querySelectorAll(".plot-card strong").length, 2);
    assert.equal(host.querySelectorAll(".plot-card .document-section-title").length, 0);
  }
});

test("Execution Sections Use the Document Heading Scale", () => {
  const host = document.createElement("div");
  host.innerHTML = renderToStaticMarkup(createElement(ExecutionSectionCard, { title: "Report" }));
  assert.ok(host.querySelector("section[aria-labelledby] > header > h2.document-section-title"));
});
