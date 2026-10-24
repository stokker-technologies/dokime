// Dokime — SPDX-License-Identifier: Apache-2.0
// Test-only stdio shim: the Rust test drives Tauri's actual native dispatcher.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";
import { openHarness } from "./rpc-harness.mjs";
const require = createRequire(new URL("../../../package.json", import.meta.url));
const { create, fromBinary, toBinary } = require("@bufbuild/protobuf");
const harness = await openHarness();
const { invoke, load } = harness;
try {
  const api = await load("generated/client.js");
  const value = await harness.loadCanonical("dokime/v1/CatalogMessages/GetSuite/messages_pb.js");
  const execution = await harness.loadCanonical("dokime/v1/ExecutionMessages/StartExecution/messages_pb.js");
  const generation = process.env.DOKIME_TEST_GENERATION;
  function connected(isCurrent = () => true) {
    return { generation, isCurrent };
  }
  const session = connected();
  // The WebView holds no endpoint and no secret: every call crosses the
  // bridge, and the only fetches left are of blobs this page created.
  const pageFetch = globalThis.fetch;
  globalThis.fetch = (input, init) => {
    const url = String(input?.url ?? input);
    if (!url.startsWith("blob:")) throw new Error(`The desktop transport fetched ${url}`);
    return pageFetch(input, init);
  };
  const identity = await api.backendGetIdentity(session, {});
  assert.equal(identity.backendId, process.env.DOKIME_TEST_BACKEND_ID);
  const { loadAppearance } = await load("appearance.js");
  const appearance = await loadAppearance(session);
  assert.equal(appearance.appearance.appName, "Dokime");
  assert.equal(appearance.appearance.organization.name, "Stokker Technologies");
  assert.equal(appearance.appearance.themes.length, 1);
  assert.ok(appearance.appearance.themes[0].stylesheet.includes("@font-face"));
  assert.equal(appearance.logos.length, 4);
  for (const [index, logo] of appearance.logos.entries()) {
    assert.ok(logo.url.startsWith("blob:"));
    const bytes = await (await fetch(logo.url)).arrayBuffer();
    assert.equal(createHash("sha256").update(new Uint8Array(bytes)).digest("hex"), appearance.appearance.logos[index].asset.reference.sha256);
  }
  appearance.dispose();
  appearance.dispose();
  for (const logo of appearance.logos) await assert.rejects(fetch(logo.url));
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
  const late = api.catalogListSuites(connected(() => active), {});
  active = false;
  await assert.rejects(
    late,
    (error) => error.kind === "session" && error.detail.kind === "superseded",
  );
  active = true;
  const lateError = api.catalogGetSuite(
    connected(() => active),
    { suiteId: "missing" },
  );
  active = false;
  await assert.rejects(
    lateError,
    (error) => error.kind === "session" && error.detail.kind === "superseded",
  );
  active = true;
  const lateOpen = api.contentDownloadArtifact(
    connected(() => active),
    { artifactId: "missing" },
  );
  active = false;
  await assert.rejects(
    lateOpen,
    (error) => error.kind === "session" && error.detail.kind === "superseded",
  );
  active = true;
  const staleStream = await api.observationWatchRuntime(
    connected(() => active),
    {},
  );
  const lateRead = staleStream[Symbol.asyncIterator]().next();
  active = false;
  await assert.rejects(
    lateRead,
    (error) => error.kind === "session" && error.detail.kind === "superseded",
  );
  // Full-Control Access Permits Staging; Invalid Bytes Retain The Backend's Refusal.
  await assert.rejects(
    (async () => {
      const staging = await api.pluginsStage(session, {
        frame: {
          HarmosVariant: {
            case: "Header",
            value: { field0: { sizeBytes: 1n, sha256: "0".repeat(64) } },
          },
        },
      });
      await staging.finish();
    })(),
    (error) => error.kind === "grpc" && error.detail.code === 3 && error.message.includes("stage size or digest mismatch"),
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
    switch (frame.payload.HarmosVariant.case) {
      case "Metadata":
        assert.equal(metadata, null);
        metadata = frame.payload.HarmosVariant.value.field0;
        assert.equal(metadata.artifactId, process.env.DOKIME_TEST_ARTIFACT_ID);
        assert.equal(metadata.sizeBytes, 1200000n);
        break;
      case "Chunk": {
        assert.notEqual(metadata, null);
        const chunk = frame.payload.HarmosVariant.value.field0;
        assert.equal(chunk.offset, offset);
        // A download chunk is at most half the fixture server's 1 MiB local message budget.
        assert.ok(chunk.data.length > 0 && chunk.data.length <= 512 * 1024);
        offset += BigInt(chunk.data.length);
        received.push(chunk.data);
        break;
      }
      case "Complete":
        assert.equal(frame.payload.HarmosVariant.value.field0.sizeBytes, offset);
        assert.equal(frame.payload.HarmosVariant.value.field0.sha256, metadata.sha256);
        complete = true;
        break;
      default:
        assert.fail("missing artifact payload");
    }
  }
  // Reaching iterator EOF also proves an OK final gRPC status after completion.
  assert.equal(complete, true);
  assert.ok(received.length > 1, "the artifact spans several contiguous chunks");
  const content = Buffer.concat(received);
  assert.equal(createHash("sha256").update(content).digest("hex"), metadata.sha256);
  assert.deepEqual(content, Buffer.from(Array.from({ length: 1200000 }, (_, i) => i % 251)));
  const values = create(value.Row19Schema, {
    nodes: [
      { kind: { HarmosVariant: { case: "Object", value: { field0: { fields: [
        { key: "minimum", value: 1 }, { key: "maximum", value: 2 },
        { key: "null", value: 3 }, { key: "unset", value: 4 },
      ] } } } } },
      { kind: { HarmosVariant: { case: "Integer", value: { field0: -9223372036854775808n } } } },
      { kind: { HarmosVariant: { case: "Integer", value: { field0: 9223372036854775807n } } } },
      { kind: { HarmosVariant: { case: "Null", value: {} } } },
      {},
    ],
  });
  const valueReply = fromBinary(
    value.Row19Schema,
    new Uint8Array(await invoke("fixture_value", toBinary(value.Row19Schema, values))),
  );
  assert.deepEqual(valueReply, values);
  const admission = create(execution.Row9Schema, {
    cursor: { position: 18446744073709551615n },
    acceptedAt: { seconds: -1n, nanos: 999999999 },
  });
  const admissionReply = fromBinary(
    execution.Row9Schema,
    new Uint8Array(
      await invoke("fixture_admission", toBinary(execution.Row9Schema, admission)),
    ),
  );
  assert.deepEqual(admissionReply, admission);
} finally {
  await harness.close();
}
