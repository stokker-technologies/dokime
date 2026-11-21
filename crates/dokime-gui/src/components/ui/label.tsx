/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import * as radix from "radix-ui";
import type * as React from "react";

/* === Local Imports === */

import { cn } from "@dokime/plots/lib/utils";

/* === Components === */

function Label({ className, ...props }: React.ComponentProps<typeof radix.Label.Root>) {
  return (
    <radix.Label.Root
      data-slot="label"
      className={cn(
        "flex items-center gap-2 text-sm leading-none font-medium select-none group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed peer-disabled:opacity-50",
        className,
      )}
      {...props}
    />
  );
}

export { Label };
