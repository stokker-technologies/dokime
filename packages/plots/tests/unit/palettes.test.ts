/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Plot Series and Reference Palettes
 */

/* === Imports === */

import assert from "node:assert/strict";
import type uPlot from "uplot";
import { test } from "vitest";
import type { DocumentViewPlot } from "@dokime/plots/lib/types";
import { drawReferenceBands, paddedYRange, referenceYValues } from "@dokime/plots/CartesianPlot";
import { plotColor, referenceColor } from "@dokime/plots/colors";
import { documentPlotSpec } from "@dokime/plots/grouping";

/* === Fixtures === */

const PALETTE = ["#111111", "#222222", "#333333", "#444444", "#555555", "#666666", "#777777", "#888888"];
const DEFINITION_DEFAULT = "#60706a";

function descriptor(plotId: string, title: string, y: string[], extra: Partial<DocumentViewPlot> = {}): DocumentViewPlot {
  return { plot_id: plotId, stream: "fixture.stream", title, plot_type: "time_series", y, ...extra } as DocumentViewPlot;
}

type CanvasCall = [string, ...number[]];

function recordingPlot(range: [number, number]) {
  const calls: CanvasCall[] = [];
  const styles: { fill: string[]; stroke: string[] } = { fill: [], stroke: [] };
  const bbox = { left: 0, top: 0, width: 200, height: 100 };
  const ctx = {
    globalAlpha: 1,
    lineWidth: 1,
    set fillStyle(value: string) {
      styles.fill.push(value);
    },
    set strokeStyle(value: string) {
      styles.stroke.push(value);
    },
    save: () => calls.push(["save"]),
    restore: () => calls.push(["restore"]),
    beginPath: () => calls.push(["beginPath"]),
    stroke: () => calls.push(["stroke"]),
    fillRect: (x: number, y: number, w: number, h: number) => calls.push(["fillRect", x, y, w, h]),
    moveTo: (x: number, y: number) => calls.push(["moveTo", x, y]),
    lineTo: (x: number, y: number) => calls.push(["lineTo", x, y]),
  };
  const [low, high] = range;
  const valToPos = (value: number) => bbox.top + ((high - value) / (high - low)) * bbox.height;
  return { plot: { ctx, bbox, valToPos } as unknown as uPlot, calls, styles, valToPos };
}

/* === Tests === */

test("Series Index N Takes Palette Entry N on Every Plot", () => {
  const expected = [
    "#111111",
    "#222222",
    "#333333",
    "#444444",
    "#555555",
    "#666666",
    "#777777",
    "#888888",
    "#111111",
    "#222222",
    "#333333",
    "#444444",
    "#555555",
    "#666666",
    "#777777",
    "#888888",
  ];
  for (const [index, color] of expected.entries()) {
    assert.equal(plotColor(index, PALETTE), color);
  }
  const first = documentPlotSpec({ plot: descriptor("a", "Temperature", ["a", "b", "c"]), payload: null, plotPalette: PALETTE });
  const second = documentPlotSpec({ plot: descriptor("b", "Signal Strength", ["x", "y"]), payload: null, plotPalette: PALETTE });
  assert.deepEqual(first.colors, PALETTE.slice(0, 3));
  assert.deepEqual(second.colors, PALETTE.slice(0, 2));
});

test("References Take the Reference Palette by Index and Keep Author Colors", () => {
  const colors = ["var(--plot-reference-1)", "var(--plot-reference-2)", "var(--plot-reference-3)", "var(--plot-reference-4)"];
  for (const [index, expected] of colors.entries()) {
    assert.equal(referenceColor(DEFINITION_DEFAULT, index), expected);
    assert.equal(referenceColor(DEFINITION_DEFAULT.toUpperCase(), index), expected);
    assert.equal(referenceColor(undefined, index), expected);
    assert.equal(referenceColor(" ", index), expected);
    assert.equal(referenceColor("#abcdef", index), "#abcdef");
    assert.ok(!PALETTE.includes(referenceColor(DEFINITION_DEFAULT, index)));
  }

  const spec = documentPlotSpec({
    plot: descriptor("refs", "References", ["a"], {
      reference_bands: [
        { min: { constant: 0 }, max: { constant: 1 }, color: DEFINITION_DEFAULT },
        { min: { constant: 2 }, max: { constant: 3 }, color: "#123456" },
      ],
      reference_lines: [
        { value: { constant: 4 }, color: DEFINITION_DEFAULT },
        { value: { constant: 5 }, color: "" },
      ],
    }),
    payload: null,
    plotPalette: PALETTE,
  });
  assert.deepEqual(
    spec.referenceBands.map(({ color }) => color),
    ["var(--plot-reference-1)", "#123456"],
  );
  assert.deepEqual(
    spec.referenceLines.map(({ color }) => color),
    ["var(--plot-reference-3)", "var(--plot-reference-4)"],
  );
});

test("Data Within a Band Widens the Y Range to Both Band Edges With Padding", () => {
  const band = { min: -10, max: 10, color: "rgb(1, 2, 3)" };
  const [low, high] = paddedYRange(-2, 3, referenceYValues([], [band], "linear"), "linear");
  assert.ok(low !== null && high !== null);
  assert.ok(low < band.min && high > band.max);

  const { plot, calls, styles, valToPos } = recordingPlot([low, high]);
  drawReferenceBands(plot, [band], {} as Element, "currentColor");
  const top = valToPos(band.max);
  const bottom = valToPos(band.min);
  assert.ok(top > 0 && bottom < 100);
  assert.deepEqual(
    calls.filter(([name]) => name === "fillRect"),
    [["fillRect", 0, top, 200, bottom - top]],
  );
  assert.deepEqual(
    calls.filter(([name]) => name === "moveTo").map(([, , y]) => y),
    [Math.round(top) + 0.5, Math.round(bottom) + 0.5],
  );
  assert.deepEqual(styles.fill, [band.color]);
  assert.deepEqual(styles.stroke, [band.color]);
});

test("Unresolved Band Bounds Are Neither Drawn nor Ranged", () => {
  const band = { min: null, max: 10, color: "rgb(1, 2, 3)" };
  assert.deepEqual(referenceYValues([], [band], "linear"), [10]);
  const { plot, calls } = recordingPlot([0, 20]);
  drawReferenceBands(plot, [band], {} as Element, "currentColor");
  assert.equal(calls.filter(([name]) => name === "fillRect").length, 0);
});
