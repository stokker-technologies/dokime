/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { createContext, type ReactNode, useContext } from "react";

/* === Local Imports === */

import type { BootBranding, BootBrandPack } from "@/lib/desktop/branding";
import type { RuntimeAppLogo } from "@/lib/types/view-contracts";
import defaultBrandData from "../../../../branding/generated/default-brand.json";

/* === Types === */

export type LogoVariant = RuntimeAppLogo["variant"];
export type BrandLogo = { src: string; alt?: string };

export type Brand = {
  label: string;
  logos: Partial<Record<LogoVariant, BrandLogo>>;
};

type DefaultBrandManifest = {
  label: string;
  title: string;
  logos: Partial<Record<LogoVariant, BrandLogo>>;
};

/* === Constants === */

const BRAND_PACK_STYLE_ID = "dokime-brand-pack";

/**
 * What the desktop shell resolved before this page loaded: an operator's
 * branding pack, or why it could not apply. Absent in a browser, a report and
 * in tests, where the compiled brand stands.
 */
export const bootBranding: BootBranding = startBootBranding();

/** The operator's branding pack this process applies, above every other brand source. */
export const brandPack: BootBrandPack | null = bootBranding.pack;

/**
 * Default brand: the branding pack when one applies, otherwise the compiled
 * brand. The shell never hardcodes a customer name; it reads the brand from
 * context.
 */
const generatedDefaultBrand = defaultBrandData as DefaultBrandManifest;

export const defaultBrand: Brand = brandPack
  ? { label: brandPack.label, logos: packLogos(brandPack) }
  : { label: generatedDefaultBrand.label, logos: generatedDefaultBrand.logos };

/** The window title template of the brand that applies; `{app}` is the label. */
export const defaultTitleTemplate: string = brandPack?.titleTemplate ?? "{app}";

const BrandContext = createContext<Brand>(defaultBrand);

/* === Components === */

export function BrandProvider({ brand = defaultBrand, children }: { brand?: Brand; children: ReactNode }) {
  return <BrandContext.Provider value={brand}>{children}</BrandContext.Provider>;
}

/* === Public Functions === */

export function useBrand(): Brand {
  return useContext(BrandContext);
}

export function brandFromLogos(label: string, logos: RuntimeAppLogo[]): Brand {
  return { label, logos: Object.fromEntries(logos.map((logo) => [logo.variant, { src: logo.url, alt: logo.alt ?? label }])) };
}

export function selectLogo(brand: Brand, treatment: "transparent" | "contained", surface: "light" | "dark"): BrandLogo | undefined {
  const alternative = treatment === "transparent" ? "contained" : "transparent";
  return brand.logos[`${treatment}_on_${surface}`] ?? brand.logos[`${alternative}_on_${surface}`];
}

/** Logo variants of a pack in the frontend's shape. */
export function packLogos(pack: BootBrandPack): Brand["logos"] {
  return Object.fromEntries(Object.entries(pack.logos).map(([variant, src]) => [variant, { src, alt: pack.label }]));
}

/* === Private Functions === */

/** Reads the shell's boot branding once and applies the pack theme before the first render. */
function startBootBranding(): BootBranding {
  const value = (globalThis as { __DOKIME_BRAND_PACK__?: Partial<BootBranding> }).__DOKIME_BRAND_PACK__;
  const branding = { pack: value?.pack ?? null, error: value?.error ?? null };
  applyBrandPackStylesheet(branding.pack);
  return branding;
}

// The Pack Theme Follows The Compiled Stylesheets, So Its Rules Win On Order.
function applyBrandPackStylesheet(pack: BootBrandPack | null): void {
  if (!pack?.css || typeof document === "undefined") return;
  const style = document.createElement("style");
  style.id = BRAND_PACK_STYLE_ID;
  style.textContent = pack.css;
  document.head.append(style);
}
