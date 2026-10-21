/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { SquareTerminal } from "lucide-react";
import type { ReactNode } from "react";

/* === Local Imports === */

import { useNativeSession } from "@/app/native-session/context";
import { Badge } from "@/components/ui/badge";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { useRecordsQuery } from "@/features/cycle/server-state";
import { usePendingPromptsQuery } from "@/features/cycles-home/cycles-queries";
import { ActiveCyclePicker, ActiveCycleSummary } from "@/features/library/ActiveCycle";
import { useActiveCycle } from "@/features/library/use-active-cycle";
import { useInstalledPluginsQuery } from "@/features/settings/installed-plugins/installed-plugin-queries";
import { fetchStorageSources } from "@/lib/api/native-storage-settings";
import { cn } from "@/lib/utils";
import { useBackendStatus } from "@/shell/backend-status";
import { consoleAttentionCount } from "@/shell/ConsoleDrawerView";
import { useRouteCycleId } from "@/shell/route-scope";
import {
  type BackendOperation,
  operationPercent,
  operationPhaseLabel,
  operationsCountLabel,
  summarizeOperations,
} from "@/shell/status-bar-operations";
import { useHubStore } from "@/stores/hub-store";
import { useUiStore } from "@/stores/ui-store";
import { type FrozenCycleStatus, FrozenStatusBar, StatusBarFrame } from "./StatusBarFrame";

/* === Constants === */

const statusItemClassName =
  "inline-flex h-full min-w-[var(--status-bar-height)] items-center whitespace-nowrap rounded-sm px-2 outline-none focus-visible:ring-2 focus-visible:ring-ring";

const CONNECTION_LABEL = {
  connected: "Connected",
  disconnected: "Disconnected",
  reconnecting: "Reconnecting",
  unknown: "Unknown",
} as const;

/* === Types === */

export type StatusBarProps =
  | { live?: true }
  | {
      frozen: { generatedAt: string | null; status: FrozenCycleStatus };
      live: false;
    };

type AttentionEntry = {
  id: string;
  label: string;
  detail: string;
  open: () => void;
};

/* === Components === */

/**
 * The bar across the whole window, on the shell's two columns.
 *
 * Its first cell is the sidebar's column and carries the one collapse toggle,
 * at the bar's own height — so the sidebar simply ends where this bar begins
 * and nothing stands taller than the band beside it. The remaining column
 * holds its facts in two groups on one baseline: what the backend is and
 * what it stores from the left inset; what the backend is busy with, what
 * needs the operator and what is running from the right.
 */
export function StatusBar(props: StatusBarProps) {
  if (props.live === false) return <FrozenStatusBar status={props.frozen.status} />;
  return (
    <StatusBarFrame>
      <div className="flex min-w-0 items-stretch justify-between gap-6 px-[var(--shell-inset)]">
        <div className="flex min-w-0 items-stretch gap-6">
          <BackendItem />
          <StorageItem />
        </div>
        <div className="flex min-w-0 items-stretch gap-6">
          <OperationsItem />
          <AttentionItem />
          <ActiveCycleItem />
          <ConsoleItem />
        </div>
      </div>
    </StatusBarFrame>
  );
}

/**
 * The console toggle, at the bar's right end.
 *
 * Its badge counts the warnings and errors that arrived since the drawer was
 * last opened, so it reports what the operator has not seen rather than what
 * the run has produced.
 */
