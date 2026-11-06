/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Incremental Document Stream Commits
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { create } from "@bufbuild/protobuf";
import { test } from "vitest";

// Local Imports
import * as wire from "@generated/dokime/v1/DocumentsMessages/CaptureCurrentDocument/messages_pb";
import { assembleDocument, type CompletedDocument, mapNativeDocument, nativeDocumentCardKey } from "@/lib/api/native-documents";

/* === Re-Exports === */

/* === Constants === */

const scope = { cycleId: "00000000-0000-4000-8000-000000000001", executionId: "00000000-0000-4000-8000-000000000002" };
const expected = { scope, documentId: "default", epoch: "fixture.epoch" };
const cut = { store: "fixture.store", epoch: "fixture.history", position: 7n };

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function frame(kind: string, payload: unknown) {
  return create(wire.Row3Schema, { frame: { HarmosVariant: { case: kind, value: { field0: payload } } } } as never);
}

function binding(id: string, ready = false) {
  return frame("Binding", {
    sourceId: id,
    metadata: { identifier: id, label: "Fixture" },
    result: {
      HarmosVariant: { case: ready ? "Summary" : "Pending", value: { field0: ready ? { groupCount: 1n } : true } },
    },
  });
}

function counts(length: number, partial: boolean) {
  return { frameCount: BigInt(length), cardCount: 2n, layoutNodeCount: 2n, bindingCount: 2n, repeatCount: 0n, sectionCount: 0n, partial };
}

function initial(partial = false) {
  const frames = [
    frame("Header", {
      documentId: expected.documentId,
      metadata: { identifier: "default", label: "Fixture" },
      scope,
      frontiers: { processEpoch: expected.epoch, stateRevision: 1n },
    }),
  ];
  for (const [ordinal, id] of ["fast", "slow"].entries()) {
    frames.push(
      frame("LayoutNode", { ordinal: BigInt(ordinal), node: { HarmosVariant: { case: "Placement", value: { field0: { cardId: id } } } } }),
    );
    frames.push(
      frame("Card", {
        metadata: { identifier: id, label: "Fixture" },
        filters: [],
        body: { HarmosVariant: { case: "Summary", value: { field0: { groupCount: 1n } } } },
      }),
    );
    frames.push(
      frame("GroupDefinition", { cardId: id, ordinal: 0n, metadata: { identifier: `${id}.group`, label: "Fixture" }, tileCount: 2n }),
    );
    for (let tile = 0; tile < 2; tile++)
      frames.push(
        frame("TileDefinition", {
          cardId: id,
          groupOrdinal: 0n,
          ordinal: BigInt(tile),
          metadata: { identifier: `${id}.tile.${tile}`, label: "Fixture" },
          value: { source: { HarmosVariant: { case: "Parameter", value: { field0: { name: "fixture.value" } } } }, transforms: [] },
        }),
      );
    frames.push(binding(id));
  }
  frames.push(frame("StructureComplete", counts(frames.length + 1, partial)));
  frames.push(frame("EvidenceCut", { cut, telemetry: true, cacheable: false }));
  return frames;
}

function tile(ordinal: number) {
  return frame("SummaryTile", {
    cardId: "fast",
    groupOrdinal: 0n,
    ordinal: BigInt(ordinal),
    metadata: { identifier: `fast.tile.${ordinal}`, label: "Fixture" },
    value: { nodes: [{ kind: { HarmosVariant: { case: "Integer", value: { field0: BigInt(ordinal + 1) } } } }] },
  });
}

async function* iterable(frames: ReturnType<typeof frame>[]) {
  yield* frames;
}

