/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Tabs
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { Tabs as TabsPrimitive } from "radix-ui";
import type * as React from "react";

// Local Imports
import { cn } from "@/lib/utils";

/* === Re-Exports === */

export { Tabs, TabsContent, TabsList, TabsTrigger };

/* === Constants === */

/* === Types === */

/* === Components === */

function Tabs(props: React.ComponentProps<typeof TabsPrimitive.Root>) {
  return <TabsPrimitive.Root data-slot="tabs" {...props} />;
}

function TabsList({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.List>) {
  return <TabsPrimitive.List data-slot="tabs-list" className={cn("flex gap-1", className)} {...props} />;
}

function TabsTrigger({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Trigger>) {
  return (
    <TabsPrimitive.Trigger
      data-slot="tabs-trigger"
      className={cn(
        "h-(--control-height) rounded-lg px-3 text-xs font-semibold uppercase text-muted-foreground outline-none hover:text-foreground data-active:bg-muted data-active:text-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50",
        className,
      )}
      {...props}
    />
  );
}

function TabsContent({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Content>) {
  return (
    <TabsPrimitive.Content
      data-slot="tabs-content"
      className={cn("min-w-0 outline-none focus-visible:ring-3 focus-visible:ring-ring/50", className)}
      {...props}
    />
  );
}

/* === Hooks === */

/* === Public Functions === */

/* === Private Functions === */
