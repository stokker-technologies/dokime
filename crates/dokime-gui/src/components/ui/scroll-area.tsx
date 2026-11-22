/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import * as radix from "radix-ui";
import type * as React from "react";

import { cn } from "@/lib/utils";

function ScrollArea({ className, children, ...props }: React.ComponentProps<typeof radix.ScrollArea.Root>) {
  return (
    <radix.ScrollArea.Root data-slot="scroll-area" className={cn("relative", className)} {...props}>
      <radix.ScrollArea.Viewport
        data-slot="scroll-area-viewport"
        className="size-full rounded-[inherit] transition-[color,box-shadow] outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-1"
      >
        {children}
      </radix.ScrollArea.Viewport>
      <ScrollBar />
      <radix.ScrollArea.Corner />
    </radix.ScrollArea.Root>
  );
}

function ScrollBar({ className, orientation = "vertical", ...props }: React.ComponentProps<typeof radix.ScrollArea.ScrollAreaScrollbar>) {
  return (
    <radix.ScrollArea.ScrollAreaScrollbar
      data-slot="scroll-area-scrollbar"
      data-orientation={orientation}
      orientation={orientation}
      className={cn(
        "flex touch-none p-px transition-colors select-none data-horizontal:h-2.5 data-horizontal:flex-col data-horizontal:border-t data-horizontal:border-t-transparent data-vertical:h-full data-vertical:w-2.5 data-vertical:border-l data-vertical:border-l-transparent",
        className,
      )}
      {...props}
    >
      <radix.ScrollArea.ScrollAreaThumb data-slot="scroll-area-thumb" className="relative flex-1 rounded-full bg-border" />
    </radix.ScrollArea.ScrollAreaScrollbar>
  );
}

export { ScrollArea, ScrollBar };
