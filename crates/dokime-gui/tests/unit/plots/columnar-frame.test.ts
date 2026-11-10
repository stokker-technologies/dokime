/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Columnar Plot Decoding
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { test } from "vitest";
import { create } from "@bufbuild/protobuf";

// Local Imports
import { Row3Schema as WatchFrameSchema } from "@generated/dokime/v1/PlotsMessages/Watch/messages_pb";
import { mapNativePlotStreamFrame } from "@/lib/api/native-plots";
import { jsonToNativeValue } from "@/lib/api/native-values";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Aligned Columns Decode With One Shared Time Array and No Point Copies", () => {
  const wire = create(WatchFrameSchema, {
    updates: [
      {
        kind: { HarmosVariant: { case: "Snapshot", value: {} } },
        envelope: {
          plotId: "signal",
          epoch: "epoch",
          selection: "signal",
          diagnostics: {},
          payload: {
            stream: "signal",
            xKey: "timestamp",
            maxPoints: 100n,
            series: [
              { key: "a", label: "A" },
              { key: "b", label: "B" },
            ],
            frame: {
              mode: { HarmosVariant: { case: "Aligned", value: {} } },
              time: [1, 2, 3],
              series: [{ y: [{ value: 2 }, {}, { value: 4 }] }, { y: [{}, { value: 8 }, {}] }],
              pointCount: 3n,
              signature: "frame",
            },
          },
        },
      },
    ],
  });
  const decoded = mapNativePlotStreamFrame(wire).updates[0];
  assert.ok(decoded.kind !== "pending");
  const frame = decoded.payload.cartesian;
  assert.deepEqual(frame?.series[0].x, [1, 2, 3]);
  assert.equal(frame?.series[0].x, frame?.series[1].x);
  assert.deepEqual(frame?.series[0].y, [2, null, 4]);
  assert.ok(decoded.payload.series.every((series) => series.points.length === 0));
});

test("Independent Columns Preserve Categorical Axes and Z Values", () => {
  const wire = create(WatchFrameSchema, {
    updates: [
      {
        kind: { HarmosVariant: { case: "Snapshot", value: {} } },
        envelope: {
          plotId: "categories",
          epoch: "epoch",
          selection: "categories",
          diagnostics: {},
          payload: {
            stream: "categories",
            xKey: "category",
            maxPoints: 100n,
            series: [{ key: "value", label: "Value" }],
            frame: { series: [{ x: [jsonToNativeValue("Category")], y: [{ value: 2 }], z: [{ value: 7 }] }] },
          },
        },
      },
    ],
  });
  const decoded = mapNativePlotStreamFrame(wire).updates[0];
  assert.ok(decoded.kind !== "pending");
  assert.deepEqual(decoded.payload.series[0].points, [{ x: "Category", y: 2, z: 7 }]);
  assert.equal(decoded.payload.cartesian, null);
});