function ConsoleItem() {
  const drawer = useUiStore((state) => state.console);
  const setConsoleState = useUiStore((state) => state.setConsoleState);
  /* The badge promises rows the drawer can show. Away from a cycle every
     scope is Instance, which this backend does not serve, so there is nothing
     to promise and nothing to read. */
  const cycleId = useRouteCycleId();
  const records = useRecordsQuery(cycleId !== null, cycleId ? { test_cycle_id: cycleId } : null);
  const unseen = cycleId ? consoleAttentionCount(records.data ?? [], drawer.lastOpenedAt) : 0;

  return (
    <button
      aria-label={drawer.open ? "Close Console" : "Open Console"}
      className={cn(statusItemClassName, "gap-1.5", unseen > 0 ? "text-warning" : null)}
      onClick={() => setConsoleState(drawer.open ? { open: false } : { lastOpenedAt: Date.now(), open: true })}
      type="button"
    >
      <SquareTerminal className="size-3.5" />
      Console
      {unseen > 0 ? (
        <Badge className="h-4 px-1 text-[0.65rem]" variant="warning">
          {unseen}
        </Badge>
      ) : null}
    </button>
  );
}

/**
 * The backend's slow work, before what needs the operator.
 *
 * One operation shows its label, its phase and how far it has come; several
 * collapse to a count named after the longest running, and open a list of
 * every one. The segment is absent while the backend is idle.
 */
export function OperationsItem() {
  const operations = useHubStore((state) => state.operations);
  const summary = summarizeOperations(operations);

  if (summary.kind === "idle") return null;
  const shown = summary.kind === "single" ? summary.operation : summary.longest;
  const listed = summary.kind === "single" ? [summary.operation] : summary.operations;
  const label = summary.kind === "single" ? shown.label : operationsCountLabel(summary.count);

  return (
    <Popover>
      <PopoverTrigger
        aria-label={`${label}: ${operationPhaseLabel(shown)}`}
        className={cn(
          statusItemClassName,
          "grid min-w-0 grid-cols-[auto_minmax(0,auto)_4rem] gap-2",
          shown.status === "failed" ? "text-destructive" : null,
        )}
        data-testid="status-operations"
        type="button"
      >
        <span className="truncate font-medium text-foreground">{label}</span>
        <span className="truncate">{summary.kind === "many" ? shown.label : operationPhaseLabel(shown)}</span>
        <OperationBar operation={shown} />
      </PopoverTrigger>
      <PopoverContent align="end" className="grid w-80 gap-3 text-xs" side="top">
        <span className="font-medium">Backend Operations</span>
        {listed.map((operation) => (
          <span className="grid gap-1" key={operation.id}>
            <span className="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
              <span className="truncate font-medium">{operation.label}</span>
              <span className="text-muted-foreground tabular-nums">
                {operationPercent(operation) === null ? "" : `${operationPercent(operation)}%`}
              </span>
            </span>
            <span className={cn("truncate", operation.status === "failed" ? "text-destructive" : "text-muted-foreground")}>
              {operation.failure ?? operationPhaseLabel(operation)}
            </span>
            <OperationBar operation={operation} />
          </span>
        ))}
      </PopoverContent>
    </Popover>
  );
}

/** A thin bar: filled to the fraction, or a shimmer while the backend cannot tell. */
function OperationBar({ operation }: { operation: BackendOperation }) {
  const percent = operationPercent(operation);
  return (
    <span
      aria-valuemax={100}
      aria-valuemin={0}
      aria-valuenow={percent ?? undefined}
      className="relative h-1 w-full self-center overflow-hidden rounded-full bg-muted"
      role="progressbar"
    >
      {percent === null && operation.status === "running" ? (
        <span className="absolute inset-y-0 w-2/5 animate-progress-indeterminate rounded-full bg-primary motion-reduce:animate-none" />
      ) : (
        <span
          className={cn(
            "absolute inset-y-0 left-0 rounded-full transition-[width]",
            operation.status === "failed" ? "bg-destructive" : "bg-primary",
          )}
          style={{ width: `${operation.status === "running" ? (percent ?? 0) : 100}%` }}
        />
      )}
    </span>
  );
}

function BackendItem() {
  const status = useBackendStatus();
  const label = status.label ?? "Backend";
  const tone =
    status.connection === "connected"
      ? "good"
      : status.connection === "reconnecting"
        ? "warn"
        : status.connection === "disconnected"
          ? "bad"
          : "neutral";

  return (
    <StatusReadout
      title={status.detail ? `${label} — ${status.detail}` : label}
      tone={tone}
      value={`${label} · ${CONNECTION_LABEL[status.connection]}`}
    />
  );
}

