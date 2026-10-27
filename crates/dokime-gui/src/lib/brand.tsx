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

import type { BootOrganization, BootOrganizationPack } from "@/lib/desktop/organization";
import type { RuntimeAppLogo } from "@/lib/types/view-contracts";
import defaultBrandData from "../generated/organization/default-organization.json";

/* === Re-Exports === */

/* === Constants === */

const ORGANIZATION_PACK_STYLE_ID = "dokime-organization-pack";

/**
 * What the desktop shell resolved before this page loaded: an operator's
 * organization pack, or why it could not apply. Absent in a browser, a report and
 * in tests, where the compiled brand stands.
 */
export const bootOrganization: BootOrganization = startBootOrganization();

/** The operator's organization pack this process applies, above every other brand source. */
export const organizationPack: BootOrganizationPack | null = bootOrganization.pack;

/**
 * Default brand: the organization pack when one applies, otherwise the compiled
 * brand. The shell never hardcodes a customer name; it reads the brand from
 * context.
 */
const generatedDefaultBrand = defaultBrandData as DefaultBrandManifest;

export const defaultBrand: Brand = organizationPack
  ? { label: organizationPack.label, logos: packLogos(organizationPack) }
  : { label: generatedDefaultBrand.label, logos: generatedDefaultBrand.logos };

/** The window title template of the brand that applies; `{app}` is the label. */
export const defaultTitleTemplate: string = organizationPack?.titleTemplate ?? "{app}";

const BrandContext = createContext<Brand>(defaultBrand);

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

/* === Components === */

export function BrandProvider({ brand = defaultBrand, children }: { brand?: Brand; children: ReactNode }) {
  return <BrandContext.Provider value={brand}>{children}</BrandContext.Provider>;
}

/* === Hooks === */

export function useBrand(): Brand {
  return useContext(BrandContext);
}

/* === Public Functions === */

export function brandFromLogos(label: string, logos: RuntimeAppLogo[]): Brand {
  return { label, logos: Object.fromEntries(logos.map((logo) => [logo.variant, { src: logo.url, alt: logo.alt ?? label }])) };
}

export function selectLogo(brand: Brand, treatment: "transparent" | "contained", surface: "light" | "dark"): BrandLogo | undefined {
  const alternative = treatment === "transparent" ? "contained" : "transparent";
  return brand.logos[`${treatment}_on_${surface}`] ?? brand.logos[`${alternative}_on_${surface}`];
}

/** Logo variants of a pack in the frontend's shape. */
export function packLogos(pack: BootOrganizationPack): Brand["logos"] {
  return Object.fromEntries(Object.entries(pack.logos).map(([variant, src]) => [variant, { src, alt: pack.label }]));
}

/* === Private Functions === */

/** Reads the shell's boot organization once and applies the pack theme before the first render. */
function startBootOrganization(): BootOrganization {
  const value = (globalThis as { __DOKIME_ORGANIZATION_PACK__?: Partial<BootOrganization> }).__DOKIME_ORGANIZATION_PACK__;
  const organization = { pack: value?.pack ?? null, error: value?.error ?? null };
  applyOrganizationPackStylesheet(organization.pack);
  return organization;
}

// The Pack Theme Follows The Compiled Stylesheets, So Its Rules Win On Order.
function applyOrganizationPackStylesheet(pack: BootOrganizationPack | null): void {
  if (!pack?.css || typeof document === "undefined") return;
  const style = document.createElement("style");
  style.id = ORGANIZATION_PACK_STYLE_ID;
  style.textContent = pack.css;
  document.head.append(style);
}
