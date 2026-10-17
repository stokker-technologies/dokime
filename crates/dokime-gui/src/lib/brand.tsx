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

/* === Types === */

export type BrandLogo = { kind: "wordmark"; text: string; className?: string } | { kind: "image"; src: string; alt?: string };

export type Brand = {
  label: string;
  logo: BrandLogo;
};

type DefaultBrandManifest = {
  label: string;
  title: string;
  logo: {
    kind: "image";
    src: string;
    alt: string;
  };
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
  logo: {
    kind: generatedDefaultBrand.logo.kind,
    src: generatedDefaultBrand.logo.src,
    alt: generatedDefaultBrand.logo.alt,
  },
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
