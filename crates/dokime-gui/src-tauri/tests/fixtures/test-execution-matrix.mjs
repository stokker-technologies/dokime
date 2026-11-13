// Dokime — SPDX-License-Identifier: Apache-2.0
import assert from "node:assert/strict";
import { openHarness } from "./rpc-harness.mjs";

const harness = await openHarness();
let progress;
try {
  const api = await harness.loadHost("client.js");
  const types = await harness.loadHost("dokime/v1/execution_pb.js");
  const execution = await harness.loadHost("dokime/v1/execution_pb.js");
  const scenario = process.argv[2];
  const session = { generation: process.env.DOKIME_TEST_GENERATION, isCurrent: () => true };
  // Materialize the SDK Fixture's Frozen Origin
  const registry = await api.suitesList(session, {});
  const suite = (await api.suitesGet(session, { name: registry.values[0].metadata.identifier })).value;
  const plan = await api.casesCreate(session, { plan: { selections: [{ suiteId: suite.metadata.identifier, templateId: suite.templates[0].metadata.identifier }] }, validateOnly: true, requestId: crypto.randomUUID() });
  assert.equal(plan.cases.length, 1);
  const cycle = (await api.cyclesCreate(session, { origins: plan.cases.map(item => item.origin), continueAutomatically: true, requestId: crypto.randomUUID() })).cycle;
  const executionId = (await api.executionsList(session, { parent: cycle.cycleId })).executions[0].summary.executionId;
  // Await an Actual Prompt Phase; Never Wait for a Fixed Duration
  progress = await api.executionsProgressWatch(session, { name: executionId });
  const iterator = progress[Symbol.asyncIterator]();
  while (true) {
    const next = await iterator.next();
    assert.ok(!next.done, "Execution Must Reach Its Operator Question");
    if (next.value.phase === "prompt") break;
  }
  const pending = await api.promptsList(session, { executionId, pendingOnly: true });
  assert.equal(pending.prompts.length, 1);
  if (scenario === "cancel") {
    await api.executionsCancel(session, { name: executionId, requestId: crypto.randomUUID() });
  } else {
    await api.promptsAnswersCreate(session, { parent: pending.prompts[0].prompt.promptId, requestId: crypto.randomUUID(), action: scenario === "complete" ? "yes" : "no", accepted: true });
  }
  while (!(await iterator.next()).done) {}
  // The Host Seals Evidence for Completion, Refusal and Cancellation
  const detail = (await api.executionsGet(session, { name: executionId })).execution;
  assert.equal(detail.summary.state, types.ExecutionState.SEALED);
  assert.ok(detail.seal.length);
  if (scenario === "cancel") {
    assert.equal(detail.outcome.attribution.case, "notEvaluated");
    assert.equal(detail.outcome.attribution.value.termination, execution.Termination.CANCELLED);
  } else {
    assert.equal(detail.outcome.attribution.case, "evaluated");
    assert.equal(detail.outcome.attribution.value.status, scenario === "complete" ? types.ResultStatus.PASSED : types.ResultStatus.FAILED);
    const samples = await api.evidenceSamplesList(session, { scope: { executionId } });
    assert.ok(samples.samples.length);
    const records = await api.evidenceRecordsList(session, { selection: { scope: { executionId } } });
    const door = records.records.find(record => record.recordType === "thermal.door");
    assert.ok(door);
    assert.deepEqual([...door.payload], scenario === "complete" ? [8, 1] : []);
  }
} finally {
  await progress?.cancel();
  await harness.close();
}
