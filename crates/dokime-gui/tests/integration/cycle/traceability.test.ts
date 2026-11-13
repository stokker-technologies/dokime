/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { create } from "@bufbuild/protobuf";
import { beforeEach, test, vi } from "vitest";

// Local Imports
import { TraceabilityGetResponseSchema } from "@host/dokime/v1/evidence/traceability_pb.js";
import { fetchNativeTraceability } from "@/lib/api/native-observation";
import type { Wire } from "@/lib/rpc/canonical";

/* === Re-Exports === */

/* === Constants === */

const boundary = vi.hoisted(() => ({ unary: vi.fn() }));
vi.mock("@/lib/rpc/transport", () => ({ unary: boundary.unary }));
const session = { generation: "7", isCurrent: () => true };
const scope = { test_cycle_id: "cycle", test_execution_id: "execution", step_id: "step" };

/* === Types === */

type Response = Wire<ReturnType<typeof create<typeof TraceabilityGetResponseSchema>>>;

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

function page(sequence: bigint, nextPageToken = "") {
  return create(TraceabilityGetResponseSchema, {
    resolvedAt: { positions: [{ history: { store: "store", epoch: "execution" }, sequence: 9n }] },
    nextPageToken,
    snapshots: [
      {
        traceabilityId: `execution/${sequence}`,
        scope: { executionId: "execution", cycleId: "cycle", stepId: "step" },
        createdAt: { seconds: 100n, nanos: 0 },
        revision: new Uint8Array([Number(sequence)]),
        resolvedAt: { history: { store: "store", epoch: "execution" }, sequence },
        entries: [
          {
            metadata: { identifier: `entry-${sequence}`, label: `Entry ${sequence}` },
            value: { kind: { case: "text", value: `value-${sequence}` } },
            source: "fixture",
            unit: "V",
          },
        ],
      },
    ],
  });
}

/* === Entry Point === */

beforeEach(() => boundary.unary.mockReset());

test("Traceability Pages Retain Scope, Read Cut and Snapshot Order", async () => {
  const first = page(2n, "opaque-page");
  boundary.unary.mockResolvedValueOnce(first).mockResolvedValueOnce(page(5n));
  const result = await fetchNativeTraceability(session, scope);
  assert.equal(boundary.unary.mock.calls.length, 2);
  for (const call of boundary.unary.mock.calls) {
    assert.equal(call[0], "/dokime.v1.evidence.Traceability/Get");
    assert.equal(call[3], session);
    assert.equal(call[4].scope.cycleId, "cycle");
    assert.equal(call[4].scope.executionId, "execution");
    assert.equal(call[4].scope.stepId, "step");
  }
  assert.deepEqual(boundary.unary.mock.calls[1][4].scope, boundary.unary.mock.calls[0][4].scope);
  assert.equal(boundary.unary.mock.calls[0][4].at, undefined);
  assert.deepEqual(boundary.unary.mock.calls[1][4].at, first.resolvedAt);
  assert.equal(boundary.unary.mock.calls[1][4].page.pageToken, "opaque-page");
  assert.equal(result.scope?.test_execution_id, "execution");
  assert.deepEqual(
    result.groups.flatMap((group) => group.items.map((item) => item.value)),
    ["value-2", "value-5"],
  );
});

test("Empty Traceability Keeps Its Requested Scope and Cut", async () => {
  const response = page(1n);
  response.snapshots = [];
  boundary.unary.mockResolvedValueOnce(response);
  const result = await fetchNativeTraceability(session, scope);
  assert.deepEqual(result.groups, []);
  assert.equal(result.scope?.step_id, "step");
});

test.each([
  [
    "Missing Cut",
    (response: Response) => {
      response.resolvedAt = undefined;
    },
  ],
  [
    "Foreign Cut",
    (response: Response) => {
      response.resolvedAt!.positions[0].history!.epoch = "other";
    },
  ],
  [
    "Duplicate History",
    (response: Response) => {
      response.resolvedAt!.positions.push(response.resolvedAt!.positions[0]);
    },
  ],
  [
    "Missing History",
    (response: Response) => {
      response.resolvedAt!.positions[0].history = undefined;
    },
  ],
  [
    "Revision Cut",
    (response: Response) => {
      response.resolvedAt!.revisions.push({ name: "resource", revision: new Uint8Array([1]) });
    },
  ],
  [
    "Missing Revision",
    (response: Response) => {
      response.snapshots[0].revision = new Uint8Array();
    },
  ],
  [
    "Outside Cut",
    (response: Response) => {
      response.snapshots[0].resolvedAt!.sequence = 10n;
    },
  ],
  [
    "Foreign Store",
    (response: Response) => {
      response.snapshots[0].resolvedAt!.history!.store = "other";
    },
  ],
  [
    "Foreign Scope",
    (response: Response) => {
      response.snapshots[0].scope!.stepId = "other";
    },
  ],
  [
    "Missing Scope",
    (response: Response) => {
      response.snapshots[0].scope = undefined;
    },
  ],
  [
    "Missing Identity",
    (response: Response) => {
      response.snapshots[0].traceabilityId = "";
    },
  ],
  [
    "Missing Timestamp",
    (response: Response) => {
      response.snapshots[0].createdAt = undefined;
    },
  ],
  [
    "Missing Metadata",
    (response: Response) => {
      response.snapshots[0].entries[0].metadata = undefined;
    },
  ],
  [
    "Missing Value",
    (response: Response) => {
      response.snapshots[0].entries[0].value = undefined;
    },
  ],
  [
    "Unknown Value",
    (response: Response) => {
      response.snapshots[0].entries[0].value = { kind: { case: undefined } };
    },
  ],
] as const)("Traceability Rejects %s", async (_name, corrupt) => {
  const response = page(2n);
  corrupt(response);
  boundary.unary.mockResolvedValueOnce(response);
  await assert.rejects(fetchNativeTraceability(session, scope), (error: Error) => {
    assert.ok(error.message.startsWith("Evidence.Traceability.Get"));
    return true;
  });
});

test("Traceability Rejects Changed Cuts Between Pages", async () => {
  const second = page(5n);
  second.resolvedAt!.positions[0].sequence = 10n;
  boundary.unary.mockResolvedValueOnce(page(2n, "next")).mockResolvedValueOnce(second);
  await assert.rejects(fetchNativeTraceability(session, scope), /Evidence.Traceability.Get/);
});

test.each(["Repeated Token", "Empty Continuation", "Repeated Snapshot", "Reversed Snapshot"])("Traceability Rejects %s", async (kind) => {
  const second = page(
    kind === "Reversed Snapshot" ? 1n : kind === "Repeated Snapshot" ? 2n : 5n,
    kind === "Repeated Token" ? "next" : kind === "Empty Continuation" ? "another" : "",
  );
  if (kind === "Empty Continuation") second.snapshots = [];
  boundary.unary.mockResolvedValueOnce(page(2n, "next")).mockResolvedValueOnce(second);
  await assert.rejects(fetchNativeTraceability(session, scope), /Evidence.Traceability.Get/);
});

test("Traceability Refuses a Replaced Connection After Awaiting a Page", async () => {
  let current = true;
  boundary.unary.mockImplementationOnce(async () => {
    current = false;
    return page(2n);
  });
  await assert.rejects(fetchNativeTraceability({ generation: "7", isCurrent: () => current }, scope), /Evidence.Traceability.Get/);
});
