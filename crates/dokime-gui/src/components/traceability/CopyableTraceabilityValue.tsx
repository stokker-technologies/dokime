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

/* === Constants === */

/** Values longer than this may be truncated in a row, so the tooltip repeats them. */
const LONG_VALUE_CHARS = 24;

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
          type="button"
        >
          {value}
        </button>
      </TooltipTrigger>
      {/* A short value is fully visible; the tooltip then only says what the
          click does. A long one may be truncated, so it is repeated in full. */}
      <TooltipContent
        align="end"
        className="max-w-[min(32rem,calc(100vw-2rem))] whitespace-normal break-all text-left"
        side="top"
      >
        {value.length > LONG_VALUE_CHARS ? `Copy: ${value}` : "Copy to clipboard"}
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
