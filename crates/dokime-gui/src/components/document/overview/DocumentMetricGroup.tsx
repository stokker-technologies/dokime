/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { GripVertical } from "lucide-react";
import type { CSSProperties, DragEvent } from "react";
import { Metric } from "@/components/Metric";
import { metricGroupWidthRem, type OverviewMetric } from "./metric-model";

export function DocumentMetricGroup({
  groupId,
  isDragging = false,
  title,
  metrics,
  onDragEnd,
  onDragOver,
  onDragStart,
  onDrop,
}: {
  groupId: string;
  isDragging?: boolean;
  title: string;
  metrics: OverviewMetric[];
  onDragEnd?: () => void;
  onDragOver?: (event: DragEvent<HTMLElement>) => void;
  onDragStart?: (event: DragEvent<HTMLElement>) => void;
  onDrop?: (event: DragEvent<HTMLElement>) => void;
}) {
  const groupWidth = metricGroupWidthRem(metrics.length);
  const style = {
    "--metric-group-width": `${groupWidth}rem`,
  } as CSSProperties;

  return (
    <section
      aria-label={title}
      className={`grid min-w-[min(100%,var(--metric-group-width))] max-w-full flex-[1_1_var(--metric-group-width)] gap-2 rounded-md border border-transparent p-1 transition-colors ${isDragging ? "border-primary/40 bg-primary/5 opacity-70" : ""}`}
      data-metric-group-id={groupId}
      data-testid={`metric-group-${groupId}`}
      style={style}
      onDragOver={onDragOver}
      onDrop={onDrop}
    >
      <div className="flex min-w-0 items-center gap-1">
        <button
          aria-label={`Drag ${title}`}
          className="inline-flex size-6 shrink-0 cursor-grab items-center justify-center rounded text-muted-foreground transition-colors hover:bg-muted hover:text-foreground active:cursor-grabbing focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
          draggable
          title={`Drag ${title}`}
          type="button"
          onDragEnd={onDragEnd}
          onDragStart={onDragStart}
        >
          <GripVertical size={14} />
        </button>
        <p className="min-w-0 truncate text-xs font-bold uppercase text-muted-foreground">
          {title}
        </p>
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