function StorageItem() {
  const session = useNativeSession();
  const sources = useQuery({
    queryFn: () => fetchStorageSources(session.identity),
    queryKey: ["shell", "storage-sources", session.identity.generation],
    staleTime: 30_000,
  });
  if (sources.isError) {
    return <StatusReadout tone="warn" value="Storage Unknown" />;
  }
  const enabled = sources.data?.filter((source) => source.enabled).length;
  return (
    <StatusReadout
      tone="neutral"
      value={enabled === undefined ? "Storage Unknown" : `${enabled} ${enabled === 1 ? "Source" : "Sources"} Enabled`}
    />
  );
}

function AttentionItem() {
  const navigate = useNavigate();
  const prompts = usePendingPromptsQuery();
  const artifacts = useInstalledPluginsQuery();
  const entries: AttentionEntry[] = [
    ...(prompts.data ?? []).map((prompt) => ({
      detail: prompt.message?.trim() || "Waiting for an operator response.",
      id: `prompt:${prompt.correlation_id}`,
      label: prompt.title,
      open: () => {
        const cycleId = prompt.scope?.test_cycle_id;
        void (cycleId ? navigate({ params: { cycleId }, to: "/cycles/$cycleId" }) : navigate({ to: "/cycles" }));
      },
    })),
    ...(artifacts.data ?? [])
      .filter((artifact) => artifact.status === "failed" || artifact.error)
      .map((artifact) => ({
        detail: artifact.error ?? "The installed plugin failed to start.",
        id: `plugin:${artifact.plugin_id}`,
        label: artifact.label,
        open: () => {
          void navigate({ params: { section: "plugins" }, to: "/settings/$section" });
        },
      })),
  ];

  if (!entries.length) {
    return <StatusReadout tone="neutral" value="No Attention Needed" />;
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger className={cn(statusItemClassName, "text-warning")} type="button">
        {entries.length} Needing Attention
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-96" side="top">
        <DropdownMenuLabel>Needs Attention</DropdownMenuLabel>
        <DropdownMenuSeparator />
        {entries.map((entry) => (
          <DropdownMenuItem key={entry.id} onSelect={entry.open}>
            <span className="grid min-w-0 gap-0.5">
              <span className="truncate font-medium">{entry.label}</span>
              <span className="truncate text-xs text-muted-foreground">{entry.detail}</span>
            </span>
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

/**
 * The active cycle, the bar's last fact before the console: name, short
 * identifier and state. It opens the picker, the one way it changes.
 */
export function ActiveCycleItem() {
  const active = useActiveCycle();
  return (
    <>
      <button
        aria-label={active.cycle ? `Active Cycle: ${active.cycle.label}, ${active.cycle.statusLabel}` : "Choose Active Cycle"}
        className={cn(statusItemClassName, "min-w-0 max-w-96 gap-2")}
        data-testid="status-active-cycle"
        onClick={active.openPicker}
        type="button"
      >
        {active.cycle ? (
          <ActiveCycleSummary cycle={active.cycle} />
        ) : (
          <span>{active.cycles.length ? "Choose Active Cycle" : "No Open Cycles"}</span>
        )}
      </button>
      <ActiveCyclePicker />
    </>
  );
}

function StatusReadout({ title, tone, value }: { title?: string; tone: "neutral" | "good" | "warn" | "bad"; value: ReactNode }) {
  return (
    <span className={cn(statusItemClassName, "cursor-default", toneClassName(tone))} title={title}>
      {value}
    </span>
  );
}

/* === Private Functions === */

function toneClassName(tone: "neutral" | "good" | "warn" | "bad"): string | null {
  switch (tone) {
    case "good":
      return "text-success";
    case "warn":
      return "text-warning";
    case "bad":
      return "text-destructive";
    default:
      return null;
  }
}
