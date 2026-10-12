/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { useBrand } from "@/lib/brand";
import { cn } from "@/lib/utils";

/* === Components === */

/**
 * Renders the configured brand logo (a text wordmark or an image). The shell
 * never hardcodes a customer name — the mark comes from the brand context so
 * the product can be white-labelled.
 */
export function BrandMark({ className }: { className?: string }) {
  const { logo, label } = useBrand();

  if (logo.kind === "image") {
    return (
      <img
        alt={logo.alt ?? label}
        className={cn("h-7 w-auto shrink-0", className)}
        src={logo.src}
      />
    );
  }

  return (
    <span
      aria-label={label}
      className={cn(
        "grid h-7 shrink-0 place-items-center bg-primary px-2.5 text-sm font-black tracking-tight text-primary-foreground",
        logo.className,
        className,
      )}
      role="img"
    >
      {logo.text}
    </span>
  );
}
