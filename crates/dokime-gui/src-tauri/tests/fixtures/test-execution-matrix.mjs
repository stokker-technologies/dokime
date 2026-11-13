// Dokime — SPDX-License-Identifier: Apache-2.0
import assert from "node:assert/strict";
import { openHarness } from "./rpc-harness.mjs";

const harness = await openHarness();
let progress;
try {
  const api = await harness.loadHost("client.js");
  const types = await harness.loadHost("dokime/v1/client_types_pb.js");
  const scenario = process.argv[2];
  const session = { generation: process.env.DOKIME_TEST_GENERATION, isCurrent: () => true };
  // Materialize the SDK Fixture's Frozen Origin
  const catalog = await api.catalogListSuites(session, {});
  const suite = await api.catalogGetSuite(session, { suiteId: catalog.suites[0].suiteId });
  const plan = await api.planningMaterializeTemplate(session, { selection: {
    suiteId: suite.metadata.identifier, templateId: suite.templates[0].metadata.identifier,
  } });
  assert.equal(plan.cases.length, 1);
  const cycle = await api.executionCreateCycle(session, { origins: plan.cases.map(item => item.origin), continueAutomatically: true });
  const executionId = cycle.executionIds[0];
  // Await an Actual Prompt Phase; Never Wait for a Fixed Duration
  progress = await api.observationWatchExecutionProgress(session, { testExecutionId: executionId });
  const iterator = progress[Symbol.asyncIterator]();
  while (true) {
    const next = await iterator.next();
    assert.ok(!next.done, "Execution Must Reach Its Operator Question");
    if (next.value.progress?.phase === "prompt") break;
  }
  const pending = await api.promptsListPending(session, { executionId });
  assert.equal(pending.prompts.length, 1);
  if (scenario === "cancel") {
    await api.executionCancelExecution(session, { executionId });
  } else {
    await api.promptsRespond(session, { promptId: pending.prompts[0].promptId, action: scenario === "complete" ? "yes" : "no", accepted: true });
  }
  while (!(await iterator.next()).done) {}
  // The Host Seals Evidence for Completion, Refusal and Cancellation
  const detail = await api.executionGetExecution(session, { executionId });
  assert.equal(detail.summary.state, types.ExecutionState.SEALED);
  assert.ok(detail.seal.length);
  if (scenario === "cancel") {
    assert.equal(detail.outcome.attribution.case, "terminated");
  } else {
    assert.equal(detail.outcome.attribution.case, "evaluated");
    assert.equal(detail.outcome.attribution.value.status, scenario === "complete" ? types.ResultStatus.PASSED : types.ResultStatus.FAILED);
    const samples = await api.telemetryListSamples(session, { scope: { executionId } });
    assert.ok(samples.samples.length);
    const records = await api.evidenceListRecords(session, { selection: { scope: { executionId } } });
    assert.ok(records.records.some(record => record.payload.length));
  }
} finally {
  await progress?.cancel();
  await harness.close();
}
