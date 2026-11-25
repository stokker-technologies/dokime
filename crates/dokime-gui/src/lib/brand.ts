/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { Wire } from "@/lib/rpc/canonical";
import { type BackendAppearanceResponse, LogoVariant as HostLogoVariant } from "@generated/host/model";
import type { BootOrganization, BootOrganizationPack } from "./contracts/organization";
import defaultBrandData from "../generated/organization/default-organization.json";

/* === Re-Exports === */

/* === Constants === */

const ORGANIZATION_PACK_STYLE_ID = "dokime-organization-pack";
const HOST_LOGO_VARIANTS: Partial<Record<HostLogoVariant, LogoVariant>> = {
  [HostLogoVariant.TRANSPARENT_ON_LIGHT]: "transparent_on_light",
  [HostLogoVariant.TRANSPARENT_ON_DARK]: "transparent_on_dark",
  [HostLogoVariant.CONTAINED_ON_LIGHT]: "contained_on_light",
  [HostLogoVariant.CONTAINED_ON_DARK]: "contained_on_dark",
};

/**
 * What the desktop shell resolved before this page loaded: the installation's
 * organization pack, or why it could not apply. Absent in a browser, a report and
 * in tests, where the compiled brand stands.
 */
export const bootOrganization: BootOrganization = startBootOrganization();

/** The installation's organization pack this process applies, above every other brand source. */
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

/* === Types === */

/** The appearance a host reads from its installed organization pack. */
export type AppearanceSnapshot = Wire<NonNullable<BackendAppearanceResponse["appearance"]>>;
export type LogoVariant = "transparent_on_light" | "transparent_on_dark" | "contained_on_light" | "contained_on_dark";
export type BrandLogo = { src: string; alt?: string };

export type Brand = {
  label: string;
  logos: Partial<Record<LogoVariant, BrandLogo>>;
};

type DefaultBrandManifest = {
  label: string;
  logos: Partial<Record<LogoVariant, BrandLogo>>;
};

/* === Public Functions === */

/** The brand a host appearance declares; `accepts` keeps only the logo data a surface may load. */
export function brandFromAppearance(appearance: AppearanceSnapshot, accepts: (uri: string) => boolean = () => true): Brand {
  const label = appearance.appName.trim() || defaultBrand.label;
  const logos: Brand["logos"] = {};
  for (const logo of appearance.logos) {
    const variant = HOST_LOGO_VARIANTS[logo.variant];
    if (variant && accepts(logo.dataUri)) logos[variant] = { src: logo.dataUri };
  }
  return { label, logos };
}

export function selectLogo(brand: Brand, treatment: "transparent" | "contained", surface: "light" | "dark"): BrandLogo | undefined {
  const alternative = treatment === "transparent" ? "contained" : "transparent";
  return brand.logos[`${treatment}_on_${surface}`] ?? brand.logos[`${alternative}_on_${surface}`];
}

/* === Private Functions === */

/** Logo variants of a pack in the frontend's shape. */
function packLogos(pack: BootOrganizationPack): Brand["logos"] {
  return Object.fromEntries(Object.entries(pack.logos).map(([variant, src]) => [variant, { src, alt: pack.label }]));
}

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
