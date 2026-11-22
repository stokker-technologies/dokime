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
import { create } from "@bufbuild/protobuf";
import { test } from "vitest";

// Local Imports
import { mapNativePlotPayload, plotType } from "@dokime/plots/lib/api/native-plot-projection";
import { PlotKind, PlotPayloadSchema, PlotCartesianMode } from "@generated/host/model";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Heat Strip Projection Preserves Typed Cell Labels Empty Values and Contribution Times", () => {
  const native = create(PlotPayloadSchema, {
    series: [
      { key: "a", label: "a" },
      { key: "b", label: "b" },
    ],
    frame: {
      mode: PlotCartesianMode.FACETED,
      series: [
        {
          x: [{ kind: { case: "unsignedInteger", value: 2n } }, { kind: { case: "unsignedInteger", value: 9007199254740993n } }],
          y: [{ value: 4 }, {}],
          instants: ["1970-01-01T00:00:01Z", ""],
          totals: [],
        },
        {
          x: [{ kind: { case: "text", value: "10" } }, { kind: { case: "text", value: "2" } }],
          y: [{}, { value: 8 }],
          instants: ["", "1970-01-01T00:00:02Z"],
        },
      ],
    },
  });
  const payload = mapNativePlotPayload(native, "heat_strip");
  assert.equal(plotType("heat_strip"), PlotKind.HEAT_STRIP);
  assert.equal(payload.cartesian, null);
  assert.deepEqual(
    payload.series[0].points.map((point) => String(point.x)),
    ["2", "9007199254740993"],
  );
  assert.deepEqual(
    payload.series[1].points.map((point) => point.x),
    ["10", "2"],
  );
  assert.deepEqual(
    payload.series[0].points.map((point) => point.y),
    [4, null],
  );
  assert.equal(payload.series[1].points[1].instant, "1970-01-01T00:00:02Z");
});
