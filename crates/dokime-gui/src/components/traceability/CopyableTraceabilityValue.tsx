/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { copyTraceabilityText } from "@/components/traceability/clipboard";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

/** Values longer than this may be truncated in a row, so the tooltip repeats them. */
const LONG_VALUE_CHARS = 24;

type CopyableTraceabilityValueProps = {
  value: string;
};

export function CopyableTraceabilityValue({ value: rawValue }: CopyableTraceabilityValueProps) {
  /* Coerced once, and only the coerced text is read: a row value crosses JSON
     and hot reloads, and a missing one renders as empty text rather than
     throwing inside the tooltip. */
  const value = String(rawValue ?? "");
  const long = value.length > LONG_VALUE_CHARS;
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          aria-label={`Copy ${value} to clipboard`}
          className="block min-w-0 max-w-full truncate text-right text-xs font-medium underline-offset-2 hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
          onClick={() => {
            void copyTraceabilityText(value);
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
        className="max-w-[min(--spacing(128),calc(100vw_-_--spacing(8)))] whitespace-normal break-all text-left"
        side="top"
      >
        {long ? `Copy: ${value}` : "Copy to clipboard"}
      </TooltipContent>
    </Tooltip>
  );
}
