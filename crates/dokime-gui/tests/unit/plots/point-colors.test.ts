/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Plot Series Colors
 */

/* === Imports === */

import assert from "node:assert/strict";
import type uPlot from "uplot";
import { test } from "vitest";
import { seriesLegendRows } from "@/lib/statistics";
import { cursorPointOptions, seriesPaletteColor, seriesPointMarks } from "@/plots/CartesianPlot";

/* === Fixtures === */

const FAMILIES = {
  light: { background: "rgb(255, 255, 255)", palette: ["rgb(220, 38, 38)", "rgb(37, 99, 235)", "rgb(22, 163, 74)"] },
  dark: { background: "rgb(10, 10, 10)", palette: ["rgb(248, 113, 113)", "rgb(96, 165, 250)", "rgb(74, 222, 128)"] },
};
const STYLES = ["points", "line_points"] as const;

function callOption<T>(option: T | ((self: uPlot, seriesIdx: number) => T) | undefined, seriesIdx: number): T | undefined {
  return typeof option === "function" ? (option as (self: uPlot, seriesIdx: number) => T)({} as uPlot, seriesIdx) : option;
}

/* === Tests === */

test("Series Colors Follow the Palette by Series Index", () => {
  for (const { palette } of Object.values(FAMILIES)) {
    for (let index = 0; index < palette.length * 2; index += 1) {
      assert.equal(seriesPaletteColor([], palette, index), palette[index % palette.length]);
    }
    assert.equal(seriesPaletteColor(["rgb(1, 2, 3)"], palette, 0), "rgb(1, 2, 3)");
    assert.equal(seriesPaletteColor(["rgb(1, 2, 3)"], palette, 1), palette[1]);
  }
});

test("Point Marks, Hover Marks and Legend Swatches Use the Line Color", () => {
  for (const { background, palette } of Object.values(FAMILIES)) {
    const lineColors = palette.map((_, index) => seriesPaletteColor([], palette, index));
    const legend = seriesLegendRows({
      hidden: new Set(),
      summaries: [],
      traces: lineColors.map((color, index) => ({ key: `trace-${index}`, label: `Trace ${index}`, color })),
    });
    for (const style of STYLES) {
      for (const dense of [false, true]) {
        const marks = lineColors.map((color) => seriesPointMarks(style, dense, color, background));
        const cursor = cursorPointOptions(marks);
        lineColors.forEach((color, index) => {
          assert.equal(marks[index].fill, color);
          assert.equal(marks[index].stroke, background);
          assert.equal(callOption(cursor.fill, index + 1), color);
          assert.equal(legend[index].color, color);
        });
      }
    }
  }
});
