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
export function BrandMark({
  className,
  heading = false,
  logoOnly = false,
  treatment = "transparent",
}: {
  className?: string;
  heading?: boolean;
  logoOnly?: boolean;
  treatment?: "transparent" | "contained";
}) {
  const brand = useBrand();
  const artwork = selectLogo(brand, treatment, "light") ?? selectLogo(brand, treatment, "dark");
  const Wordmark = heading ? "h1" : "span";
  return (
    <div className={cn("brand brand-row brand-center", className)}>
      {(["light", "dark"] as const).map((surface) => {
        const logo = selectLogo(brand, treatment, surface);
        const visibility = surface === "light" ? "dark:hidden" : "hidden dark:block";
        if (!logo) {
          return artwork && logoOnly ? (
            <span key={surface} className={cn("logo", visibility)}>
              {brand.label}
            </span>
          ) : null;
        }
        return <img key={surface} alt={logo.alt ?? brand.label} className={cn("logo", visibility)} src={logo.src} />;
      })}
      {(!logoOnly || !artwork) && <Wordmark className={cn("wordmark", !artwork && "logo")}>{brand.label}</Wordmark>}
    </div>
  );
}
