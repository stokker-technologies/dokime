/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import assert from "node:assert/strict";
import { test } from "node:test";
import { statsSummaryCells } from "../src/lib/statistics.ts";
import { histogramBars } from "../src/plots/histogram-bins.ts";
import {
  NO_NUMERIC_SAMPLES,
  plotPlaceholderText,
  plotUnavailableText,
  WAITING_FOR_SAMPLES,
} from "../src/plots/plot-placeholder.ts";

/* === Tests === */

test("a histogram draws one bar per backend bin with the backend's counts", () => {
  const bars = histogramBars({
    bins: [
      { lower: 0, upper: 1.5, count: 3 },
      { lower: 1.5, upper: 3, count: 0 },
      { lower: 3, upper: 4.5, count: 7 },
    ],
    count: 10,
  });

  assert.deepEqual(bars, [
    { slot: "0", min: 0, max: 1.5, count: 3 },
    { slot: "1", min: 1.5, max: 3, count: 0 },
    { slot: "2", min: 3, max: 4.5, count: 7 },
  ]);
});

test("the summary grid prints the backend's statistics and count", () => {
  const cells = statsSummaryCells({
    label: "Voltage",
    unit: "V",
    statistics: { mean: 12.5, median: 12, std: 0.25, rms: 12.6, min: 11, max: 14 },
    count: 42,
  });

  assert.deepEqual(cells, [
    ["Mean", "12.50 V"],
    ["Median", "12.00 V"],
    ["Std", "0.25 V"],
    ["RMS", "12.60 V"],
    ["Min", "11.00 V"],
    ["Max", "14.00 V"],
    ["Count", "42"],
  ]);
});

test("statistics the backend has not delivered print dashes, never browser numbers", () => {
  const cells = statsSummaryCells({ label: "Voltage", unit: "V", statistics: null, count: 0 });

  assert.deepEqual(
    cells.map(([, value]) => value),
    ["-", "-", "-", "-", "-", "-", "0"],
  );
});

test("a plot without samples waits with the shared label and detail", () => {
  assert.deepEqual(plotPlaceholderText({}), {
    label: WAITING_FOR_SAMPLES,
    detail: NO_NUMERIC_SAMPLES,
  });
  assert.equal(WAITING_FOR_SAMPLES, "Waiting For Samples");
});

test("a stated stream condition replaces the wait in the placeholder", () => {
  const text = plotPlaceholderText({
    stateLabel: "Reconnecting",
    stateDetail: "The last samples remain visible while the plot stream reopens.",
  });

  assert.deepEqual(text, {
    label: "Reconnecting",
    detail: "The last samples remain visible while the plot stream reopens.",
  });
});

test("a plot kind may word its own detail but keeps the shared label", () => {
  const text = plotPlaceholderText({}, "No categorical samples are available.");

  assert.equal(text.label, WAITING_FOR_SAMPLES);
  assert.equal(text.detail, "No categorical samples are available.");
});

test("a plot the backend could not produce says so instead of waiting", () => {
  assert.deepEqual(plotUnavailableText("Field voltage_v is not numeric."), {
    label: "Plot Unavailable",
    detail: "Field voltage_v is not numeric.",
  });
});
