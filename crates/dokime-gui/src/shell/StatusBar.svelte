<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { ConnectedSession } from "../app/native-session/session";
  import type { OpenCycle } from "../features/library/active-cycle-model";
  import type { BackendStatus } from "./backend-status";
  import type { runDestination } from "./run-focus";
  import type { BackendOperation } from "./status-bar-operations";

  export type AttentionEntry = { id: string; label: string; detail: string; open: () => void };
  export type StatusBarProps = {
    session: ConnectedSession;
    onRunNavigate: (destination: NonNullable<ReturnType<typeof runDestination>>) => void;
    status: BackendStatus;
    onBackends: () => void;
    operations: readonly BackendOperation[];
    attention: AttentionEntry[];
    cycle: OpenCycle | null;
    hasOpenCycles: boolean;
    onOpenCyclePicker: () => void;
    consoleOpen: boolean;
    unseen: number;
    onConsole: () => void;
  };
</script>

<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { DropdownMenu, Popover } from "bits-ui";
  import { SquareTerminal } from "lucide-svelte";
  import ErrorCard from "../components/ErrorCard.svelte";
  import Badge from "../components/ui/Badge.svelte";
  import Button from "../components/ui/Button.svelte";
  import Progress from "../components/ui/Progress.svelte";
  import ActiveCycleSummary from "../features/library/ActiveCycleSummary.svelte";
  import { storageSourcesQueryOptions } from "../features/settings/settings-queries";
  import { failureOf } from "../lib/notification-details";
  import RunIndicator from "./RunIndicator.svelte";
  import { operationPercent, operationPhaseLabel, operationsCountLabel, summarizeOperations } from "./status-bar-operations";

  let props: StatusBarProps = $props();
  const itemClass =
    "inline-flex h-full min-w-[var(--status-bar-height)] items-center whitespace-nowrap rounded-lg px-2 focus-visible:ring-2 focus-visible:ring-ring";
  const connectionLabels = { connected: "Connected", disconnected: "Offline", reconnecting: "Reconnecting", unknown: "Unknown" };
  const connectionClasses = { connected: "text-success", disconnected: "text-destructive", reconnecting: "text-warning", unknown: "" };
  const backend = $derived(props.status.label ?? "Backend");
  const sources = createQuery(
    () => storageSourcesQueryOptions(props.session.identity),
    () => props.session.queryClient,
  );
  const enabled = $derived(sources.data?.filter((source) => source.enabled).length);
  const operations = $derived(summarizeOperations(props.operations));
</script>

{#snippet operationBar(
  operation: BackendOperation,
)}
  <Progress
    aria-label={operation.label}
    value={operation.status === "running" ? operationPercent(operation) : 100}
    class={["self-center", operation.status === "failed" && "[&_[data-slot=progress-indicator]]:bg-destructive"]}
  />
{/snippet}

