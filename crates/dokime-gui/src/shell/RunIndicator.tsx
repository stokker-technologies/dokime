/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Stream Refreshed Execution Focus In The Status Bar.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
import { useQueries } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { LoaderCircle } from "lucide-react";
import { useEffect, useState } from "react";
import { useNativeSession } from "@/app/native-session/context";
import { useCyclesQuery } from "@/features/cycles-home/cycles-queries";
import { workflowQueryKeys } from "@/features/cycle/server-state";
import { fetchNativeTestCycleDetail, fetchNativeTestExecutionDetail } from "@/lib/api/native-execution";
import { activeRunFocus, elapsedMinutes, runDestination, runElapsed } from "./run-focus";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Components === */

/* === Hooks === */

/* === Public Functions === */
export function RunIndicator() {
  const session = useNativeSession();
  const cycles = useCyclesQuery();
  const navigate = useNavigate();
  const activeCycles = (cycles.data ?? []).filter((cycle) =>
    ["Running", "Paused", "Stopping", "Blocked"].includes(cycle.status?.HarmosVariant.case ?? ""),
  );
  const details = useQueries({
    queries: activeCycles.map((cycle) => ({
      queryKey: [...workflowQueryKeys.cycleDetail(cycle.cycleId), session.identity.generation],
      queryFn: () => fetchNativeTestCycleDetail(session.identity, cycle.cycleId),
      staleTime: Number.POSITIVE_INFINITY,
    })),
  });
  const cycleDetails = details.flatMap((detail) => (detail.data ? [detail.data] : []));
  const candidates = activeRunFocus(cycleDetails);
  const executions = useQueries({
    queries: candidates.map(({ execution }) => ({
      queryKey: [...workflowQueryKeys.executionDetail(execution.test_execution_id), session.identity.generation],
      queryFn: () => fetchNativeTestExecutionDetail(session.identity, execution.test_execution_id),
      staleTime: Number.POSITIVE_INFINITY,
    })),
  });
  const hydrated = new Map(executions.flatMap((item) => (item.data ? [[item.data.test_execution_id, item.data] as const] : [])));
  const focus = activeRunFocus(
    cycleDetails.map((detail) => ({
      ...detail,
      executions: detail.executions.map((execution) => hydrated.get(execution.test_execution_id) ?? execution),
    })),
  );

  const first = focus[0];
  const [now, setNow] = useState(Date.now);
  const ticking = first !== undefined && first.percent === null;
  useEffect(() => {
    if (!ticking) return;
    const tick = () => setNow(Date.now());
    tick();
    // This updates only the displayed clock; runtime data is subscription driven.
    const timer = window.setInterval(tick, 1000);
    window.addEventListener("focus", tick);
    document.addEventListener("visibilitychange", tick);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener("focus", tick);
      document.removeEventListener("visibilitychange", tick);
    };
  }, [ticking]);
  if (!first) {
    const failed = cycles.isError || details.some((detail) => detail.isError);
    const pending = cycles.isPending || details.some((detail) => detail.isPending);
    return (
      <span className="flex min-w-0 flex-1 items-center truncate text-muted-foreground">
        {failed ? "Execution status unavailable" : pending ? "Loading executions…" : "No active execution"}
      </span>
    );
  }
  const destination = runDestination(focus);
  return (
    <button
      type="button"
      className="flex min-w-0 flex-1 items-center gap-2 text-left text-foreground outline-none focus-visible:ring-2 focus-visible:ring-ring"
      onClick={() => {
        if (destination) void navigate(destination);
      }}
    >
      {first.percent === null ? <LoaderCircle aria-hidden className="size-4 shrink-0 animate-spin motion-reduce:animate-none" /> : null}
      <span className="truncate">
        {first.cycleLabel} / {first.executionLabel} /{" "}
        {first.percent === null ? runElapsed(elapsedMinutes(first.execution.started_at, now)) : `${first.percent}%`}
      </span>
      {focus.length > 1 ? <span className="shrink-0">+{focus.length - 1}</span> : null}
    </button>
  );
}

/* === Private Functions === */
