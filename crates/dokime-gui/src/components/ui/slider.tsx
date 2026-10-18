/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { Slider as SliderPrimitive } from "radix-ui";
import * as React from "react";

/* === Local Imports === */

import { cn } from "@/lib/utils";

/* === Components === */

function Slider({ className, defaultValue, value, min = 0, max = 100, ...props }: React.ComponentProps<typeof SliderPrimitive.Root>) {
  const ariaLabel = props["aria-label"];
  const ariaValueText = props["aria-valuetext"];
  const values = React.useMemo(
    () => (Array.isArray(value) ? value : Array.isArray(defaultValue) ? defaultValue : [min, max]),
    [defaultValue, max, min, value],
  );

  return (
    <SliderPrimitive.Root
      data-slot="slider"
      defaultValue={defaultValue}
      value={value}
      min={min}
      max={max}
      className={cn("relative flex w-full touch-none items-center select-none data-disabled:opacity-50", className)}
      {...props}
    >
      <SliderPrimitive.Track data-slot="slider-track" className="relative h-1.5 w-full grow overflow-hidden rounded-full bg-muted">
        <SliderPrimitive.Range data-slot="slider-range" className="absolute h-full bg-primary" />
      </SliderPrimitive.Track>
      {values.map((_, index) => (
        <SliderPrimitive.Thumb
          aria-label={ariaLabel}
          aria-valuetext={ariaValueText}
          data-slot="slider-thumb"
          // The key must be POSITIONAL, never the thumb's value: keying on the
          // value remounts the thumb on the first drag step, destroying the DOM
          // node that holds Radix's pointer capture — the drag dies after one
          // step. Thumb count is fixed per slider, so the index is stable.
          // biome-ignore lint/suspicious/noArrayIndexKey: positional identity is the correct key for a fixed set of slider thumbs (see above).
          key={index}
          className="block size-4 rounded-full border border-primary bg-background shadow-sm transition-colors focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50"
        />
      ))}
    </SliderPrimitive.Root>
  );
}

export { Slider };
