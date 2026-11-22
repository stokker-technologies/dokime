// Dokime — SPDX-License-Identifier: Apache-2.0
import { createContext, type ReactNode, useContext } from "react";
import { type Brand, defaultBrand } from "./brand";

const BrandContext = createContext<Brand>(defaultBrand);

export function BrandProvider({ brand = defaultBrand, children }: { brand?: Brand; children: ReactNode }) {
  return <BrandContext.Provider value={brand}>{children}</BrandContext.Provider>;
}

export function useBrand(): Brand {
  return useContext(BrandContext);
}
