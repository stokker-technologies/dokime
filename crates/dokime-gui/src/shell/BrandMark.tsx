/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { selectLogo, useBrand } from "@/lib/brand";
import { cn } from "@/lib/utils";

/* === Components === */

/**
 * Renders the configured surface-specific artwork or app label. The shell
 * never hardcodes a customer name — the mark comes from the brand context so
 * the product can be white-labelled.
 */
export function BrandMark({ className, treatment = "transparent" }: { className?: string; treatment?: "transparent" | "contained" }) {
  const brand = useBrand();
  return (
    <>
      {(["light", "dark"] as const).map((surface) => {
        const logo = selectLogo(brand, treatment, surface);
        const visibility = surface === "light" ? "dark:hidden" : "hidden dark:block";
        return logo ? (
          <img key={surface} alt={logo.alt ?? brand.label} className={cn("logo h-7 w-auto shrink-0", visibility, className)} src={logo.src} />
        ) : (
          <span key={surface} className={cn("logo text-sm font-black tracking-tight", visibility, className)}>
            {brand.label}
          </span>
        );
      })}
    </>
  );
}
