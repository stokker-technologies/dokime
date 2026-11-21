/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { XIcon } from "lucide-react";
import * as radix from "radix-ui";
import * as React from "react";

/* === Local Imports === */

import { Button } from "@dokime/plots/components/ui/button";
import { cn } from "@dokime/plots/lib/utils";

/* === Components === */

function Sheet({ ...props }: React.ComponentProps<typeof radix.Dialog.Root>) {
  return <radix.Dialog.Root data-slot="sheet" {...props} />;
}

const SheetTrigger = React.forwardRef<
  React.ElementRef<typeof radix.Dialog.Trigger>,
  React.ComponentPropsWithoutRef<typeof radix.Dialog.Trigger>
>(({ ...props }, ref) => <radix.Dialog.Trigger data-slot="sheet-trigger" ref={ref} {...props} />);

SheetTrigger.displayName = radix.Dialog.Trigger.displayName;

const SheetClose = React.forwardRef<React.ElementRef<typeof radix.Dialog.Close>, React.ComponentPropsWithoutRef<typeof radix.Dialog.Close>>(
  ({ ...props }, ref) => <radix.Dialog.Close data-slot="sheet-close" ref={ref} {...props} />,
);

SheetClose.displayName = radix.Dialog.Close.displayName;

/**
 * A full-height side panel.
 *
 * `modal` defaults to `false` so the panel never traps focus or blocks the
 * surface behind it: an operator must keep reaching Stop while the panel is
 * open. Pass `modal` on the `Sheet` root when a panel really owns the screen.
 */
function SheetContent({
  className,
  children,
  side = "left",
  showCloseButton = true,
  ...props
}: React.ComponentProps<typeof radix.Dialog.Content> & {
  side?: "left" | "right";
  showCloseButton?: boolean;
}) {
  return (
    <radix.Dialog.Portal>
      <radix.Dialog.Content
        data-slot="sheet-content"
        data-side={side}
        className={cn(
          "fixed inset-y-0 z-40 flex w-full max-w-100 flex-col gap-0 bg-popover text-sm text-popover-foreground ring-1 ring-border duration-150 outline-none",
          side === "left"
            ? "left-0 data-open:slide-in-from-left data-closed:slide-out-to-left"
            : "right-0 data-open:slide-in-from-right data-closed:slide-out-to-right",
          "data-open:animate-in data-closed:animate-out",
          className,
        )}
        {...props}
      >
        {children}
        {showCloseButton && (
          <radix.Dialog.Close data-slot="sheet-close" asChild>
            <Button variant="ghost" className="absolute top-3 right-3" size="icon-sm">
              <XIcon />
              <span className="sr-only">Close</span>
            </Button>
          </radix.Dialog.Close>
        )}
      </radix.Dialog.Content>
    </radix.Dialog.Portal>
  );
}

function SheetHeader({ className, ...props }: React.ComponentProps<"div">) {
  return <div data-slot="sheet-header" className={cn("flex flex-col gap-1 border-b px-4 py-3", className)} {...props} />;
}

function SheetTitle({ className, ...props }: React.ComponentProps<typeof radix.Dialog.Title>) {
  return (
    <radix.Dialog.Title data-slot="sheet-title" className={cn("font-sans text-base leading-none font-medium", className)} {...props} />
  );
}

function SheetDescription({ className, ...props }: React.ComponentProps<typeof radix.Dialog.Description>) {
  return <radix.Dialog.Description data-slot="sheet-description" className={cn("text-sm text-muted-foreground", className)} {...props} />;
}

export { Sheet, SheetClose, SheetContent, SheetDescription, SheetHeader, SheetTitle, SheetTrigger };
