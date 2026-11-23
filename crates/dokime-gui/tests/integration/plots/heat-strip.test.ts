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
import { cleanup, fireEvent, render } from "@testing-library/svelte";
import { afterEach, test } from "vitest";

// Local Imports
import PlotGroupGrid from "@/components/document/PlotGroupGrid.svelte";
import { emptyLivePlotState } from "@/plots/live-frames";
import HeatStripPlot from "@/plots/catalog/HeatStripPlot.svelte";
import { documentPlotSpec } from "@/plots/grouping";
import type { DocumentViewPlot, PlotPayload } from "@/lib/types";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

afterEach(cleanup);

test("Heat Strip Fills Each Facet With Equal Width Cells and Shares Its Scale", async () => {
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
        { x: index ? 36 : 10, y: 4, instant: "1970-01-01T00:00:02Z" },
        { x: "empty", y: null, instant: null },
      ],
    })),
  };
  const descriptor = { plot_id: "heat", title: "Heat Strip", plot_type: "heat_strip", y: ["value"] } as DocumentViewPlot;
  const plot = documentPlotSpec({ plot: descriptor, payload });
  const { container, rerender } = render(HeatStripPlot, { plot });
  const { container: routed } = render(PlotGroupGrid, {
    documentPlots: [descriptor],
    liveStates: new Map([["heat", { ...emptyLivePlotState(), payload }]]),
    title: "Values",
    viewportMode: "fit",
  });
  assert.ok(routed.querySelector('[data-testid="heat-strip-plot"]'));
  const strips = container.querySelectorAll("[data-facet]");
  assert.equal(strips.length, 2);
  for (const [index, strip] of Array.from(strips).entries()) {
    const cells = strip.querySelectorAll<HTMLElement>("[data-cell]");
    assert.deepEqual(
      Array.from(cells, (cell) => cell.dataset.cell),
      ["2", index ? "36" : "10", "empty"],
    );
    assert.equal(cells[0].parentElement?.style.gridTemplateColumns, "repeat(3, minmax(0, 1fr))");
    assert.ok(cells[0].parentElement?.classList.contains("w-full"));
    for (const cell of cells) {
      assert.equal(cell.style.width, "");
      assert.equal(cell.style.minHeight, "3rem");
    }
    assert.equal(cells[2].dataset.valid, "false");
    assert.equal(cells[2].style.backgroundColor, "");
  }
  const legend = container.querySelectorAll<HTMLElement>("[data-heat-legend]");
  assert.equal(legend.length, 1);
  // A Count Ramps From Zero, Whatever the Smallest Observed Value.
  assert.equal(legend[0].dataset.min, "0");
  assert.equal(legend[0].dataset.max, "4");
  const colored = container.querySelectorAll<HTMLElement>('[data-cell="10"], [data-cell="36"]');
  assert.equal(colored[0].style.backgroundColor, colored[1].style.backgroundColor);
  assert.ok(colored[0].title.includes("1970-01-01T00:00:02Z"));
  for (const series of plot.series) {
    for (const point of series.points) {
      point.y = 4;
    }
  }
  await rerender({ plot: { ...plot } });
  assert.equal(container.querySelectorAll("[data-heat-legend]").length, 1);
  for (const series of plot.series) {
    for (const point of series.points) {
      point.y = null;
    }
  }
  await rerender({ plot: { ...plot } });
  assert.equal(container.querySelectorAll("[data-heat-legend]").length, 0);
  assert.equal(container.querySelectorAll('[data-valid="false"]').length, 6);
});

test("Heat Strip Slider Selects Values and Latest Returns to New Live Samples", async () => {
  const descriptor = { plot_id: "heat", title: "Heat Strip", plot_type: "heat_strip", y: ["value"] } as DocumentViewPlot;
  const fixture = (values: number[]) => {
    const payload: PlotPayload = {
      stream: "samples",
      x_key: "cell",
      max_points: 10,
      generated_at: "",
      series: [
        {
          key: "a",
          label: "A",
          downsampled: false,
          raw_points: values.length,
          points: values.map((y, index) => ({ x: 6, y, instant: `1970-01-01T00:00:0${index + 1}Z` })),
        },
      ],
    };
    const plot = documentPlotSpec({ plot: descriptor, payload });
    return { plot };
  };
  const { container: host, getByRole, rerender } = render(HeatStripPlot, fixture([3, 7]));
  assert.ok(host.querySelector<HTMLElement>("[data-cell]")?.title.includes(" · 7 · "));
  const slider = host.querySelector<HTMLElement>('[role="slider"]');
  assert.ok(slider);
  assert.equal(slider.getAttribute("aria-valuenow"), "1");
  assert.ok(host.querySelector('[data-slot="slider"]')?.classList.contains("w-full"));
  await fireEvent.keyDown(slider, { key: "ArrowLeft" });
  assert.ok(host.querySelector<HTMLElement>("[data-cell]")?.title.includes(" · 3 · "));
  await rerender(fixture([3, 7, 9]));
  assert.ok(host.querySelector<HTMLElement>("[data-cell]")?.title.includes(" · 3 · "));
  await rerender(fixture([]));
  assert.ok(host.querySelector<HTMLElement>("[data-cell]")?.title.includes(" · 3 · "));
  await rerender(fixture([3, 7, 9]));
  const latest = getByRole("button", { name: "Latest" });
  await fireEvent.click(latest);
  assert.ok(host.querySelector<HTMLElement>("[data-cell]")?.title.includes(" · 9 · "));
  assert.ok(host.querySelector("output")?.textContent?.includes("00:00:03"));
});
