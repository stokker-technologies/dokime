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
import test from "node:test";

import { toGeneratedKey, withLf } from "../../../../../../tools/node/gui/generate-rpc.mjs";

/* === Tests === */

test("Windows Paths", () => {
  // `path.relative` on win32 joins segments with '\\'; the obsolete-file scan
  // must fold that back to '/' to match the POSIX-style keys in `outputs`,
  // or a current file such as `src\lib\rpc\generated\client.ts` is
  // misclassified as obsolete (the reported Windows CI failure).
  assert.equal(
    toGeneratedKey("src\\lib\\rpc\\generated\\client.ts"),
    "src/lib/rpc/generated/client.ts",
  );
});

test("POSIX Paths", () => {
  assert.equal(
    toGeneratedKey("src/lib/rpc/generated/client.ts"),
    "src/lib/rpc/generated/client.ts",
  );
});

test("CRLF Normalization", () => {
  assert.equal(
    withLf("export const a = 1;\r\nexport const b = 2;\r\n"),
    "export const a = 1;\nexport const b = 2;\n",
  );
});

test("LF Identity", () => {
  const content = "export const a = 1;\nexport const b = 2;\n";
  assert.equal(withLf(content), content);
});

test("LF Comparison", () => {
  const generated = 'export const descriptorSha256 = "abc";\n';
  const checkedOutOnWindows = generated.replace(/\n/g, "\r\n");
  assert.equal(withLf(checkedOutOnWindows), withLf(generated));
});
