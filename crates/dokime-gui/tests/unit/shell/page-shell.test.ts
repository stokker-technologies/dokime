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
import { type ComponentProps, createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { test } from "vitest";

// Local Imports
import { PageBleed, PageNotice, PageShell, PageToolbar, pageTitle } from "@/shell/PageShell";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Tests === */

test("content pages share one gutter, section gap and breadcrumb", () => {
  assert.equal(pageTitle(["History", "Compare"]), "History › Compare");
  assert.equal(pageTitle(["Library"]), "Library");
  const html = renderToStaticMarkup(
    createElement(
      PageShell,
      { trail: ["Cycles"] } as ComponentProps<typeof PageShell>,
      createElement(PageToolbar, null, "filters"),
      createElement(PageNotice, { tone: "alert" } as ComponentProps<typeof PageNotice>, "failed"),
      createElement(PageBleed, null, "table"),
    ),
  );
  assert.match(
    html,
    /^<div class="[^"]*\bp-\[var\(--shell-inset\)\][^"]*\bgap-\[var\(--section-gap\)\]|^<div class="[^"]*\bgap-\[var\(--section-gap\)\][^"]*\bp-\[var\(--shell-inset\)\]/,
  );
  assert.match(html, /py-\[var\(--bar-pad-y\)\]/);
  assert.match(html, /role="alert"/);
  assert.match(html, /-mx-\[var\(--shell-inset\)\]/);
});
