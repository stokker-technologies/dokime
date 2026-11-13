// Dokime — SPDX-License-Identifier: Apache-2.0
import assert from "node:assert/strict";
import { openHarness } from "./rpc-harness.mjs";

const harness = await openHarness();
try {
  const api = await harness.loadHost("client.js");
  let active = true;
  const session = { generation: process.env.DOKIME_TEST_GENERATION, isCurrent: () => active };
  // The Frontend Has No Network Transport
  globalThis.fetch = () => { throw new Error("Desktop RPC Must Cross Tauri"); };
  const status = await api.statusGet(session, {});
  assert.equal(status.status.identity.hostId, process.env.DOKIME_TEST_BACKEND_ID);
  assert.ok(status.status.available);
  const suites = await api.catalogListSuites(session, {});
  assert.equal(suites.suites.length, 0);
  await assert.rejects(api.catalogGetSuite(session, { suiteId: "missing" }), error => error.kind === "grpc" && error.detail.code === 5);
  // Watch Opening and Cancellation Use the Same Descriptor Fence
  const watch = await api.observationWatchRuntime(session, {});
  const opening = await watch[Symbol.asyncIterator]().next();
  assert.equal(opening.done, false);
  assert.ok(opening.value.resynchronized);
  await watch.cancel();
  // A Late Reply Cannot Escape Its Replaced Frontend Generation
  const pending = api.catalogListSuites(session, {});
  active = false;
  await assert.rejects(pending, error => error.kind === "session" && error.detail.kind === "superseded");
} finally {
  await harness.close();
}
