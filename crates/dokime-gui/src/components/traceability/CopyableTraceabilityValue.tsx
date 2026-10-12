/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { toast } from "sonner";

/* === Local Imports === */

import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

/* === Components === */

export function CopyableTraceabilityValue({ value }: { value: string }) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          aria-label={`Copy ${value} to clipboard`}
          className="block min-w-0 max-w-full truncate text-right text-xs font-medium underline-offset-2 hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          onClick={() => {
            void copyTraceabilityValue(value);
          }}
          title={value}
          type="button"
        >
          {value}
        </button>
      </TooltipTrigger>
      <TooltipContent
        align="end"
        className="max-w-[min(32rem,calc(100vw-2rem))] whitespace-normal break-all text-left"
        side="top"
      >
        {value}
      </TooltipContent>
    </Tooltip>
  );
}

/* === Private Functions === */

async function copyTraceabilityValue(value: string): Promise<void> {
  try {
    if (!navigator.clipboard?.writeText) {
      throw new Error("Clipboard API is unavailable.");
    }
    await navigator.clipboard.writeText(value);
    toast.success("Copied to clipboard");
  } catch {
    toast.error("Could not copy value");
  }
}
