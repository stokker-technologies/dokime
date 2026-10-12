// Dokime — SPDX-License-Identifier: Apache-2.0
// Test-only stdio shim: the Rust test drives Tauri's actual native dispatcher.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { openHarness } from "./rpc-harness.mjs";
const require = createRequire(new URL("../../../../package.json", import.meta.url));
const { create, fromBinary, toBinary } = require("@bufbuild/protobuf");
const harness = openHarness();
const { invoke, load } = harness;
try {
  const api = await load("generated/client.js");
  const value = await load("generated/dokime/v1/value_pb.js");
  const execution = await load("generated/dokime/v1/execution_pb.js");
  const session = { generation: "1", isCurrent: () => true };
  const { loadAppearance } = await load("appearance.js");
  const appearance = await loadAppearance(session);
  assert.equal(appearance.appearance.appName, "Dokime");
  assert.equal(appearance.appearance.organization.name, "Stokker Technologies");
  assert.equal(appearance.appearance.themes.length, 1);
  assert.ok(appearance.appearance.themes[0].stylesheet.includes("@font-face"));
  assert.ok(appearance.logoUrl.startsWith("blob:"));
  const logo = await (await fetch(appearance.logoUrl)).arrayBuffer();
  assert.equal(
    createHash("sha256").update(new Uint8Array(logo)).digest("hex"),
    appearance.appearance.logo.asset.reference.sha256,
  );
  appearance.dispose();
  appearance.dispose();
  await assert.rejects(fetch(appearance.logoUrl));
  const suites = await api.catalogListSuites(session, { limit: 10 });
  assert.equal(suites.suites.length, 1);
  assert.equal(suites.suites[0].suiteId, "rpc.artifact");
  assert.equal(typeof suites.cursor.position, "bigint");
  await assert.rejects(
    api.catalogGetSuite(session, { suiteId: "missing" }),
    (error) => error.kind === "grpc" && error.detail.code === 5,
  );
  const recordSelection = { recordTypes: ["calibration.field.completed"] };
  const records = await api.evidenceListRecords(session, { selection: recordSelection });
  assert.equal(records.records.length, 1);
  assert.equal(typeof records.records[0].sequence, "bigint");
  assert.equal(typeof records.records[0].timestamp.seconds, "bigint");
  const recordSchema = await api.evidenceGetCapturedRecordSchema(session, {
    recordType: records.records[0].recordType,
    schemaVersion: records.records[0].schemaVersion,
    epoch: records.scanned.epoch,
  });
  assert.equal(recordSchema.schemaVersion, records.records[0].schemaVersion);
  const facts = await api.evidenceWatchRecords(session, { selection: recordSelection });
  const factPage = await facts[Symbol.asyncIterator]().next();
  assert.equal(factPage.value.records[0].recordId, records.records[0].recordId);
  await facts.cancel();
  const stream = await api.observationWatchRuntime(session, {});
  const frame = await stream[Symbol.asyncIterator]().next();
  assert.equal(frame.done, false);
  assert.equal(frame.value.resynchronized, true);
  assert.equal(frame.value.cycleCount, 1n);
  await stream.cancel();
  let active = true;
  const late = api.catalogListSuites({ generation: "1", isCurrent: () => active }, {});
  active = false;
  await assert.rejects(
    late,
    (error) => error.kind === "session" && error.detail.kind === "superseded",
  );
  active = true;
  const lateError = api.catalogGetSuite(
    { generation: "1", isCurrent: () => active },
    { suiteId: "missing" },
  );
  active = false;
  await assert.rejects(
    lateError,
    (error) => error.kind === "session" && error.detail.kind === "superseded",
  );
  active = true;
  const lateOpen = api.contentDownloadArtifact(
    { generation: "1", isCurrent: () => active },
    { artifactId: "missing" },
  );
  active = false;
  await assert.rejects(
    lateOpen,
    (error) => error.kind === "session" && error.detail.kind === "superseded",
  );
  active = true;
  const staleStream = await api.observationWatchRuntime(
    { generation: "1", isCurrent: () => active },
    {},
  );
  await invoke("fixture_cancel_streams", new Uint8Array());
  const lateRead = staleStream[Symbol.asyncIterator]().next();
  active = false;
  await assert.rejects(
    lateRead,
    (error) => error.kind === "session" && error.detail.kind === "superseded",
  );
  // A download opens with the artifact's metadata, so the stream is the only
  // request needed to learn what it is about to deliver.
  const download = await api.contentDownloadArtifact(session, {
    artifactId: process.env.DOKIME_TEST_ARTIFACT_ID,
  });
  const received = [];
  let metadata = null,
    offset = 0n,
    complete = false;
  for await (const frame of download) {
    assert.equal(complete, false, "completion must be last data frame");
    switch (frame.payload.case) {
      case "metadata":
        assert.equal(metadata, null);
        metadata = frame.payload.value;
        assert.equal(metadata.artifactId, process.env.DOKIME_TEST_ARTIFACT_ID);
        assert.equal(metadata.sizeBytes, 150000n);
        break;
      case "chunk": {
        assert.notEqual(metadata, null);
        const chunk = frame.payload.value;
        assert.equal(chunk.offset, offset);
        assert.ok(chunk.data.length > 0 && chunk.data.length <= 65536);
        offset += BigInt(chunk.data.length);
        received.push(chunk.data);
        break;
      }
      case "complete":
        assert.equal(frame.payload.value.sizeBytes, offset);
        assert.equal(frame.payload.value.sha256, metadata.sha256);
        complete = true;
        break;
      default:
        assert.fail("missing artifact payload");
    }
  }
  // Reaching iterator EOF also proves an OK final gRPC status after completion.
  assert.equal(complete, true);
  const content = Buffer.concat(received);
  assert.equal(createHash("sha256").update(content).digest("hex"), metadata.sha256);
  assert.deepEqual(content, Buffer.from(Array.from({ length: 150000 }, (_, i) => i % 251)));
  const values = create(value.ValueObjectSchema, {
    fields: {
      minimum: { kind: { case: "integerValue", value: -9223372036854775808n } },
      maximum: { kind: { case: "integerValue", value: 9223372036854775807n } },
      null: { kind: { case: "nullValue", value: 0 } },
      unset: {},
    },
  });
  const valueReply = fromBinary(
    value.ValueObjectSchema,
    new Uint8Array(await invoke("fixture_value", toBinary(value.ValueObjectSchema, values))),
  );
  assert.deepEqual(valueReply, values);
  const admission = create(execution.AdmissionSchema, {
    cursor: { position: 18446744073709551615n },
    acceptedAt: { seconds: -1n, nanos: 999999999 },
  });
  const admissionReply = fromBinary(
    execution.AdmissionSchema,
    new Uint8Array(
      await invoke("fixture_admission", toBinary(execution.AdmissionSchema, admission)),
    ),
  );
  assert.deepEqual(admissionReply, admission);
} finally {
  harness.close();
}
