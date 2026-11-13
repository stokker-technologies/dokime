/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Document Plot Frame Commits
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { test } from "vitest";
import { create } from "@bufbuild/protobuf";

// Local Imports
import { PlotStreamFrameSchema as WatchFrameSchema, PlotStreamKind } from "@host/dokime/v1/client_types_pb.js";
import { mapNativePlotStreamFrame } from "@/lib/api/native-plots";
import type { PlotPayloadStreamMessage } from "@/lib/types";
import { applyDocumentPlotMessages, applyPendingNotice, emptyLivePlotState } from "@/plots/live-frames";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function frame(plot_id: string, revision: number, kind: PlotPayloadStreamMessage["kind"] = "snapshot"): PlotPayloadStreamMessage {
  return {
    plot_id,
    revision,
    kind,
    epoch: "epoch",
    selection: plot_id,
    warnings: [],
    payload: { stream: "telemetry", x_key: "timestamp", max_points: 100, generated_at: "2026-11-09T00:00:00Z", series: [] },
  };
}

/* === Entry Point === */

test("Document Frames Commit Together Without Mutating the Visible State", () => {
  const ids = Array.from({ length: 8 }, (_, index) => `plot.${index}`);
  const original = Object.fromEntries(ids.map((id) => [id, emptyLivePlotState()]));
  const opened = applyDocumentPlotMessages(
    original,
    ids.map((id) => frame(id, 10)),
  );
  const queued = ids.map((id) => frame(id, 11, "replace"));
  assert.equal(applyPendingNotice(opened, ids[0], 100), opened);
  assert.ok(Object.values(opened).every((plot) => plot.revision === 10));
  const committed = applyDocumentPlotMessages(opened, queued);
  assert.ok(Object.values(committed).every((plot) => plot.revision === 11 && plot.status === "live"));
  assert.ok(Object.values(opened).every((plot) => plot.revision === 10));
  assert.ok(Object.values(original).every((plot) => plot.payload === null));
});

test("Partial Dirty Sets Preserve Idle Plots and Isolate Errors", () => {
  const original = { first: emptyLivePlotState(), second: emptyLivePlotState(), idle: emptyLivePlotState() };
  const opened = applyDocumentPlotMessages(original, [frame("first", 1), frame("second", 1), frame("idle", 1)]);
  const committed = applyDocumentPlotMessages(opened, [
    frame("first", 2, "replace"),
    { ...frame("second", 0, "error"), warnings: ["query refused"] },
    frame("removed", 2),
  ]);
  assert.equal(committed.first.revision, 2);
  assert.equal(committed.second.status, "error");
  assert.equal(committed.idle, opened.idle);
  assert.equal(committed.removed, undefined);
  assert.equal(applyDocumentPlotMessages(committed, []), committed);
});

test("Outdated Frames Cannot Roll a Document Back", () => {
  const opened = applyDocumentPlotMessages({ first: emptyLivePlotState() }, [frame("first", 4)]);
  const committed = applyDocumentPlotMessages(opened, [
    frame("first", 5, "replace"),
    frame("first", 4, "replace"),
    frame("first", 5, "replace"),
  ]);
  assert.equal(committed.first.revision, 5);
  assert.equal(committed.first.droppedBatches, 2);
  assert.equal(opened.first.droppedBatches, 0);
});

test("The Native Decoder Preserves an Ordered Revision and Payload-Free Keepalives", () => {
  const revision = create(WatchFrameSchema, {
    updates: [
      {
        kind: PlotStreamKind.SNAPSHOT,
        envelope: {
          plotId: "healthy",
          revision: 12n,
          epoch: "epoch",
          selection: "healthy",
          payload: { stream: "telemetry", xKey: "timestamp", generatedAt: "2026-11-09T00:00:00Z", maxPoints: 100n },
          diagnostics: {},
        },
      },
      {
        kind: PlotStreamKind.ERROR,
        envelope: {
          plotId: "refused",
          epoch: "epoch",
          selection: "refused",
          warnings: ["query refused"],
        },
      },
    ],
  });
  const decoded = mapNativePlotStreamFrame(revision);
  assert.deepEqual(
    decoded.updates.map((update) => [update.plot_id, update.kind]),
    [
      ["healthy", "snapshot"],
      ["refused", "error"],
    ],
  );
  const pending = mapNativePlotStreamFrame(
    create(WatchFrameSchema, {
      updates: [{ kind: PlotStreamKind.UNSPECIFIED, envelope: { plotId: "healthy" } }],
    }),
  );
  assert.deepEqual(pending, { updates: [{ kind: "pending", plot_id: "healthy" }] });
  assert.throws(() => mapNativePlotStreamFrame(create(WatchFrameSchema)));
  assert.throws(() => mapNativePlotStreamFrame(create(WatchFrameSchema, { updates: [{}] })));
});
