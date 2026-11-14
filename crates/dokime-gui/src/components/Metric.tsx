/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { PathPill } from "@/components/PathPill";
import { Badge } from "@/components/ui/badge";
import { Progress } from "@/components/ui/progress";
import { Card, CardContent } from "@/components/ui/card";
import { isPathLike } from "@/lib/path-display";
import type { MetricState } from "@/lib/types";
import { cn } from "@/lib/utils";

/* === Types === */

type MetricProps = {
  metricId?: string;
  label: string;
  value: string;
  rawValue?: unknown;
  detail?: string;
  presentation?: "value" | "progress" | "badge";
  state?: MetricState | null;
};

/* === Components === */

export function Metric({ metricId, label, value, rawValue, detail, presentation = "value", state = null }: MetricProps) {
  const progress = presentation === "progress" ? progressValue(rawValue, value) : null;

  return (
    <Card data-document-metric={metricId} className={cn("min-h-18 min-w-0 py-2", stateClass(state))} size="sm">
      <CardContent className="grid min-w-0 grid-rows-[--spacing(5)_minmax(--spacing(6),auto)_--spacing(3)] gap-1 px-3">
        <span className="min-w-0 self-end truncate text-xs font-bold leading-tight text-muted-foreground" title={label}>
          {label}
        </span>
        {progress === null ? (
          <strong className="min-w-0 text-lg leading-tight">
            <MetricValue forceBadge={presentation === "badge"} state={state} value={value} />
          </strong>
        ) : (
          <ProgressValue percent={progress} state={state} />
        )}
        {detail ? (
          <small className="min-w-0 truncate text-xs leading-tight text-muted-foreground" title={detail}>
            {detail}
          </small>
        ) : null}
      </CardContent>
    </Card>
  );
}

function ProgressValue({ percent, state }: { percent: number; state?: MetricState | null }) {
  return (
    <div className="grid min-w-0 gap-1 self-center">
      <Progress aria-label="Progress" value={percent} className={cn("h-2 bg-background ring-1 ring-border", progressClass(state))} />
      <span className="text-xs font-semibold tabular-nums">{percent.toFixed(0)}%</span>
    </div>
  );
}

function MetricValue({ forceBadge = false, state, value }: { forceBadge?: boolean; state?: MetricState | null; value: string }) {
  const normalized = value.trim().toLowerCase();
  const status = stateVariant(state) ?? statusVariant(normalized);
  if (status) {
    return (
      <Badge className="max-w-full truncate capitalize" variant={status}>
        {value}
      </Badge>
    );
  }
  if (forceBadge) {
    return (
      <Badge className="max-w-full truncate" variant="outline">
        {value}
      </Badge>
    );
  }
  if (isPathLike(value)) {
    return <PathPill value={value} />;
  }
  const compact = value.length > 10;
  return (
    <span className={`block min-w-0 max-w-full truncate ${compact ? "text-base" : ""}`} title={value}>
      {value}
    </span>
  );
}

function progressValue(rawValue: unknown, formatted: string): number | null {
  const candidate = typeof rawValue === "number" ? rawValue : Number.parseFloat(formatted.replace("%", ""));
  if (!Number.isFinite(candidate)) {
    return null;
  }
  return Math.max(0, Math.min(100, candidate));
}

function statusVariant(value: string): "default" | "secondary" | "destructive" | null {
  if (["connected", "completed", "passed", "running", "true"].includes(value)) {
    return "default";
  }
  if (["failed", "error", "disconnected", "false", "blocked"].includes(value)) {
    return "destructive";
  }
  if (["unknown", "queued", "skipped", "stopped", "stopping"].includes(value)) {
    return "secondary";
  }
  return null;
}

function stateVariant(state?: MetricState | null): "default" | "secondary" | "destructive" | "success" | "warning" | null {
  if (state === "ok") return "success";
  if (state === "warning") return "warning";
  if (state === "critical") return "destructive";
  if (state === "muted") return "secondary";
  return null;
}

function stateClass(state?: MetricState | null): string {
  if (state === "ok") return "border-success/30 bg-success/10";
  if (state === "warning") return "border-warning/30 bg-warning/10";
  if (state === "critical") return "border-destructive/30 bg-destructive/10";
  return "";
}

function progressClass(state?: MetricState | null): string {
  if (state === "ok") return "[&_[data-slot=progress-indicator]]:bg-success";
  if (state === "warning") return "[&_[data-slot=progress-indicator]]:bg-warning";
  if (state === "critical") return "[&_[data-slot=progress-indicator]]:bg-destructive";
  if (state === "muted") return "[&_[data-slot=progress-indicator]]:bg-muted-foreground";
  return "[&_[data-slot=progress-indicator]]:bg-primary";
}
