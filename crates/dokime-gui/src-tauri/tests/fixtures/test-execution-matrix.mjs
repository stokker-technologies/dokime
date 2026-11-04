/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Desktop client driver for shared execution scenarios
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";
import { createHash, randomUUID } from "node:crypto";
import { readFileSync } from "node:fs";

// Third Party Imports

// Local Imports
import { openHarness } from "./rpc-harness.mjs";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

const harness = await openHarness();
let changes;
try {
  const contract = JSON.parse(readFileSync(process.argv[2], "utf8"));
  const scenario = contract.scenarios.find((item) => item.name === process.argv[3]);
  const api = await harness.load("generated/client.js");
  const session = { generation: process.env.DOKIME_TEST_GENERATION, isCurrent: () => true };
  changes = await api.observationWatchRuntime(session, {});
  const changed = changes[Symbol.asyncIterator]();
  async function nextChange() {
    assert.ok(!(await changed.next()).done, "runtime observation ended before the condition");
  }
  const suite = await api.catalogGetSuite(session, { suiteId: contract.suite });
  const template = suite.templates.find((item) => item.metadata.identifier === contract.template);
  assert.ok(template);
  // Materialize Cases And Append Explicit Unstarted Attempts
  const cycle = await api.executionCreateCycle(session, {
    metadata: { identifier: `matrix.${scenario.name}`, label: "Execution Matrix" },
  });
  const cycleId = cycle.cycleId;
  const registered = await api.planningMaterializeTemplate(session, {
    selection: { suiteId: suite.identity.suiteId, templateId: template.templateId },
  });
  assert.equal(registered.cases.length, 1);
  const liveExecutionId = randomUUID();
  await api.executionAppendExecutions(session, {
    cycleId,
    executions: [{ executionId: liveExecutionId, caseId: registered.cases[0].caseId }],
  });
  const admission = await api.executionStartExecution(session, {
    executionId: liveExecutionId,
    mode: { HarmosVariant: { case: "Automatic", value: {} } },
  });
  assert.equal(admission.outcome.HarmosVariant.case, "Accepted");
  const status = () => api.executionGetExecution(session, { executionId: liveExecutionId });
  const terminal = (status) => {
    return status.summary.sealed;
  };
  let pending = await api.promptsListPending(session, { cycleId });
  while (!pending.prompts.length) {
    assert.ok(!terminal(await status()));
    await nextChange();
    pending = await api.promptsListPending(session, { cycleId });
  }
  assert.equal(pending.prompts.length, 1);
  async function progressAt(completed) {
    const progressStream = await api.observationWatchExecutionProgress(session, {
      testExecutionId: liveExecutionId,
    });
    try {
      for await (const frame of progressStream) {
        if (frame.progress) {
          assert.equal(frame.progress.phase, "Measuring");
          assert.equal(frame.progress.total, 2n);
          assert.equal(frame.progress.completed, BigInt(completed));
          return;
        }
      }
      assert.fail("progress ended before its snapshot");
    } finally {
      await progressStream.cancel();
    }
  }
  await progressAt(0);

  async function records() {
    const result = [];
    let after, through;
    while (true) {
      const page = await api.evidenceListRecords(session, {
        selection: { scope: { cycleId } },
        after,
        through,
      });
      result.push(...page.records);
      if (page.replayComplete) return result;
      after = page.scanned;
      through = page.through;
    }
  }
  async function samples() {
    const result = [];
    const afterStreamCursors = new Map();
    while (true) {
      const page = await api.telemetryListSamples(session, {
        scope: { cycleId },
        streams: [contract.stream],
        limit: 1,
        afterStreamCursors: [...afterStreamCursors].map(([key, value]) => ({ key, value })),
      });
      result.push(...page.samples);
      if (page.complete) return result;
      for (const cursor of page.throughStreamCursors) {
        afterStreamCursors.set(cursor.key, cursor.value);
      }
    }
  }
  function scalarField(value, key, kinds) {
    const object = value.nodes[0].kind.HarmosVariant;
    assert.equal(object.case, "Object");
    const field = object.value.field0.fields.find((field) => field.key === key);
    assert.ok(field, `the value must contain ${key}`);
    const scalar = value.nodes[field.value].kind.HarmosVariant;
    assert.ok(kinds.includes(scalar.case), `the ${key} field must have its declared scalar kind`);
    return scalar.value.field0;
  }
  const reading = (row) => Number(scalarField(row.values, "value", ["Integer", "Unsigned", "Number"]));
  const phase = (record) => scalarField(record.details, "phase", ["Text"]);
  // Prompt admission and evidence visibility have independent frontiers.
  const recordStream = await api.evidenceWatchRecords(session, { selection: { scope: { cycleId } } });
  try {
    for await (const page of recordStream) {
      if (page.records.some((record) => record.recordType === contract.record)) break;
    }
  } finally {
    await recordStream.cancel();
  }
  const sampleStream = await api.telemetryWatchSamples(session, {
    selection: { scope: { executionId: liveExecutionId }, streams: [contract.stream], limit: 1, latest: false },
    follow: true,
  });
  try {
    for await (const page of sampleStream) {
      if (page.samples.length) break;
    }
  } finally {
    await sampleStream.cancel();
  }
  const liveRecords = await records();
  const liveRows = await samples();
  assert.deepEqual(liveRows.map(reading), [1]);
  assert.ok(!terminal(await status()));
  const ready = liveRecords.find((record) => record.recordType === contract.record);
  assert.equal(phase(ready), "ready");
  const filtered = (await records()).filter((record) => record.recordId === ready.recordId);
  assert.equal(filtered.length, 1);

  if (scenario.action === "stop") await api.executionStopExecution(session, { executionId: liveExecutionId });
  else
    await api.promptsRespond(session, {
      promptId: pending.prompts[0].promptId,
      action: scenario.action,
      accepted: true,
    });
  if (scenario.action === "yes") {
    const next = async () =>
      (await api.promptsListPending(session, { cycleId })).prompts.filter(
        (item) => item.promptId !== pending.prompts[0].promptId,
      );
    let nextPrompts = await next();
    while (!nextPrompts.length) {
      assert.ok(!terminal(await status()));
      await nextChange();
      nextPrompts = await next();
    }
    await progressAt(1);
    await api.promptsRespond(session, {
      promptId: nextPrompts[0].promptId,
      action: "yes",
      accepted: true,
    });
  }
  let finished = await status();
  while (!terminal(finished)) {
    await nextChange();
    finished = await status();
  }
  const finalRecords = await records();
  const rows = await samples();
  const summary = await api.executionGetCycle(session, { cycleId });
  const inventory = await api.contentListArtifacts(session, {});
  const artifact = inventory.artifacts.find((item) => item.filename.endsWith("matrix.txt") && item.scope.executionId === liveExecutionId);
  assert.ok(artifact);
  assert.equal(artifact.scope.executionId, liveExecutionId);
  assert.equal(artifact.state.HarmosVariant.case, "Complete");
  const download = await api.contentDownloadArtifact(session, { artifactId: artifact.artifactId });
  const chunks = [];
  let size = 0,
    completed = false;
  try {
    for await (const frame of download) {
      const payload = frame.payload.HarmosVariant;
      if (payload.case === "Chunk") {
        assert.equal(payload.value.field0.offset, BigInt(size));
        const chunk = Buffer.from(payload.value.field0.data);
        chunks.push(chunk);
        size += chunk.length;
      } else if (payload.case === "Complete") {
        assert.equal(payload.value.field0.sizeBytes, BigInt(size));
        assert.equal(
          payload.value.field0.sha256,
          createHash("sha256").update(Buffer.concat(chunks)).digest("hex"),
        );
        completed = true;
      }
    }
  } finally {
    await download.cancel();
  }
  assert.ok(completed);
  const data = Buffer.concat(chunks).toString();
  assert.equal(data, "open\ncleanup\n");
  const authored = finalRecords.filter((record) => record.recordType === contract.record);
  assert.equal(new Set(finalRecords.map((record) => record.recordId)).size, finalRecords.length);
  assert.equal(new Set(rows.map((row) => row.sampleId)).size, rows.length);
  assert.deepEqual(
    finalRecords.map((record) => record.sequence),
    finalRecords.map((record) => record.sequence).sort((a, b) => (a < b ? -1 : a > b ? 1 : 0)),
  );
  assert.equal(finished.summary.executionId, liveExecutionId);
  assert.equal(summary.phase.HarmosVariant.case, "Active");
  for (const item of [...authored, ...rows]) {
    assert.equal(item.scope.cycleId, cycleId);
    assert.equal(item.scope.executionId, liveExecutionId);
    assert.ok(item.timestamp);
  }
  assert.ok(liveRows.every((live) => rows.some((row) => row.sampleId === live.sampleId)));
  process.stdout.write(
    `${JSON.stringify({
      result: {
        status: { Passed: "passed", Failed: "failed", Cancelled: "cancelled" }[summary.result.status.HarmosVariant.case],
        phase: { Passed: "completed", Failed: "failed", Cancelled: "stopped" }[finished.summary.status.HarmosVariant.case],
        values: rows.map(reading),
        records: authored.map(phase),
        cases: Number(summary.caseCount),
        executions: Number(summary.executionCount),
        filtered: filtered.length,
        live: liveRows.length,
        artifact: data,
        progress: { completed: scenario.action === "yes" ? 1 : 0, total: 2 },
      },
    })}\n`,
  );
} finally {
  if (changes) await changes.cancel();
  await harness.close();
}
