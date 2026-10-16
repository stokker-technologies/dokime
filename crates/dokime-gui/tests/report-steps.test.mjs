/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import assert from "node:assert/strict";
import { test } from "node:test";
import { reportFixturePayload } from "../src/report/report-fixture.ts";

/* === Tests === */

test("a written report carries its steps under their own execution", () => {
  const payload = reportFixturePayload();

  assert.equal("test_steps" in payload, false);
  for (const execution of payload.test_executions) {
    assert.deepEqual(
      execution.steps.map((row) => row.record_type),
      ["step", "criterion"],
    );
    assert.ok(
      execution.steps.every((row) => row.test_execution_id === execution.test_execution_id),
    );
  }
});

test("a written report carries each artifact complete with its download target", () => {
  const payload = reportFixturePayload();
  const [artifact] = payload.snapshot.artifacts;

  assert.equal("artifact_links" in payload, false);
  assert.equal(artifact.downloadHref, artifact.href);
  assert.equal(artifact.group_label, "Execution Artifacts");
});
