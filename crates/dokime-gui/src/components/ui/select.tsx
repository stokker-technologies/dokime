/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { CheckIcon, ChevronDownIcon, ChevronUpIcon } from "lucide-react";
import * as radix from "radix-ui";
import * as React from "react";

/* === Local Imports === */

import { cn } from "../../lib/utils";

/* === Components === */

function Select({ ...props }: React.ComponentProps<typeof radix.Select.Root>) {
  return <radix.Select.Root data-slot="select" {...props} />;
}

function SelectGroup({ className, ...props }: React.ComponentProps<typeof radix.Select.Group>) {
  return <radix.Select.Group data-slot="select-group" className={cn("scroll-my-1 p-1", className)} {...props} />;
}

function SelectValue({ ...props }: React.ComponentProps<typeof radix.Select.Value>) {
  return <radix.Select.Value data-slot="select-value" {...props} />;
}

const SelectTrigger = React.forwardRef<
  React.ElementRef<typeof radix.Select.Trigger>,
  React.ComponentPropsWithoutRef<typeof radix.Select.Trigger> & {
    size?: "sm" | "default";
  }
>(({ className, size = "default", children, ...props }, ref) => (
  <radix.Select.Trigger
    data-slot="select-trigger"
    data-size={size}
    ref={ref}
    className={cn(
      "flex w-fit items-center justify-between gap-2 rounded-lg border border-input bg-transparent py-2 pr-2 pl-3 text-sm whitespace-nowrap transition-colors outline-none select-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 data-placeholder:text-muted-foreground h-(--control-height) data-[size=sm]:rounded-lg *:data-[slot=select-value]:line-clamp-1 *:data-[slot=select-value]:flex *:data-[slot=select-value]:items-center *:data-[slot=select-value]:gap-2 dark:bg-input/30 dark:hover:bg-input/50 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
      className,
    )}
    {...props}
  >
    {children}
    <radix.Select.Icon asChild>
      <ChevronDownIcon className="pointer-events-none size-4 text-muted-foreground" />
    </radix.Select.Icon>
  </radix.Select.Trigger>
));

SelectTrigger.displayName = radix.Select.Trigger.displayName;

function SelectContent({
  className,
  children,
  position = "item-aligned",
  align = "center",
  ...props
}: React.ComponentProps<typeof radix.Select.Content>) {
  return (
    <radix.Select.Portal>
      <radix.Select.Content
        data-slot="select-content"
        data-align-trigger={position === "item-aligned"}
        className={cn(
          "relative z-50 max-h-(--radix-select-content-available-height) min-w-36 origin-(--radix-select-content-transform-origin) overflow-x-hidden overflow-y-auto rounded-lg bg-popover text-popover-foreground ring-1 ring-border duration-100 data-[align-trigger=true]:animate-none data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-open:animate-in data-open:fade-in-0 data-closed:animate-out data-closed:fade-out-0",
          position === "popper" &&
            "data-[side=bottom]:translate-y-1 data-[side=left]:-translate-x-1 data-[side=right]:translate-x-1 data-[side=top]:-translate-y-1",
          className,
        )}
        position={position}
        align={align}
        {...props}
      >
        <SelectScrollUpButton />
        <radix.Select.Viewport
          data-position={position}
          className={cn(
            "data-[position=popper]:h-(--radix-select-trigger-height) data-[position=popper]:w-full data-[position=popper]:min-w-(--radix-select-trigger-width)",
            position === "popper" && "",
          )}
        >
          {children}
        </radix.Select.Viewport>
        <SelectScrollDownButton />
      </radix.Select.Content>
    </radix.Select.Portal>
  );
}

function SelectLabel({ className, ...props }: React.ComponentProps<typeof radix.Select.Label>) {
  return <radix.Select.Label data-slot="select-label" className={cn("px-2 py-1 text-xs text-muted-foreground", className)} {...props} />;
}

function SelectItem({ className, children, ...props }: React.ComponentProps<typeof radix.Select.Item>) {
  return (
    <radix.Select.Item
      data-slot="select-item"
      className={cn(
        "max-lg:min-h-(--control-height) [[data-density=touch]_&]:min-h-(--control-height) relative flex w-full cursor-default items-center gap-2 rounded-lg py-1 pr-8 pl-2 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground not-data-[variant=destructive]:focus:**:text-accent-foreground data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 *:[span]:last:flex *:[span]:last:items-center *:[span]:last:gap-2",
        className,
      )}
      {...props}
    >
      <span className="pointer-events-none absolute right-2 flex size-4 items-center justify-center">
        <radix.Select.ItemIndicator>
          <CheckIcon className="pointer-events-none" />
        </radix.Select.ItemIndicator>
      </span>
      <radix.Select.ItemText>{children}</radix.Select.ItemText>
    </radix.Select.Item>
  );
}

function SelectSeparator({ className, ...props }: React.ComponentProps<typeof radix.Select.Separator>) {
  return (
    <radix.Select.Separator
      data-slot="select-separator"
      className={cn("pointer-events-none -mx-1 my-1 h-px bg-border", className)}
      {...props}
    />
  );
}

function SelectScrollUpButton({ className, ...props }: React.ComponentProps<typeof radix.Select.ScrollUpButton>) {
  return (
    <radix.Select.ScrollUpButton
      data-slot="select-scroll-up-button"
      className={cn("z-10 flex cursor-default items-center justify-center bg-popover py-1 [&_svg:not([class*='size-'])]:size-4", className)}
      {...props}
    >
      <ChevronUpIcon />
    </radix.Select.ScrollUpButton>
  );
}

function SelectScrollDownButton({ className, ...props }: React.ComponentProps<typeof radix.Select.ScrollDownButton>) {
  return (
    <radix.Select.ScrollDownButton
      data-slot="select-scroll-down-button"
      className={cn("z-10 flex cursor-default items-center justify-center bg-popover py-1 [&_svg:not([class*='size-'])]:size-4", className)}
      {...props}
    >
      <ChevronDownIcon />
    </radix.Select.ScrollDownButton>
  );
}

export {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectScrollDownButton,
  SelectScrollUpButton,
  SelectSeparator,
  SelectTrigger,
  SelectValue,
};
