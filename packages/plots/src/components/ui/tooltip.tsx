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

function TooltipProvider({ delayDuration = 0, ...props }: React.ComponentProps<typeof radix.Tooltip.Provider>) {
  return <radix.Tooltip.Provider data-slot="tooltip-provider" delayDuration={delayDuration} {...props} />;
}

function Tooltip({ ...props }: React.ComponentProps<typeof radix.Tooltip.Root>) {
  return <radix.Tooltip.Root data-slot="tooltip" {...props} />;
}

const TooltipTrigger = React.forwardRef<
  React.ElementRef<typeof radix.Tooltip.Trigger>,
  React.ComponentPropsWithoutRef<typeof radix.Tooltip.Trigger>
>(({ ...props }, ref) => <radix.Tooltip.Trigger data-slot="tooltip-trigger" ref={ref} {...props} />);

TooltipTrigger.displayName = radix.Tooltip.Trigger.displayName;

function TooltipContent({ className, sideOffset = 0, children, ...props }: React.ComponentProps<typeof radix.Tooltip.Content>) {
  return (
    <radix.Tooltip.Portal>
      <radix.Tooltip.Content
        data-slot="tooltip-content"
        sideOffset={sideOffset}
        className={cn(
          "z-50 inline-flex w-fit max-w-xs origin-(--radix-tooltip-content-transform-origin) items-center gap-2 rounded-lg bg-foreground px-3 py-2 text-xs text-background has-data-[slot=kbd]:pr-2 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 **:data-[slot=kbd]:relative **:data-[slot=kbd]:isolate **:data-[slot=kbd]:z-50 **:data-[slot=kbd]:rounded-lg data-[state=delayed-open]:animate-in data-[state=delayed-open]:fade-in-0 data-open:animate-in data-open:fade-in-0 data-closed:animate-out data-closed:fade-out-0 ",
          className,
        )}
        {...props}
      >
        {children}
        <radix.Tooltip.Arrow className="z-50 size-2.5 translate-y-[calc(-50%_-_2px)] rotate-45 rounded-lg bg-foreground fill-foreground" />
      </radix.Tooltip.Content>
    </radix.Tooltip.Portal>
  );
}

export { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger };
