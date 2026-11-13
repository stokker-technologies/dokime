/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Secret Catalog and Transfer
 */

import assert from "node:assert/strict";
import { create, fromBinary, toBinary } from "@bufbuild/protobuf";
import { SecretImportAction, SecretSnapshotSchema } from "@host/dokime/client/v1/secrets_pb.js";
import { beforeEach, test, vi } from "vitest";
import { SecretsImportRequestSchema } from "@host/dokime/client/v1/secrets_pb.js";
import { readSecrets, putSecret, removeSecret, previewSecrets, importSecrets } from "@/lib/api/native-secrets";

const fixture = vi.hoisted(() => ({ unary: vi.fn() }));
const session = { generation: "8", isCurrent: () => true };
const generation = 9007199254740993n;
vi.mock("@/lib/rpc/transport", () => ({ unary: fixture.unary }));

beforeEach(() => {
  fixture.unary.mockReset();
});

test("Secret Catalog Uses Canonical Metadata and Exact Generations", async () => {
  const snapshot = create(SecretSnapshotSchema, {
    hostId: "fixture-host",
    hostLabel: "Fixture Host",
    generation,
    entries: [
      {
        metadata: { identifier: "token", label: "Token", attributes: { count: { kind: { case: "integer", value: 7n } } } },
        knownUses: ["fixture"],
      },
    ],
  });
  const requests: { path: string; value: unknown }[] = [];
  const revision = new Uint8Array([1, 2, 3]);
  fixture.unary.mockImplementation(async (path, input, output, identity, request) => {
    assert.equal(identity, session);
    const value = fromBinary(input, toBinary(input, create(input, request)));
    requests.push({ path, value });
    const reply =
      path === "/harmos.v1.Status/Get"
        ? { status: { identity: { hostId: "fixture-host", applicationName: "Fixture Host" } } }
        : path.endsWith("/Get")
          ? { revision, value: { metadata: { identifier: "token" } } }
          : path.endsWith("/List")
            ? { generation, entries: snapshot.entries }
            : { receipt: { requestId: request.requestId, completed: true } };
    return fromBinary(output, toBinary(output, create(output, reply)));
  });
  const catalog = await readSecrets(session);
  assert.equal(catalog.generation, generation);
  assert.equal(catalog.entries[0].metadata.identifier, "token");
  assert.deepEqual(catalog.entries[0].knownUses, ["fixture"]);
  await putSecret(session, { identifier: "token", label: "Token" }, undefined, generation);
  await removeSecret(session, "token", generation);
  const put = requests.find((item) => item.path.endsWith("/Set"))?.value as {
    name: string;
    expectedRevision: Uint8Array;
    value: { metadata: { identifier: string }; value?: string };
  };
  const remove = requests.find((item) => item.path.endsWith("/Delete"))?.value as { names: string[]; expectedRevision: Uint8Array };
  assert.equal(put.name, "token");
  assert.equal(put.value.value, undefined);
  assert.equal(put.value.metadata.identifier, "token");
  assert.deepEqual(put.expectedRevision, revision);
  assert.deepEqual(remove.names, ["token"]);
  assert.deepEqual(remove.expectedRevision, revision);
  await assert.rejects(putSecret(session, { identifier: "token", label: "Token" }, undefined, generation - 1n));
});

test("Secret Transfer Sends Explicit Import Choices and Preserves Conflicts", async () => {
  const data = new Uint8Array([1, 2, 3]);
  const conflict = { kind: "conflict", message: "generation changed" };
  fixture.unary.mockImplementationOnce(async (_path, _input, output, _identity, request) => {
    assert.equal(request.validateOnly, true);
    return create(output, {
      preview: {
        hostId: "fixture-host",
        generation,
        entries: [
          { metadata: { identifier: "replace", label: "Replace" }, exists: true },
          { metadata: { identifier: "skip", label: "Skip" } },
        ],
      },
    });
  });
  const preview = await previewSecrets(session, data);
  fixture.unary.mockImplementationOnce(async (path, input, _output, _identity, request) => {
    assert.equal(path, "/dokime.client.v1.Secrets/Import");
    const decoded = fromBinary(SecretsImportRequestSchema, toBinary(input, create(input, request)));
    assert.equal(decoded.generation, generation);
    assert.deepEqual(decoded.data, data);
    assert.deepEqual(
      decoded.choices.map(({ identifier, action }) => [identifier, action]),
      [
        ["replace", SecretImportAction.REPLACE],
        ["skip", SecretImportAction.SKIP],
      ],
    );
    throw conflict;
  });
  await assert.rejects(importSecrets(session, data, preview, new Set(["replace"])), (error) => error === conflict);
});

test("Secret Catalog Rejects an Incomplete Host Reply", async () => {
  fixture.unary.mockResolvedValue(create(SecretSnapshotSchema, { entries: [{}] }));
  await assert.rejects(readSecrets(session), (error) => error instanceof Error);
});
