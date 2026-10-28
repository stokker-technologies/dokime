/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { Activity, LoaderCircle } from "lucide-react";
import type { LiveExecutionProgress } from "@/app/live/use-execution-progress";
import { formatDuration } from "@/lib/format";
import { cn } from "@/lib/utils";

export function ProgressCard({
  progress,
  className,
  mode = "determinate",
  runFinished = false,
}: {
  progress: LiveExecutionProgress | null;
  className?: string;
  mode?: "determinate" | "manual";
  runFinished?: boolean;
}) {
  const manual = mode === "manual";
  const percent = progress ? (progress.total === 0 ? 100 : Math.max(0, Math.min(100, (progress.completed / progress.total) * 100))) : null;

  return (
    <div className={cn("grid gap-3 rounded-lg border bg-muted/20 p-3", className)}>
      <div className="flex items-center justify-between gap-3">
        <span className="inline-flex min-w-0 items-center gap-2 text-xs font-bold uppercase text-muted-foreground">
          <Activity className="shrink-0" size={14} />
          <span className="truncate">Progress</span>
        </span>
        {percent !== null && !manual ? <span className="shrink-0 text-sm font-semibold tabular-nums">{percent.toFixed(0)}%</span> : null}
      </div>

      {manual ? (
        <div className="grid gap-1 text-sm">
          <strong>{runFinished ? "Run Finished" : "Running Until Stopped"}</strong>
          <span className="text-xs text-muted-foreground">
            {runFinished ? "The operator stopped this run." : "Stop the run when enough evidence is available."}
          </span>
        </div>
      ) : progress ? (
        <div className="grid gap-2">
          <div className="h-2.5 overflow-hidden rounded-full bg-background ring-1 ring-border">
            <div className="h-full rounded-full bg-primary transition-[width] duration-500" style={{ width: `${percent ?? 0}%` }} />
          </div>
          <div className="grid gap-1 text-sm">
            <strong className="inline-flex min-w-0 items-center gap-2" title={progress.phase}>
              {(percent ?? 0) < 100 && !runFinished ? <LoaderCircle className="shrink-0 animate-spin" size={14} /> : null}
              <span className="truncate">{progress.phase}</span>
            </strong>
            <span className="truncate text-xs text-muted-foreground">
              {progress.completed.toLocaleString()} of {progress.total.toLocaleString()} complete
              {` · ${formatDuration(progress.elapsedMs / 1_000) ?? ""} elapsed`}
              {progress.remainingMs === null
                ? progress.completed > 0
                  ? " · Estimating time remaining"
                  : ""
                : ` · ${formatDuration(progress.remainingMs / 1_000) ?? ""} remaining`}
            </span>
          </div>
        </div>
      ) : runFinished ? (
        <p className="text-sm text-muted-foreground">No live progress was reported.</p>
      ) : (
        <p className="text-sm text-muted-foreground">Waiting for live progress.</p>
      )}
    </div>
  );
}
