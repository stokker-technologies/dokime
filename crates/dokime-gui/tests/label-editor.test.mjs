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
  CLOSED_LABEL_DRAFT,
  EMPTY_LABEL_ERROR,
  labelEditorKey,
  openLabelEdit,
  refuseLabel,
  updateLabelText,
} = await import(pathToFileURL(resolve(gui, "src/features/cycle/label-editor-model.ts")).href);

/* === Tests === */

test("the pencil opens a draft holding the current label", () => {
  assert.deepEqual(openLabelEdit("Charger #2"), {
    mode: "editing",
    original: "Charger #2",
    text: "Charger #2",
    error: null,
  });
});

test("Enter saves the trimmed label", () => {
  const draft = updateLabelText(openLabelEdit("Charger #2"), "  Charger Burn-In  ");
  assert.deepEqual(labelEditorKey(draft, "Enter"), { kind: "save", label: "Charger Burn-In" });
});

test("Escape cancels whatever was typed", () => {
  const draft = updateLabelText(openLabelEdit("Charger #2"), "Charger Burn-In");
  assert.deepEqual(labelEditorKey(draft, "Escape"), { kind: "cancel" });
});

test("Enter on an unchanged label closes the draft without saving", () => {
  const draft = updateLabelText(openLabelEdit("Charger #2"), " Charger #2 ");
  assert.deepEqual(labelEditorKey(draft, "Enter"), { kind: "cancel" });
});

test("Enter on a blank label is refused in place", () => {
  const draft = updateLabelText(openLabelEdit("Charger #2"), "   ");
  assert.deepEqual(labelEditorKey(draft, "Enter"), { kind: "refuse", error: EMPTY_LABEL_ERROR });
});

test("other keys type, and a closed editor ignores every key", () => {
  assert.deepEqual(labelEditorKey(openLabelEdit("Charger #2"), "a"), { kind: "none" });
  assert.deepEqual(labelEditorKey(CLOSED_LABEL_DRAFT, "Enter"), { kind: "none" });
});

test("a refusal stays beside the draft until the operator types again", () => {
  const refused = refuseLabel(openLabelEdit("Charger #2"), "label must not be empty");
  assert.equal(refused.error, "label must not be empty");
  assert.equal(refused.text, "Charger #2");
  assert.equal(updateLabelText(refused, "Charger").error, null);
  assert.equal(refuseLabel(CLOSED_LABEL_DRAFT, "ignored"), CLOSED_LABEL_DRAFT);
});
