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
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { fromBinary } from "../../../dokime-gui/node_modules/@bufbuild/protobuf/dist/esm/index.js";
import { Row3Schema as WatchFrameSchema } from "@generated/dokime/v1/PlotsMessages/Watch/messages_pb";
import { mapNativePlotStreamFrame } from "@/lib/api/native-plots";
import { applyPlotMessages, emptyLivePlotState } from "@/plots/live-frames";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export function prepare(snapshotPath: string, deltaPath: string) {
  const snapshotBytes = Array.from({ length: 30 }, (_, index) => readFileSync(join(snapshotPath, `${index}.bin`)));
  const deltaBytes = Array.from({ length: 30 }, (_, index) => readFileSync(join(deltaPath, `${index}.bin`)));
  const snapshot = snapshotBytes.flatMap((bytes) => mapNativePlotStreamFrame(fromBinary(WatchFrameSchema, bytes)).updates);
  const delta = deltaBytes.flatMap((bytes) => mapNativePlotStreamFrame(fromBinary(WatchFrameSchema, bytes)).updates);
  assert.equal(snapshot.length, 30);
  assert.equal(delta.length, snapshot.length);
  const retained = snapshot.map((message) => applyPlotMessages(emptyLivePlotState(), [message]));
  const updates = delta.map((message) => [message]);
  const expected = retained.map((state, index) => applyPlotMessages(state, updates[index]));
  assert.ok(expected.every((state, index) => state.payload !== retained[index].payload));
  for (const [index, message] of delta.entries()) {
    assert.ok(message.kind !== "pending");
    const values = message.payload.cartesian?.series[0].y ?? [];
    const minimum = Math.min(...values.filter((value): value is number => value != null));
    assert.ok(minimum <= -1000);
    assert.ok(!(retained[index].payload?.cartesian?.series[0].y ?? []).includes(minimum));
    assert.ok((expected[index].payload?.cartesian?.series[0].y ?? []).includes(minimum));
  }

  let result: unknown;
  return (operation: string, iterations: number): string => {
    const start = process.hrtime.bigint();
    if (operation === "decode") {
      for (let index = 0; index < iterations; index++) {
        result = snapshotBytes.map((bytes) => mapNativePlotStreamFrame(fromBinary(WatchFrameSchema, bytes)));
      }
    } else {
      assert.equal(operation, "apply_delta");
      for (let index = 0; index < iterations; index++) {
        // Each Iteration Replays The Same Late Bucket Against The Original Retained Frames.
        result = retained.map((state, plot) => applyPlotMessages(state, updates[plot]));
      }
    }
    const elapsed = process.hrtime.bigint() - start;
    assert.ok(result);
    return String(elapsed);
  };
}

/* === Private Functions === */

/* === Entry Point === */
