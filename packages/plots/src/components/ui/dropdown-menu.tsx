/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { CheckIcon, ChevronRightIcon } from "lucide-react";
import * as radix from "radix-ui";
import * as React from "react";
import { cn } from "../../lib/utils";

function DropdownMenu({ ...props }: React.ComponentProps<typeof radix.DropdownMenu.Root>) {
  return <radix.DropdownMenu.Root data-slot="dropdown-menu" {...props} />;
}

function DropdownMenuPortal({ ...props }: React.ComponentProps<typeof radix.DropdownMenu.Portal>) {
  return <radix.DropdownMenu.Portal data-slot="dropdown-menu-portal" {...props} />;
}

const DropdownMenuTrigger = React.forwardRef<
  React.ElementRef<typeof radix.DropdownMenu.Trigger>,
  React.ComponentPropsWithoutRef<typeof radix.DropdownMenu.Trigger>
>(({ className, ...props }, ref) => (
  <radix.DropdownMenu.Trigger
    data-slot="dropdown-menu-trigger"
    className={cn("max-lg:min-h-(--control-height) [[data-density=touch]_&]:min-h-(--control-height)", className)}
    ref={ref}
    {...props}
  />
));

DropdownMenuTrigger.displayName = radix.DropdownMenu.Trigger.displayName;

function DropdownMenuContent({
  className,
  align = "start",
  sideOffset = 4,
  ...props
}: React.ComponentProps<typeof radix.DropdownMenu.Content>) {
  return (
    <radix.DropdownMenu.Portal>
      <radix.DropdownMenu.Content
        data-slot="dropdown-menu-content"
        sideOffset={sideOffset}
        align={align}
        className={cn(
          "z-50 max-h-(--radix-dropdown-menu-content-available-height) w-(--radix-dropdown-menu-trigger-width) min-w-32 origin-(--radix-dropdown-menu-content-transform-origin) overflow-x-hidden overflow-y-auto rounded-lg bg-popover p-1 text-popover-foreground ring-1 ring-border duration-100 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-[state=closed]:overflow-hidden data-open:animate-in data-open:fade-in-0 data-closed:animate-out data-closed:fade-out-0",
          className,
        )}
        {...props}
      />
    </radix.DropdownMenu.Portal>
  );
}

function DropdownMenuGroup({ ...props }: React.ComponentProps<typeof radix.DropdownMenu.Group>) {
  return <radix.DropdownMenu.Group data-slot="dropdown-menu-group" {...props} />;
}

function DropdownMenuItem({
  className,
  inset,
  variant = "default",
  ...props
}: React.ComponentProps<typeof radix.DropdownMenu.Item> & {
  inset?: boolean;
  variant?: "default" | "destructive";
}) {
  return (
    <radix.DropdownMenu.Item
      data-slot="dropdown-menu-item"
      data-inset={inset}
      data-variant={variant}
      className={cn(
        "max-lg:min-h-(--control-height) [[data-density=touch]_&]:min-h-(--control-height) group/dropdown-menu-item relative flex cursor-default items-center gap-2 rounded-lg px-2 py-1 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground not-data-[variant=destructive]:focus:**:text-accent-foreground data-inset:pl-6 data-[variant=destructive]:text-destructive data-[variant=destructive]:focus:bg-destructive/10 data-[variant=destructive]:focus:text-destructive dark:data-[variant=destructive]:focus:bg-destructive/20 data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 data-[variant=destructive]:*:[svg]:text-destructive",
        className,
      )}
      {...props}
    />
  );
}

function DropdownMenuCheckboxItem({
  className,
  children,
  checked,
  inset,
  ...props
}: React.ComponentProps<typeof radix.DropdownMenu.CheckboxItem> & {
  inset?: boolean;
}) {
  return (
    <radix.DropdownMenu.CheckboxItem
      data-slot="dropdown-menu-checkbox-item"
      data-inset={inset}
      className={cn(
        "max-lg:min-h-(--control-height) [[data-density=touch]_&]:min-h-(--control-height) relative flex cursor-default items-center gap-2 rounded-lg py-1 pr-8 pl-2 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground focus:**:text-accent-foreground data-inset:pl-6 data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
        className,
      )}
      checked={checked}
      {...props}
    >
      <span
        className="pointer-events-none absolute right-2 flex items-center justify-center"
        data-slot="dropdown-menu-checkbox-item-indicator"
      >
        <radix.DropdownMenu.ItemIndicator>
          <CheckIcon />
        </radix.DropdownMenu.ItemIndicator>
      </span>
      {children}
    </radix.DropdownMenu.CheckboxItem>
  );
}

