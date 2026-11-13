/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { Button } from "@/components/ui/button";
import { GripVertical } from "lucide-react";
import type { CSSProperties, PointerEvent } from "react";
import { Metric } from "@/components/Metric";
import { metricGroupWidthRem, type OverviewMetric } from "./metric-model";

export function DocumentMetricGroup({
  groupId,
  isDragging = false,
  isTarget = false,
  title,
  metrics,
  onGripPointerDown,
}: {
  groupId: string;
  isDragging?: boolean;
  isTarget?: boolean;
  title: string;
  metrics: OverviewMetric[];
  onGripPointerDown?: (event: PointerEvent<HTMLButtonElement>) => void;
}) {
  const groupWidth = metricGroupWidthRem(metrics.length);
  const style = {
    "--metric-group-width": `${groupWidth}rem`,
  } as CSSProperties;

  return (
    <section
      aria-label={title}
      className={`grid min-w-[min(100%,var(--metric-group-width))] max-w-full flex-[1_1_var(--metric-group-width)] gap-2 rounded-lg border border-transparent p-1 ${isDragging ? "border-primary/40 bg-primary/5 opacity-70" : ""} ${isTarget ? "border-primary/40" : ""}`}
      data-metric-group-id={groupId}
      data-testid={`metric-group-${groupId}`}
      style={style}
    >
      <div className="flex min-w-0 items-center gap-1">
        <Button
          variant="ghost"
          size="icon-xs"
          aria-label={`Drag ${title}`}
          className="size-6 cursor-grab touch-none text-muted-foreground active:cursor-grabbing"
          title={`Drag ${title}`}
          type="button"
          onPointerDown={onGripPointerDown}
        >
          <GripVertical size={14} />
        </Button>
        <p className="min-w-0 truncate text-xs font-bold uppercase text-muted-foreground">{title}</p>
      </div>
      <div className="grid gap-2 [grid-template-columns:repeat(auto-fit,minmax(min(100%,9.5rem),1fr))]">
        {metrics.map((metric) => (
          <Metric
            detail={metric.detail}
            key={metric.label}
            label={metric.label}
            rawValue={metric.rawValue}
            presentation={metric.presentation}
            state={metric.state}
            value={metric.value}
          />
        ))}
      </div>
    </section>
  );
}
