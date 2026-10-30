/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Organization Favicon Selection And Replacement
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { afterEach, test, vi } from "vitest";

// Local Imports
import type { Brand } from "@/lib/brand";
import { applyBrandFavicon } from "@/lib/favicon";
import { applyReportAppearance } from "@/report/report-appearance";

/* === Re-Exports === */

/* === Constants === */

const LIGHT = "data:image/svg+xml;base64,PHN2Zy8+";
const DARK = "data:image/png;base64,iVBORw==";

/* === Types === */

/* === Classes === */

/* === Public Functions === */

afterEach(() => vi.unstubAllGlobals());

test("Brand Favicon Replaces All Existing Tab Icons", () => {
  const head = documentHead();
  const brand: Brand = {
    label: "Organization",
    logos: {
      contained_on_light: { src: LIGHT },
      transparent_on_light: { src: DARK },
    },
  };

  applyBrandFavicon(brand);
  assert.equal(head.icon.href, LIGHT);
  assert.equal(head.icon.rel, "icon");
  for (const icon of head.existing) assert.equal(icon.remove.mock.calls.length, 1);
  assert.deepEqual(head.append.mock.calls, [[head.icon]]);
  assert.deepEqual(head.querySelectorAll.mock.calls, [['link[rel~="icon"]']]);
});

test("Brand Favicon Follows The Viewer Color Scheme", () => {
  const head = documentHead(true);
  applyBrandFavicon({ label: "Organization", logos: { contained_on_light: { src: LIGHT }, contained_on_dark: { src: DARK } } });
  assert.equal(head.icon.href, DARK);
});

test("Brand Favicon Falls Back To Transparent Or The Other Surface", () => {
  const head = documentHead(true);
  applyBrandFavicon({ label: "Organization", logos: { transparent_on_dark: { src: DARK }, contained_on_light: { src: LIGHT } } });
  assert.equal(head.icon.href, DARK);

  applyBrandFavicon({ label: "Organization", logos: { transparent_on_light: { src: LIGHT } } });
  assert.equal(head.icon.href, LIGHT);
});

test("Missing Brand Logos Preserve The Default Favicon", () => {
  const head = documentHead();
  applyBrandFavicon({ label: "Organization", logos: {} });
  applyReportAppearance(null);
  assert.equal(head.querySelectorAll.mock.calls.length, 0);
  assert.equal(head.append.mock.calls.length, 0);
  for (const icon of head.existing) assert.equal(icon.remove.mock.calls.length, 0);
});

test("Report Favicon Uses Only Captured Image Data", () => {
  const head = documentHead();
  const appearance = { app_name: "Organization", themes: [], logos: [{ variant: "contained_on_light", url: LIGHT }] };
  applyReportAppearance(appearance);
  assert.equal(head.icon.href, LIGHT);
  assert.equal(head.append.mock.calls.length, 1);

  head.append.mockClear();
  head.querySelectorAll.mockClear();
  applyReportAppearance({ ...appearance, logos: [{ variant: "contained_on_light", url: "https://example.com/logo.svg" }] });
  assert.equal(head.querySelectorAll.mock.calls.length, 0);
  assert.equal(head.append.mock.calls.length, 0);
});

/* === Private Functions === */

/** Supply the document head boundary used by the favicon without a browser runtime. */
function documentHead(dark = false) {
  const existing = [{ remove: vi.fn() }, { remove: vi.fn() }];
  const icon = { href: "", rel: "" };
  const append = vi.fn();
  const querySelectorAll = vi.fn(() => existing);
  vi.stubGlobal("document", { head: { querySelectorAll, append }, createElement: () => icon });
  vi.stubGlobal("window", { matchMedia: () => ({ matches: dark }) });
  return { existing, icon, append, querySelectorAll };
}

/* === Entry Point === */