function DropdownMenuRadioGroup({ ...props }: React.ComponentProps<typeof radix.DropdownMenu.RadioGroup>) {
  return <radix.DropdownMenu.RadioGroup data-slot="dropdown-menu-radio-group" {...props} />;
}

function DropdownMenuRadioItem({
  className,
  children,
  inset,
  ...props
}: React.ComponentProps<typeof radix.DropdownMenu.RadioItem> & {
  inset?: boolean;
}) {
  return (
    <radix.DropdownMenu.RadioItem
      data-slot="dropdown-menu-radio-item"
      data-inset={inset}
      className={cn(
        "max-lg:min-h-(--control-height) [[data-density=touch]_&]:min-h-(--control-height) relative flex cursor-default items-center gap-2 rounded-lg py-1 pr-8 pl-2 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground focus:**:text-accent-foreground data-inset:pl-6 data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
        className,
      )}
      {...props}
    >
      <span
        className="pointer-events-none absolute right-2 flex items-center justify-center"
        data-slot="dropdown-menu-radio-item-indicator"
      >
        <radix.DropdownMenu.ItemIndicator>
          <CheckIcon />
        </radix.DropdownMenu.ItemIndicator>
      </span>
      {children}
    </radix.DropdownMenu.RadioItem>
  );
}

function DropdownMenuLabel({
  className,
  inset,
  ...props
}: React.ComponentProps<typeof radix.DropdownMenu.Label> & {
  inset?: boolean;
}) {
  return (
    <radix.DropdownMenu.Label
      data-slot="dropdown-menu-label"
      data-inset={inset}
      className={cn("px-2 py-1 text-xs font-medium text-muted-foreground data-inset:pl-6", className)}
      {...props}
    />
  );
}

function DropdownMenuSeparator({ className, ...props }: React.ComponentProps<typeof radix.DropdownMenu.Separator>) {
  return (
    <radix.DropdownMenu.Separator data-slot="dropdown-menu-separator" className={cn("-mx-1 my-1 h-px bg-border", className)} {...props} />
  );
}

function DropdownMenuShortcut({ className, ...props }: React.ComponentProps<"span">) {
  return (
    <span
      data-slot="dropdown-menu-shortcut"
      className={cn(
        "ml-auto text-xs tracking-widest text-muted-foreground group-focus/dropdown-menu-item:text-accent-foreground",
        className,
      )}
      {...props}
    />
  );
}

function DropdownMenuSub({ ...props }: React.ComponentProps<typeof radix.DropdownMenu.Sub>) {
  return <radix.DropdownMenu.Sub data-slot="dropdown-menu-sub" {...props} />;
}

function DropdownMenuSubTrigger({
  className,
  inset,
  children,
  ...props
}: React.ComponentProps<typeof radix.DropdownMenu.SubTrigger> & {
  inset?: boolean;
}) {
  return (
    <radix.DropdownMenu.SubTrigger
      data-slot="dropdown-menu-sub-trigger"
      data-inset={inset}
      className={cn(
        "flex cursor-default items-center gap-2 rounded-lg px-2 py-1 text-sm outline-hidden select-none focus:bg-accent focus:text-accent-foreground not-data-[variant=destructive]:focus:**:text-accent-foreground data-inset:pl-6 data-open:bg-accent data-open:text-accent-foreground [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
        className,
      )}
      {...props}
    >
      {children}
      <ChevronRightIcon className="ml-auto" />
    </radix.DropdownMenu.SubTrigger>
  );
}

function DropdownMenuSubContent({ className, ...props }: React.ComponentProps<typeof radix.DropdownMenu.SubContent>) {
  return (
    <radix.DropdownMenu.SubContent
      data-slot="dropdown-menu-sub-content"
      className={cn(
        "z-50 min-w-24 origin-(--radix-dropdown-menu-content-transform-origin) overflow-hidden rounded-lg bg-popover p-1 text-popover-foreground ring-1 ring-border duration-100 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-open:animate-in data-open:fade-in-0 data-closed:animate-out data-closed:fade-out-0",
        className,
      )}
      {...props}
    />
  );
}

export {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuPortal,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
};
