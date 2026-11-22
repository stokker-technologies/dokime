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

function Progress({ className, value, ...props }: React.ComponentProps<typeof radix.Progress.Root>) {
  return (
    <radix.Progress.Root
      data-slot="progress"
      className={cn("relative flex h-1 w-full items-center overflow-x-hidden rounded-full bg-muted", className)}
      value={value}
      {...props}
    >
      <radix.Progress.Indicator
        data-slot="progress-indicator"
        className="size-full flex-1 bg-primary transition-transform data-[state=indeterminate]:w-2/5 data-[state=indeterminate]:flex-none data-[state=indeterminate]:animate-progress-indeterminate motion-reduce:animate-none"
        style={value == null ? undefined : { transform: `translateX(-${100 - value}%)` }}
      />
    </radix.Progress.Root>
  );
}

export { Progress };
