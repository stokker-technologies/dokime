/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test } from "vitest";

// Local Imports
import { Dialog } from "@/components/ui/dialog";
import { AccessKeyPanel } from "@/features/settings/sections/AccessKeyDialog";

/* === Re-Exports === */

/* === Constants === */

const KEY = "0123456789abcdef".repeat(4);

/* === Types === */

/* === Classes === */

/* === Tests === */

test("key disclosure", () => {
  const html = panel(() => undefined);
  assert.equal(KEY.length, 64);
  assert.match(html, new RegExp(`<textarea[^>]*>${KEY}</textarea>`));
  assert.equal(html.match(/<button\b/g)?.length, 4);
  assert.doesNotMatch(html, /<button[^>]*disabled=""/);
  const input = html.match(/<textarea[^>]*>/)?.[0] ?? "";
  assert.match(input, /readOnly=""/);
});

test("optional certificate", () => {
  const html = panel(undefined);
  assert.equal(html.match(/<button\b/g)?.length, 3);
});

/* === Private Functions === */

function panel(saveCertificate: (() => void) | undefined): string {
  return renderToStaticMarkup(
    createElement(
      Dialog,
      { open: true },
      createElement(AccessKeyPanel, {
        value: KEY,
        busy: false,
        saved: false,
        saveError: null,
        close: () => undefined,
        save: () => undefined,
        saveCertificate,
      }),
    ),
  );
}
