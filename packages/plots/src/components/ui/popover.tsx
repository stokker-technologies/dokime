"use client";

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import * as radix from "radix-ui";
import * as React from "react";

import { cn } from "../../lib/utils";

function Popover({ ...props }: React.ComponentProps<typeof radix.Popover.Root>) {
  return <radix.Popover.Root data-slot="popover" {...props} />;
}

const PopoverTrigger = React.forwardRef<
  React.ElementRef<typeof radix.Popover.Trigger>,
  React.ComponentPropsWithoutRef<typeof radix.Popover.Trigger>
>(({ ...props }, ref) => <radix.Popover.Trigger data-slot="popover-trigger" ref={ref} {...props} />);

PopoverTrigger.displayName = radix.Popover.Trigger.displayName;

function PopoverAnchor({ ...props }: React.ComponentProps<typeof radix.Popover.Anchor>) {
  return <radix.Popover.Anchor data-slot="popover-anchor" {...props} />;
}

function PopoverContent({ align = "center", className, sideOffset = 4, ...props }: React.ComponentProps<typeof radix.Popover.Content>) {
  return (
    <radix.Popover.Portal>
      <radix.Popover.Content
        align={align}
        className={cn(
          "z-50 w-72 origin-(--radix-popover-content-transform-origin) rounded-lg bg-popover p-3 text-popover-foreground ring-1 ring-border outline-none duration-100 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-open:animate-in data-open:fade-in-0 data-closed:animate-out data-closed:fade-out-0",
          className,
        )}
        data-slot="popover-content"
        sideOffset={sideOffset}
        {...props}
      />
    </radix.Popover.Portal>
  );
}

export { Popover, PopoverAnchor, PopoverContent, PopoverTrigger };
