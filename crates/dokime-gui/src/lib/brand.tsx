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

import defaultBrandData from "../../../../branding/generated/default-brand.json";
import type { RuntimeAppLogo } from "@/lib/types/view-contracts";

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

/**
 * Default brand. The shell never hardcodes a customer name; it reads the brand from
 * context so the product can be white-labelled later (a user-facing brand
 * editor is deferred to a later phase).
 */
const generatedDefaultBrand = defaultBrandData as DefaultBrandManifest;

export const defaultBrand: Brand = {
  label: generatedDefaultBrand.label,
  logos: generatedDefaultBrand.logos,
};

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