test("Completed Cards Survive An Interrupted Sibling", async () => {
  const updates: CompletedDocument[] = [];
  async function* frames() {
    yield* initial();
    yield binding("fast", true);
    yield frame("SummaryGroup", { cardId: "fast", ordinal: 0n, metadata: { identifier: "fast.group", label: "Fixture" }, tileCount: 2n });
    yield tile(0);
    assert.equal(updates.at(-1)?.bindings.get("fast")?.result?.HarmosVariant.case, "Pending");
    yield tile(1);
    yield frame("CardComplete", "fast");
    assert.equal(updates.at(-1)?.bindings.get("fast")?.result?.HarmosVariant.case, "Summary");
    throw new Error("fixture.interrupted");
  }
  await assert.rejects(
    assembleDocument(
      frames(),
      expected,
      () => true,
      (capture) => updates.push(capture),
    ),
  );
  assert.equal(updates[0]?.bindings.get("fast")?.result?.HarmosVariant.case, "Pending");
  assert.equal(updates.at(-1)?.bindings.get("slow")?.result?.HarmosVariant.case, "Pending");
  const last = updates.at(-1);
  assert.ok(last);
  const mapped = mapNativeDocument(last);
  assert.deepEqual(
    mapped.cards.find((card) => card.card_id === "fast")?.groups[0]?.metrics.map((tile) => tile.value),
    [1, 2],
  );
});

test("Structure Projection And Binding Identity", async () => {
  const frames = initial(true);
  frames.push(frame("Completion", counts(frames.length + 1, true)));
  const capture = await assembleDocument(iterable(frames), { ...expected, structureOnly: true }, () => true);
  assert.equal(capture.complete, true);
  assert.ok(mapNativeDocument(capture).cards.every((card) => card.loading));
  assert.ok(capture.header.frontiers);
  const next = {
    ...capture,
    header: { ...capture.header, frontiers: { ...capture.header.frontiers, stateRevision: 2n, evidence: { ...cut, position: 9n } } },
  };
  assert.equal(nativeDocumentCardKey(capture, "fast"), nativeDocumentCardKey(next, "fast"));
  const source = capture.cards.get("fast");
  assert.ok(source?.metadata);
  const changed = { ...capture, cards: new Map(capture.cards) };
  changed.cards.set("fast", { ...source, metadata: { ...source.metadata, tags: [...source.metadata.tags, "fixture.changed"] } });
  assert.notEqual(nativeDocumentCardKey(capture, "fast"), nativeDocumentCardKey(changed, "fast"));
});

test("A Budget Failure Keeps The Completed Sibling", async () => {
  const budget = { name: "frame_bytes", limit: 4096n, reached: 32768n, cardId: "slow" };
  const frames = initial();
  frames.push(
    binding("fast", true),
    frame("SummaryGroup", { cardId: "fast", ordinal: 0n, metadata: { identifier: "fast.group", label: "Fixture" }, tileCount: 2n }),
    tile(0),
    tile(1),
    frame("CardComplete", "fast"),
    frame("Binding", {
      sourceId: "slow",
      metadata: { identifier: "slow", label: "Fixture" },
      result: {
        HarmosVariant: {
          case: "Failure",
          value: {
            field0: {
              code: budget.name,
              message: `${budget.name}: ${budget.reached} > ${budget.limit}, ${budget.cardId}`,
              budget,
            },
          },
        },
      },
    }),
    frame("CardComplete", "slow"),
  );
  frames.push(frame("Completion", counts(frames.length + 1, false)));
  const capture = await assembleDocument(iterable(frames), expected, () => true);
  const mapped = mapNativeDocument(capture);
  assert.deepEqual(
    mapped.cards.find((card) => card.card_id === "fast")?.groups[0]?.metrics.map((tile) => tile.value),
    [1, 2],
  );
  const failed = mapped.cards.find((card) => card.card_id === budget.cardId);
  assert.ok(failed);
  assert.ok(!failed.loading);
  assert.equal(failed?.failure?.reason, budget.name);
  for (const value of [budget.name, budget.cardId, String(budget.limit), String(budget.reached)])
    assert.ok(failed?.failure?.message.includes(value));
});

/* === Entry Point === */
