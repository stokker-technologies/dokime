// @vitest-environment happy-dom
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
import { onTestFinished, test, vi } from "vitest";

// Local Imports
import { PlotCardDocument } from "./document-fixture";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

/* === Private Functions === */

/* === Entry Point === */

test("Desktop and Frozen Documents Share Section Headings and Single Plot Cards", () => {
  vi.stubGlobal("IntersectionObserver", undefined);
  onTestFinished(() => {
    vi.unstubAllGlobals();
  });
  const host = document.createElement("div");
  for (const frozen of [false, true]) {
    host.innerHTML = renderToStaticMarkup(createElement(PlotCardDocument, { frozen }));
    assert.equal(host.querySelectorAll("h4.document-section-title").length, 2);
    assert.equal(host.querySelectorAll("article.plot-card.border").length, 2);
    assert.equal(host.querySelectorAll('table[aria-label="Trace Statistics"]').length, 2);
    assert.equal(host.querySelectorAll('[data-slot="data-table"], input, select, tfoot').length, 0);
    for (const card of host.querySelectorAll("article.plot-card")) {
      assert.equal(card.querySelectorAll("article, [data-slot=data-table]").length, 0);
      assert.equal(card.children[2].tagName, "P");
      assert.ok(card.children[3].querySelector("table"));
      const headers = [...card.querySelectorAll("thead th")].map((cell) => cell.textContent);
      const values = [...card.querySelectorAll("tbody td")].map((cell) => cell.textContent);
      assert.equal(
        values[headers.findIndex((header) => header?.startsWith("Median"))],
        card.textContent?.includes("Voltage") ? "5.50" : "6.50",
      );
      assert.equal(values[headers.findIndex((header) => header?.startsWith("Std"))], "3.45");
      assert.equal(card.querySelectorAll('[data-testid="plot-settings"]').length, frozen ? 0 : 1);
    }
  }
});
