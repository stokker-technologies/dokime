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
import {
  HISTOGRAM_COUNT_LABEL,
  histogramAxes,
  histogramAxisTitle,
} from "../src/plots/histogram-axes.ts";

/* === Tests === */

test("A histogram draws its declared value on x and the host's count on y", () => {
  const format = { kind: "fixed", decimals: 1 };
  const axes = histogramAxes(
    {
      title: "Voltage Distribution",
      x: "voltage_v",
      unit: "V",
      histogram: { bins: 12 },
      x_format: format,
    },
    { label: "Voltage", unit: "mV", format: null },
  );

  assert.deepEqual(axes, {
    x: { label: "Voltage", unit: "V", format },
    y: { label: HISTOGRAM_COUNT_LABEL },
  });
  assert.equal(histogramAxisTitle(axes.x), "Voltage (V)");
  assert.equal(histogramAxisTitle(axes.y), "Count");
});

test("A histogram without a column spec titles x by its declared field", () => {
  const axes = histogramAxes({
    title: "Time Until Ready",
    x: "time_until_ready_s",
    unit: null,
    histogram: { bins: 12, unit: "s" },
    x_format: null,
  });

  assert.equal(histogramAxisTitle(axes.x), "time_until_ready_s (s)");
  assert.equal(axes.x.format, null);
});

test("A histogram never titles x by the row timestamp", () => {
  const axes = histogramAxes(
    { title: "Current Distribution", x: "timestamp", unit: null, histogram: null, x_format: null },
    { label: null, unit: "A", format: { kind: "engineering", significant_digits: 3 } },
  );

  assert.deepEqual(axes.x, {
    label: "Current Distribution",
    unit: "A",
    format: { kind: "engineering", significant_digits: 3 },
  });
});