<footer class="flex min-w-0 shrink-0 border-t bg-card text-xs text-muted-foreground" style:height="var(--status-bar-height)">
  <div class="flex w-full min-w-0 items-stretch justify-between gap-2 px-4">
    <RunIndicator session={props.session} onNavigate={props.onRunNavigate} />
    <div class="hidden min-w-0 items-stretch gap-4 lg:flex">
      <Button
        variant="plain"
        size="inline"
        aria-label="Open Backends"
        class={[itemClass, "gap-2", connectionClasses[props.status.connection]]}
        onclick={props.onBackends}
        title={props.status.detail ? `${backend} — ${props.status.detail}` : backend}
      >
        <span aria-hidden="true" class="size-1.5 shrink-0 rounded-full bg-current"></span>
        {backend}
        · {connectionLabels[props.status.connection]}
      </Button>
      <span class={[itemClass, "cursor-default", sources.isError && "text-warning"]}>
        {sources.isError || enabled === undefined ? "Storage Unknown" : `${enabled} ${enabled === 1 ? "Source" : "Sources"} Enabled`}
      </span>
    </div>
    <div class="flex min-w-0 items-stretch gap-2">
      <div class="hidden items-stretch gap-4 tablet:flex">
        {#if operations.kind !== "idle"}
          {@const shown = operations.kind === "single" ? operations.operation : operations.longest}
          {@const listed = operations.kind === "single" ? [shown] : operations.operations}
          {@const label = operations.kind === "single" ? shown.label : operationsCountLabel(operations.count)}
          <Popover.Root>
            <Popover.Trigger
              aria-label={`${label}: ${operationPhaseLabel(shown)}`}
              class={[
                itemClass,
                "grid min-w-0 grid-cols-[auto_minmax(0,auto)_--spacing(16)] gap-2",
                shown.status === "failed" && "text-destructive",
              ]}
              data-testid="status-operations"
            >
              <span class="truncate font-medium text-foreground">{label}</span>
              <span class="truncate">{operations.kind === "many" ? shown.label : operationPhaseLabel(shown)}</span>
              {@render operationBar(shown)}
            </Popover.Trigger>
            <Popover.Portal>
              <Popover.Content
                align="end"
                side="top"
                class="z-50 grid w-80 gap-3 rounded-lg bg-popover p-4 text-xs text-popover-foreground ring-1 ring-border"
              >
                <span class="font-medium">Backend Operations</span>
                {#each listed as operation (operation.id)}
                  {@const percent = operationPercent(operation)}
                  <div class="grid min-w-0 gap-1">
                    <span class="grid grid-cols-[minmax(0,1fr)_auto] gap-2">
                      <span class="truncate font-medium">{operation.label}</span>
                      <span class="text-muted-foreground tabular-nums">{percent === null ? "" : `${percent}%`}</span>
                    </span>
                    {#if operation.failure}
                      <ErrorCard
                        failure={failureOf(operation.failure, "Operation Failed", `operation:${operation.id}:${operation.failure}`)}
                      />
                    {:else}
                      <span class="text-muted-foreground">{operationPhaseLabel(operation)}</span>
                    {/if}
                    {@render operationBar(operation)}
                  </div>
                {/each}
              </Popover.Content>
            </Popover.Portal>
          </Popover.Root>
        {/if}
        {#if props.attention.length}
          <DropdownMenu.Root>
            <DropdownMenu.Trigger class={[itemClass, "text-warning"]}>{props.attention.length} Needing Attention</DropdownMenu.Trigger>
            <DropdownMenu.Portal>
              <DropdownMenu.Content
                align="end"
                side="top"
                class="z-50 w-96 rounded-lg bg-popover p-1 text-popover-foreground ring-1 ring-border"
              >
                <DropdownMenu.Group>
                  <DropdownMenu.GroupHeading class="px-2 py-1 text-xs font-medium">Needs Attention</DropdownMenu.GroupHeading>
                  <DropdownMenu.Separator class="my-1 h-px bg-border" />
                  {#each props.attention as entry (entry.id)}
                    <DropdownMenu.Item
                      class="cursor-default rounded-lg px-2 py-1 text-sm outline-none data-highlighted:bg-accent"
                      onSelect={entry.open}
                    >
                      <span class="grid min-w-0 gap-1">
                        <span class="truncate font-medium">{entry.label}</span>
                        <span class="truncate text-xs text-muted-foreground">{entry.detail}</span>
                      </span>
                    </DropdownMenu.Item>
                  {/each}
                </DropdownMenu.Group>
              </DropdownMenu.Content>
            </DropdownMenu.Portal>
          </DropdownMenu.Root>
        {:else}
          <span class={[itemClass, "cursor-default"]}>No Attention Needed</span>
        {/if}
        <Button
          variant="plain"
          size="inline"
          aria-label={props.cycle ? `Active Cycle: ${props.cycle.label}, ${props.cycle.statusLabel}` : "Choose Active Cycle"}
          class={[itemClass, "min-w-0 max-w-96 gap-2"]}
          data-testid="status-active-cycle"
          onclick={props.onOpenCyclePicker}
        >
          {#if props.cycle}
            <ActiveCycleSummary cycle={props.cycle} />
          {:else}
            <span>{props.hasOpenCycles ? "Choose Active Cycle" : "No Open Cycles"}</span>
          {/if}
        </Button>
      </div>
      <Button
        variant="plain"
        size="inline"
        aria-label={props.consoleOpen ? "Close Console" : "Open Console"}
        class={[itemClass, "gap-2", props.unseen > 0 && "text-warning"]}
        onclick={props.onConsole}
      >
        <SquareTerminal class="size-4" />
        Console
        {#if props.unseen > 0}
          <Badge class="h-4 px-1 text-xs" variant="warning">{props.unseen}</Badge>
        {/if}
      </Button>
    </div>
  </div>
</footer>
