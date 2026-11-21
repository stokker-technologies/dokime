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

import { Button } from "./button";
import { cn } from "../../lib/utils";

/* === Constants === */

// One Shrinkable Track Keeps Wide Children Inside The Box Instead Of Widening It.
const dialogContentClass =
  "fixed top-1/2 left-1/2 z-50 grid w-full max-w-[calc(100%_-_--spacing(8))] grid-cols-[minmax(0,1fr)] -translate-x-1/2 -translate-y-1/2 gap-4 rounded-lg bg-popover p-4 text-sm text-popover-foreground ring-1 ring-border duration-100 outline-none sm:max-w-sm data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95 data-closed:animate-out data-closed:fade-out-0 data-closed:zoom-out-95";

/* === Components === */

function Dialog({ ...props }: React.ComponentProps<typeof radix.Dialog.Root>) {
  return <radix.Dialog.Root data-slot="dialog" {...props} />;
}

const DialogTrigger = React.forwardRef<
  React.ElementRef<typeof radix.Dialog.Trigger>,
  React.ComponentPropsWithoutRef<typeof radix.Dialog.Trigger>
>(({ ...props }, ref) => <radix.Dialog.Trigger data-slot="dialog-trigger" ref={ref} {...props} />);

DialogTrigger.displayName = radix.Dialog.Trigger.displayName;

function DialogPortal({ ...props }: React.ComponentProps<typeof radix.Dialog.Portal>) {
  return <radix.Dialog.Portal data-slot="dialog-portal" {...props} />;
}

const DialogClose = React.forwardRef<
  React.ElementRef<typeof radix.Dialog.Close>,
  React.ComponentPropsWithoutRef<typeof radix.Dialog.Close>
>(({ ...props }, ref) => <radix.Dialog.Close data-slot="dialog-close" ref={ref} {...props} />);

DialogClose.displayName = radix.Dialog.Close.displayName;

function DialogOverlay({ className, ...props }: React.ComponentProps<typeof radix.Dialog.Overlay>) {
  return (
    <radix.Dialog.Overlay
      data-slot="dialog-overlay"
      className={cn(
        "fixed inset-0 isolate z-50 bg-black/10 duration-100 data-open:animate-in data-open:fade-in-0 data-closed:animate-out data-closed:fade-out-0",
        className,
      )}
      {...props}
    />
  );
}

function DialogContent({
  className,
  children,
  showCloseButton = true,
  ...props
}: React.ComponentProps<typeof radix.Dialog.Content> & {
  showCloseButton?: boolean;
}) {
  return (
    <DialogPortal>
      <DialogOverlay />
      <radix.Dialog.Content data-slot="dialog-content" className={cn(dialogContentClass, className)} {...props}>
        {children}
        {showCloseButton && (
          <radix.Dialog.Close data-slot="dialog-close" asChild>
            <Button variant="ghost" className="absolute top-2 right-2" size="icon-sm">
              <XIcon />
              <span className="sr-only">Close</span>
            </Button>
          </radix.Dialog.Close>
        )}
      </radix.Dialog.Content>
    </DialogPortal>
  );
}

function DialogHeader({ className, ...props }: React.ComponentProps<"div">) {
  return <div data-slot="dialog-header" className={cn("flex flex-col gap-2", className)} {...props} />;
}

function DialogFooter({
  className,
  showCloseButton = false,
  children,
  ...props
}: React.ComponentProps<"div"> & {
  showCloseButton?: boolean;
}) {
  return (
    <div
      data-slot="dialog-footer"
      className={cn(
        "-mx-4 -mb-4 flex min-w-0 flex-col-reverse gap-2 rounded-b-lg border-t bg-muted/50 p-4 sm:flex-row sm:flex-wrap sm:justify-end",
        className,
      )}
      {...props}
    >
      {children}
      {showCloseButton && (
        <radix.Dialog.Close asChild>
          <Button variant="outline">Close</Button>
        </radix.Dialog.Close>
      )}
    </div>
  );
}

function DialogTitle({ className, ...props }: React.ComponentProps<typeof radix.Dialog.Title>) {
  return (
    <radix.Dialog.Title data-slot="dialog-title" className={cn("font-sans text-base leading-none font-medium", className)} {...props} />
  );
}

function DialogDescription({ className, ...props }: React.ComponentProps<typeof radix.Dialog.Description>) {
  return (
    <radix.Dialog.Description
      data-slot="dialog-description"
      className={cn("text-sm text-muted-foreground *:[a]:underline *:[a]:underline-offset-3 *:[a]:hover:text-foreground", className)}
      {...props}
    />
  );
}

export {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogOverlay,
  DialogPortal,
  DialogTitle,
  DialogTrigger,
  dialogContentClass,
};
