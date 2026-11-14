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
import { DocumentFrameSchema } from "@host/dokime/client/v1/documents_pb.js";
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
  return create(DocumentFrameSchema, { frame: { kind: { case: kind, value: payload } } } as never);
}

function binding(id: string, ready = false) {
  return frame("binding", {
    sourceId: id,
    metadata: { identifier: id, label: "Fixture" },
    result: {
      kind: ready ? { case: "summary", value: { groupCount: 1n } } : { case: "pending", value: true },
    },
  });
}

function counts(length: number, partial: boolean) {
  return { frameCount: BigInt(length), cardCount: 2n, layoutNodeCount: 2n, bindingCount: 2n, repeatCount: 0n, sectionCount: 0n, partial };
}

function initial(partial = false) {
  const frames = [
    frame("header", {
      documentId: expected.documentId,
      metadata: { identifier: "default", label: "Fixture" },
      scope,
      frontiers: { processEpoch: expected.epoch, stateRevision: 1n },
    }),
  ];
  for (const [ordinal, id] of ["fast", "slow"].entries()) {
    frames.push(frame("layoutNode", { ordinal: BigInt(ordinal), node: { kind: { case: "placement", value: { cardId: id } } } }));
    frames.push(
      frame("card", {
        metadata: { identifier: id, label: "Fixture" },
        filters: [],
        body: { kind: { case: "summary", value: { groupCount: 1n } } },
      }),
    );
    frames.push(
      frame("groupDefinition", { cardId: id, ordinal: 0n, metadata: { identifier: `${id}.group`, label: "Fixture" }, tileCount: 2n }),
    );
    for (let tile = 0; tile < 2; tile++)
      frames.push(
        frame("tileDefinition", {
          cardId: id,
          groupOrdinal: 0n,
          ordinal: BigInt(tile),
          metadata: { identifier: `${id}.tile.${tile}`, label: "Fixture" },
          value: { kind: { case: "parameter", value: "fixture.value" } },
        }),
      );
    frames.push(binding(id));
  }
  frames.push(frame("structureComplete", counts(frames.length + 1, partial)));
  frames.push(frame("evidenceCut", { cut, telemetry: true, cacheable: false }));
  return frames;
}

function tile(ordinal: number) {
  return frame("summaryTile", {
    cardId: "fast",
    groupOrdinal: 0n,
    ordinal: BigInt(ordinal),
    metadata: { identifier: `fast.tile.${ordinal}`, label: "Fixture" },
    value: { kind: { case: "integer", value: BigInt(ordinal + 1) } },
  });
}

async function* iterable(frames: ReturnType<typeof frame>[]) {
  yield* frames;
}

test("Completed Cards Survive an Interrupted Sibling", async () => {
  const updates: CompletedDocument[] = [];
  async function* frames() {
    yield* initial();
    yield binding("fast", true);
    yield frame("summaryGroup", { cardId: "fast", ordinal: 0n, metadata: { identifier: "fast.group", label: "Fixture" }, tileCount: 2n });
    yield tile(0);
    assert.equal(updates.at(-1)?.bindings.get("fast")?.result?.kind.case, "pending");
    yield tile(1);
    yield frame("cardComplete", "fast");
    assert.equal(updates.at(-1)?.bindings.get("fast")?.result?.kind.case, "summary");
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
  assert.equal(updates[0]?.bindings.get("fast")?.result?.kind.case, "pending");
  assert.equal(updates.at(-1)?.bindings.get("slow")?.result?.kind.case, "pending");
  const last = updates.at(-1);
  assert.ok(last);
  const mapped = mapNativeDocument(last);
  assert.deepEqual(
    mapped.cards.find((card) => card.card_id === "fast")?.groups[0]?.metrics.map((tile) => tile.value),
    [1, 2],
  );
});

test("Structure Projection and Binding Identity", async () => {
  const frames = initial(true);
  frames.push(frame("completion", counts(frames.length + 1, true)));
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

test("Page Tokens Preserve Binding Identity", async () => {
  const frames = initial(true);
  const page = { current: "fixture.first", next: "fixture.second" };
  frames.push(frame("page", page));
  frames.push(frame("expansionComplete", false));
  frames.push(frame("completion", counts(frames.length + 1, true)));
  const captured = await assembleDocument(iterable(frames), { ...expected, structureOnly: true }, () => true);
  assert.deepEqual(captured.page, page);
  const following = { ...captured, page: { current: page.next } };
  assert.equal(nativeDocumentCardKey(captured, "fast"), nativeDocumentCardKey(following, "fast"));
});

test("A Budget Failure Keeps the Completed Sibling", async () => {
  const budget = { name: "frame_bytes", limit: 4096n, reached: 32768n, cardId: "slow" };
  const frames = initial();
  frames.push(
    binding("fast", true),
    frame("summaryGroup", { cardId: "fast", ordinal: 0n, metadata: { identifier: "fast.group", label: "Fixture" }, tileCount: 2n }),
    tile(0),
    tile(1),
    frame("cardComplete", "fast"),
    frame("binding", {
      sourceId: "slow",
      metadata: { identifier: "slow", label: "Fixture" },
      result: {
        kind: {
          case: "failure",
          value: {
            code: budget.name,
            message: `${budget.name}: ${budget.reached} > ${budget.limit}, ${budget.cardId}`,
            budget,
          },
        },
      },
    }),
    frame("cardComplete", "slow"),
  );
  frames.push(frame("completion", counts(frames.length + 1, false)));
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
