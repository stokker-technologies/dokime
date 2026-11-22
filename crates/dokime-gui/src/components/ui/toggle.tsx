"use client";

/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import type { VariantProps } from "class-variance-authority";
import * as radix from "radix-ui";
import type * as React from "react";

import { cn } from "@/lib/utils";
import { toggleVariants } from "./toggle-variants";

function Toggle({
  className,
  variant = "default",
  size = "default",
  ...props
}: React.ComponentProps<typeof radix.Toggle.Root> & VariantProps<typeof toggleVariants>) {
  return <radix.Toggle.Root data-slot="toggle" className={cn(toggleVariants({ variant, size, className }))} {...props} />;
}

export { Toggle };
