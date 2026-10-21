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
import { Dialog, DialogFooter, dialogContentClass } from "@/components/ui/dialog";
import { AccessKeyPanel } from "@/features/settings/sections/AccessKeyDialog";
import { cn } from "@/lib/utils";

/* === Re-Exports === */

/* === Constants === */

const KEY = "0123456789abcdef".repeat(4);

/* === Types === */

/* === Classes === */

/* === Tests === */

test("access key dialog shows the whole key and every action without fixed widths", () => {
  const html = panel(() => undefined);
  assert.equal(KEY.length, 64);
  assert.match(html, new RegExp(`<textarea[^>]*>${KEY}</textarea>`));
  for (const label of ["Copy Key", "Save Key File…", "Save Public Certificate…", "I Have Saved The Key"]) {
    assert.ok(html.includes(`>${label}</button>`), label);
  }
  assert.doesNotMatch(html, /<button[^>]*disabled=""[^>]*>Save Public Certificate/);
  assert.doesNotMatch(html, /(?<![\w-])(?:min-)?w-\[|width:/);
  const input = html.match(/<textarea[^>]*aria-label="Generated Access Key"[^>]*>/)?.[0] ?? "";
  assert.match(input, /\bmin-w-0\b/);
  assert.match(input, /\bbreak-all\b/);
  assert.match(input, /readOnly=""/);
});

test("access key dialog hides the certificate action when the host has no certificate", () => {
  const html = panel(undefined);
  assert.ok(!html.includes("Save Public Certificate"));
  assert.ok(html.includes("I Have Saved The Key"));
});

test("dialog content keeps one shrinkable track so wide children cannot widen it", () => {
  assert.match(dialogContentClass, /\bgrid-cols-\[minmax\(0,1fr\)\]/);
  assert.match(dialogContentClass, /\bw-full\b/);
  assert.match(dialogContentClass, /\bmax-w-\[calc\(100%-2rem\)\]/);
});

test("a dialog's own breakpoint width replaces the default one", () => {
  for (const width of ["sm:max-w-3xl", "sm:max-w-5xl"]) {
    const merged = cn(dialogContentClass, width).split(" ");
    assert.ok(merged.includes(width), width);
    assert.ok(!merged.includes("sm:max-w-sm"), width);
  }
  assert.ok(cn(dialogContentClass, "max-w-5xl").split(" ").includes("sm:max-w-sm"), "an unprefixed width loses from sm up");
});

test("dialog footers wrap their actions instead of overflowing", () => {
  const html = renderToStaticMarkup(createElement(DialogFooter, null, "actions"));
  assert.match(html, /\bsm:flex-wrap\b/);
  assert.match(html, /\bmin-w-0\b/);
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
