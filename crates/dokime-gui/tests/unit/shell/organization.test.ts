/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Organization Appearance Tokens And Favicon Selection
 */

/* === Imports === */

// Standard Library Imports
import assert from "node:assert/strict";

// Third Party Imports
import { afterEach, test, vi } from "vitest";

// Local Imports
import type { Brand } from "@/lib/brand";
import type { RuntimeThemeFamily } from "@/lib/appearance";
import { applyBrandFavicon } from "@/lib/favicon";
import { materializeThemeFamilyVariables } from "@/lib/theme-family";
import { applyReportAppearance } from "@/report/report-appearance";

/* === Re-Exports === */

/* === Constants === */

const LIGHT = "data:image/svg+xml;base64,PHN2Zy8+";
const DARK = "data:image/png;base64,iVBORw==";

/* === Types === */

/* === Classes === */

/* === Public Functions === */

afterEach(() => vi.unstubAllGlobals());

test.each(["light", "dark"] as const)("Red Organization Palettes Keep %s Focus Distinct From Errors", (mode) => {
  for (const destructive of ["#d2051e", "#b42318"]) {
    const palette = { primary: "#d2051e", destructive, ring: "var(--primary)" };
    const theme: RuntimeThemeFamily = {
      plugin_id: "organization",
      theme_id: "red-brand",
      label: "Red Brand",
      light: { variables: palette },
      dark: { variables: palette },
    };
    const variables = materializeThemeFamilyVariables(theme, mode);
    assert.ok(variables);
    const focus = variables["focus-ring"];
    assert.notEqual(focus, palette.primary);
    assert.notEqual(focus, palette.destructive);
    assert.notEqual(focus, palette.ring);
    assert.equal(variables.destructive, destructive);
    assert.equal(variables.primary, palette.primary);

    const surfaces = mode === "dark" ? ["#1b1915", "#24211b"] : ["#ffffff", "#f2ebdd", "#fbf8f0"];
    for (const surface of surfaces) {
      assert.ok(focusContrast(focus, surface, 1) >= 3);
      assert.ok(focusContrast(focus, surface, 0.5) >= 3);
    }
  }
});

test("Explicit Focus Colors Respect Shared And Mode Theme Precedence", () => {
  const theme: RuntimeThemeFamily = {
    plugin_id: "organization",
    theme_id: "custom-focus",
    label: "Custom Focus",
    variables: { "focus-ring": "#005577", destructive: "#d2051e" },
    light: {},
    dark: { variables: { "focus-ring": "#88ddff" } },
  };
  assert.equal(materializeThemeFamilyVariables(theme, "light")?.["focus-ring"], "#005577");
  assert.equal(materializeThemeFamilyVariables(theme, "dark")?.["focus-ring"], "#88ddff");
});

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

function focusContrast(foreground: string, background: string, opacity: number): number {
  const channels = (hex: string) => [1, 3, 5].map((offset) => Number.parseInt(hex.slice(offset, offset + 2), 16) / 255);
  const surface = channels(background);
  const ring = channels(foreground).map((channel, index) => channel * opacity + surface[index] * (1 - opacity));
  const luminance = (rgb: number[]) => {
    const linear = rgb.map((channel) => (channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4));
    return linear[0] * 0.2126 + linear[1] * 0.7152 + linear[2] * 0.0722;
  };
  const ringLuminance = luminance(ring);
  const surfaceLuminance = luminance(surface);
  return (Math.max(ringLuminance, surfaceLuminance) + 0.05) / (Math.min(ringLuminance, surfaceLuminance) + 0.05);
}

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
