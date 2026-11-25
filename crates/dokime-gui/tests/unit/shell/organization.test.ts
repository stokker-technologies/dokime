/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Organization Appearance And Favicon Selection
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { afterEach, test, vi } from "vitest";

// Local Imports
import { type AppearanceSnapshot, brandFromAppearance, selectLogo, type Brand } from "@/lib/brand";
import { loadAppearance } from "@/lib/rpc/appearance";
import { LogoVariant } from "@generated/host/model";
import { applyBrandFavicon } from "@/lib/favicon";
import { applyReportAppearance } from "@/report/report-appearance";

/* === Re-Exports === */

/* === Constants === */

const LIGHT = "data:image/svg+xml;base64,PHN2Zy8+";
const DARK = "data:image/png;base64,iVBORw==";
const hostAppearance = vi.hoisted(() => vi.fn());
vi.mock("@/lib/host/client", () => ({ backendAppearance: hostAppearance }));

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

test("Brand Favicon Follows the Viewer Color Scheme", () => {
  const head = documentHead(true);
  applyBrandFavicon({ label: "Organization", logos: { contained_on_light: { src: LIGHT }, contained_on_dark: { src: DARK } } });
  assert.equal(head.icon.href, DARK);
});

test("Brand Favicon Falls Back to Transparent or the Other Surface", () => {
  const head = documentHead(true);
  applyBrandFavicon({ label: "Organization", logos: { transparent_on_dark: { src: DARK }, contained_on_light: { src: LIGHT } } });
  assert.equal(head.icon.href, DARK);

  applyBrandFavicon({ label: "Organization", logos: { transparent_on_light: { src: LIGHT } } });
  assert.equal(head.icon.href, LIGHT);
});

test("Missing Brand Logos Remove Previous Artwork", () => {
  const head = documentHead();
  applyBrandFavicon({ label: "Organization", logos: {} });
  applyReportAppearance(null);
  assert.equal(head.querySelectorAll.mock.calls.length, 1);
  assert.equal(head.append.mock.calls.length, 0);
  for (const icon of head.existing) assert.equal(icon.remove.mock.calls.length, 1);
});

test("Report Favicon Uses Only Captured Image Data", () => {
  const head = documentHead();
  const logo = { variant: LogoVariant.CONTAINED_ON_LIGHT, dataUri: LIGHT };
  const appearance: AppearanceSnapshot = { appName: "Organization", themes: [], logos: [logo] };
  applyReportAppearance(appearance);
  assert.equal(head.icon.href, LIGHT);
  assert.equal(head.append.mock.calls.length, 1);

  head.append.mockClear();
  head.querySelectorAll.mockClear();
  applyReportAppearance({ ...appearance, logos: [{ ...logo, dataUri: "https://example.com/logo.svg" }] });
  assert.equal(head.querySelectorAll.mock.calls.length, 1);
  assert.equal(head.append.mock.calls.length, 0);
});

test("Host Appearance Carries Inline Logos", async () => {
  const session = { generation: "1", isCurrent: () => true };
  const logos = [
    { variant: LogoVariant.TRANSPARENT_ON_LIGHT, dataUri: LIGHT },
    { variant: LogoVariant.TRANSPARENT_ON_DARK, dataUri: DARK },
  ];
  hostAppearance.mockResolvedValue({ appearance: { appName: "Fixture", logos, themes: [] } });
  const brand = brandFromAppearance(await loadAppearance(session));
  assert.equal(selectLogo(brand, "transparent", "light")?.src, LIGHT);
  assert.equal(selectLogo(brand, "transparent", "dark")?.src, DARK);
  await assert.rejects(loadAppearance({ ...session, isCurrent: () => false }), (error: { kind: string }) => error.kind === "session");
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
