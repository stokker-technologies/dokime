// @vitest-environment happy-dom
/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Plot Marker Colors
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test } from "vitest";

// Local Imports
import { LivePlotGroupGrid } from "@dokime/plots/components/document/PlotGroupGrid";
import { emptyLivePlotState } from "@dokime/plots/live-frames";
import { HeatStripPlot } from "../../src/catalog/HeatStripPlot";
import { documentPlotSpec } from "@dokime/plots/grouping";
import type { DocumentViewPlot, PlotPayload } from "@dokime/plots/lib/types";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Heat Strip Renders Aligned Facets Equal Squares One Scale and Empty Cells", () => {
  const payload: PlotPayload = {
    stream: "samples",
    x_key: "cell",
    max_points: 6,
    generated_at: "",
    series: ["a", "b"].map((label, index) => ({
      key: label,
      label,
      raw_points: 2,
      downsampled: false,
      points: [
        { x: 2, y: index ? 4 : 1, instant: "1970-01-01T00:00:01Z" },
        { x: 10, y: 4, instant: "1970-01-01T00:00:02Z" },
        { x: "empty", y: null, instant: null },
      ],
    })),
  };
  const descriptor = { plot_id: "heat", title: "Heat Strip", plot_type: "heat_strip", y: ["value"] } as DocumentViewPlot;
  const plot = documentPlotSpec({ plot: descriptor, payload });
  const container = document.createElement("div");
  container.innerHTML = renderToStaticMarkup(createElement(HeatStripPlot, { plot }));
  const routed = renderToStaticMarkup(
    createElement(LivePlotGroupGrid, {
      documentPlots: [descriptor],
      liveStates: { heat: { ...emptyLivePlotState(), payload } },
      title: "Values",
      viewportMode: "fit",
    }),
  );
  assert.ok(routed.includes('data-testid="heat-strip-plot"'));
  const strips = container.querySelectorAll("[data-facet]");
  assert.equal(strips.length, 2);
  for (const strip of strips) {
    const cells = strip.querySelectorAll<HTMLElement>("[data-cell]");
    assert.deepEqual(
      Array.from(cells, (cell) => cell.dataset.cell),
      ["2", "10", "empty"],
    );
    for (const cell of cells) assert.equal(cell.style.width, cell.style.height);
    assert.equal(cells[2].dataset.valid, "false");
    assert.equal(cells[2].style.backgroundColor, "");
  }
  const legend = container.querySelectorAll<HTMLElement>("[data-heat-legend]");
  assert.equal(legend.length, 1);
  assert.equal(legend[0].dataset.min, "1");
  assert.equal(legend[0].dataset.max, "4");
  const colored = container.querySelectorAll<HTMLElement>('[data-cell="10"]');
  assert.equal(colored[0].style.backgroundColor, colored[1].style.backgroundColor);
  assert.ok(colored[0].title.includes("1970-01-01T00:00:02Z"));
  for (const series of plot.series) for (const point of series.points) point.y = 4;
  container.innerHTML = renderToStaticMarkup(createElement(HeatStripPlot, { plot }));
  assert.equal(container.querySelectorAll("[data-heat-legend]").length, 1);
  for (const series of plot.series) for (const point of series.points) point.y = null;
  container.innerHTML = renderToStaticMarkup(createElement(HeatStripPlot, { plot }));
  assert.equal(container.querySelectorAll("[data-heat-legend]").length, 0);
  assert.equal(container.querySelectorAll('[data-valid="false"]').length, 6);
});
