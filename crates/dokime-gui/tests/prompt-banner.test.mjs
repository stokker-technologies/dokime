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
import { dirname, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

/* === Constants === */

const gui = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const {
  answerablePrompts,
  highestPromptLevel,
  orderPendingPrompts,
  pendingPromptCountLabel,
  promptLevelTone,
} = await import(pathToFileURL(resolve(gui, "src/features/cycle/prompt-banner-model.ts")).href);

/* === Fixture === */

function prompt(id, level, createdAt) {
  return { correlation_id: id, created_at: createdAt, level };
}

/* === Tests === */

test("the banner puts the most urgent level first and the oldest first within it", () => {
  const ordered = orderPendingPrompts([
    prompt("info-old", "info", "2026-10-13T08:00:00Z"),
    prompt("warning-new", "warning", "2026-10-13T08:05:00Z"),
    prompt("error", "error", "2026-10-13T08:10:00Z"),
    prompt("warning-old", "warning", "2026-10-13T08:01:00Z"),
    prompt("debug", "debug", "2026-10-13T07:00:00Z"),
  ]);

  assert.deepEqual(
    ordered.map((item) => item.correlation_id),
    ["error", "warning-old", "warning-new", "info-old", "debug"],
  );
});

test("prompts asked in the same instant keep a stable order", () => {
  const at = "2026-10-13T08:00:00Z";
  const ordered = orderPendingPrompts([prompt("b", "info", at), prompt("a", "info", at)]);

  assert.deepEqual(
    ordered.map((item) => item.correlation_id),
    ["a", "b"],
  );
});

test("an unreadable timestamp sorts last within its level", () => {
  const ordered = orderPendingPrompts([
    prompt("broken", "warning", "not a time"),
    prompt("dated", "warning", "2026-10-13T08:00:00Z"),
  ]);

  assert.deepEqual(
    ordered.map((item) => item.correlation_id),
    ["dated", "broken"],
  );
});

test("ordering leaves the pending list it was given untouched", () => {
  const pending = [
    prompt("info", "info", "2026-10-13T08:00:00Z"),
    prompt("error", "error", "2026-10-13T08:00:00Z"),
  ];
  orderPendingPrompts(pending);

  assert.deepEqual(
    pending.map((item) => item.correlation_id),
    ["info", "error"],
  );
});

test("a count is coloured by its most urgent prompt and reads as a phrase", () => {
  assert.equal(highestPromptLevel([]), null);
  assert.equal(
    highestPromptLevel([
      prompt("info", "info", "2026-10-13T08:00:00Z"),
      prompt("warning", "warning", "2026-10-13T08:00:00Z"),
    ]),
    "warning",
  );
  assert.equal(pendingPromptCountLabel(1), "1 prompt waiting");
  assert.equal(pendingPromptCountLabel(3), "3 prompts waiting");
});

test("the tone follows the record severity scale with Info kept visible", () => {
  assert.deepEqual(["error", "warning", "info", "debug"].map(promptLevelTone), [
    "danger",
    "warning",
    "info",
    "muted",
  ]);
});

test("a prompt is answerable only while its owner runs", () => {
  const scoped = (id, executionId) => ({
    correlation_id: id,
    scope: { test_execution_id: executionId },
  });
  const pending = [
    scoped("running", "one"),
    scoped("interrupted", "two"),
    { correlation_id: "cycle" },
  ];
  const owners = (cycleEnded) => ({
    cycleEnded,
    executionActive: (executionId) => executionId === "one",
  });

  assert.deepEqual(
    answerablePrompts(pending, owners(false)).map((item) => item.correlation_id),
    ["running", "cycle"],
  );
  assert.deepEqual(
    answerablePrompts(pending, owners(true)).map((item) => item.correlation_id),
    ["running"],
  );
});
