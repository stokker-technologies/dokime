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
import { brandFromLogos, selectLogo, type Brand } from "@/lib/brand";
import { loadAppearance } from "@/lib/rpc/appearance";
import { LogoVariant } from "@host/dokime/client/v1/backend_pb.js";
import type { RuntimeThemeFamily } from "@/lib/appearance";
import { applyBrandFavicon } from "@/lib/favicon";
import { materializeThemeFamilyVariables } from "@/lib/theme-family";
import { applyReportAppearance } from "@/report/report-appearance";

/* === Re-Exports === */

/* === Constants === */

const LIGHT = "data:image/svg+xml;base64,PHN2Zy8+";
const DARK = "data:image/png;base64,iVBORw==";
const hostAppearance = vi.hoisted(() => vi.fn());
const fileMetadata = vi.hoisted(() => vi.fn());
const fileRead = vi.hoisted(() => vi.fn());
vi.mock("@/lib/host/client", () => ({ backendAppearance: hostAppearance, filesGet: fileMetadata, filesRead: fileRead }));

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

    const surfaces = mode === "dark" ? ["#111111", "#1a1a1a"] : ["#efecdf", "#d9d4be"];
    for (const surface of surfaces) {
      assert.ok(focusContrast(focus, surface, 1) >= 3);
      assert.ok(focusContrast(focus, surface, 0.5) >= 3);
    }
  }
});

test("Explicit Focus Colors Respect Shared and Mode Theme Precedence", () => {
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
  const appearance = { app_name: "Organization", themes: [], logos: [{ variant: "contained_on_light", url: LIGHT }] };
  applyReportAppearance(appearance);
  assert.equal(head.icon.href, LIGHT);
  assert.equal(head.append.mock.calls.length, 1);

  head.append.mockClear();
  head.querySelectorAll.mockClear();
  applyReportAppearance({ ...appearance, logos: [{ variant: "contained_on_light", url: "https://example.com/logo.svg" }] });
  assert.equal(head.querySelectorAll.mock.calls.length, 1);
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

test("Host Appearance Preserves Declarations Without Inventing Asset Delivery", async () => {
  const session = { generation: "1", isCurrent: () => true };
  const logos = [
    LogoVariant.TRANSPARENT_ON_LIGHT,
    LogoVariant.TRANSPARENT_ON_DARK,
    LogoVariant.CONTAINED_ON_LIGHT,
    LogoVariant.CONTAINED_ON_DARK,
  ].map((variant) => ({ variant }));
  hostAppearance.mockResolvedValue({ appearance: { logos } });
  const resource = await loadAppearance(session);
  assert.equal(resource.appearance.logos.length, 4);
  assert.deepEqual(resource.logos, []);
  assert.equal(typeof resource.unavailableReason, "string");
  resource.dispose();
  resource.dispose();

  // Installed Logo Selection Remains Independent of Connected-Host Delivery.
  const installed = [
    { plugin_id: "fixture", asset_id: "light", variant: "transparent_on_light" as const, url: LIGHT },
    { plugin_id: "fixture", asset_id: "dark", variant: "transparent_on_dark" as const, url: DARK },
  ];
  const brand = brandFromLogos("Fixture", installed);
  assert.equal(selectLogo(brand, "transparent", "light")?.src, LIGHT);
  assert.equal(selectLogo(brand, "transparent", "dark")?.src, DARK);
  hostAppearance.mockResolvedValue({ appearance: { logos: [logos[0], logos[0]] } });
  await assert.rejects(loadAppearance(session));
  hostAppearance.mockResolvedValue({ appearance: { logos: [] } });
  const empty = await loadAppearance(session);
  assert.equal(empty.unavailableReason, undefined);
  empty.dispose();
  await assert.rejects(loadAppearance({ ...session, isCurrent: () => false }), (error: { kind: string }) => error.kind === "session");
});

test("Host Appearance Reads Verified Product Files and Releases Object URLs", async () => {
  const session = { generation: "asset", isCurrent: () => true };
  const bytes = new TextEncoder().encode('<svg xmlns="http://www.w3.org/2000/svg"/>');
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
  const file = { length: BigInt(bytes.length), sha256: digest, metadata: { id: "published-logo" } };
  const at = { positions: [{ history: { store: "assets", epoch: "files" }, sequence: 1n }] };
  hostAppearance.mockResolvedValue({
    appearance: {
      logos: [
        {
          variant: LogoVariant.TRANSPARENT_ON_LIGHT,
          asset: { reference: { pluginId: "fixture", assetId: "logo" }, file, contentType: "image/svg+xml" },
        },
      ],
    },
  });
  fileMetadata.mockResolvedValue({ file, chunkBytes: BigInt(bytes.length), messageBytes: 1024n, resolvedAt: at });
  fileRead.mockResolvedValue({ data: bytes, totalLength: BigInt(bytes.length), sha256: digest, resolvedAt: at });
  const revoke = vi.spyOn(URL, "revokeObjectURL");
  const resource = await loadAppearance(session);
  assert.equal(resource.logos[0].asset_id, "logo");
  assert.ok(resource.logos[0].url.startsWith("blob:"));
  assert.equal(resource.unavailableReason, undefined);
  resource.dispose();
  resource.dispose();
  assert.deepEqual(revoke.mock.calls, [[resource.logos[0].url]]);
  revoke.mockRestore();
});
