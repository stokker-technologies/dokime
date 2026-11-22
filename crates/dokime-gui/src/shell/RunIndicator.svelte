<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createQueries, createQuery } from "@tanstack/svelte-query";
  import { LoaderCircle } from "lucide-svelte";
  import type { ConnectedSession } from "../app/native-session/session";
  import Button from "../components/ui/Button.svelte";
  import { workflowQueryKeys } from "../features/cycle/workflow-query-keys";
  import { cyclesQueryOptions, pendingPromptsQueryOptions } from "../features/cycles-home/cycles-query-options";
  import { fetchNativeTestCycleDetail, fetchNativeTestExecutionDetail } from "../lib/api/native-execution";
  import { isNativeCycleLive } from "../lib/api/native-planning-projection";
  import { cn } from "../lib/utils";
  import { activeRunFocus, elapsedMinutes, runDestination, runElapsed } from "./run-focus";

  type Destination = NonNullable<ReturnType<typeof runDestination>>;
  let { session, onNavigate }: { session: ConnectedSession; onNavigate: (destination: Destination) => void } = $props();
  const client = () => session.queryClient;
  const cycles = createQuery(() => cyclesQueryOptions(session.identity), client);
  const prompts = createQuery(() => pendingPromptsQueryOptions(session.identity), client);
  const activeCycles = $derived((cycles.data ?? []).filter((cycle) => isNativeCycleLive(cycle.status)));
  const details = createQueries(() => {
    const identity = session.identity;
    return {
      queries: activeCycles.map((cycle) => ({
        queryKey: [...workflowQueryKeys.cycleDetail(cycle.cycleId), identity.generation],
        queryFn: () => fetchNativeTestCycleDetail(identity, cycle.cycleId),
        staleTime: Number.POSITIVE_INFINITY,
      })),
    };
  }, client);
  const cycleDetails = $derived(details.flatMap((detail) => (detail.data ? [detail.data] : [])));
  const candidates = $derived(activeRunFocus(cycleDetails));
  const executions = createQueries(() => {
    const identity = session.identity;
    return {
      queries: candidates.map(({ execution }) => ({
        queryKey: [...workflowQueryKeys.executionDetail(execution.test_execution_id), identity.generation],
        queryFn: () => fetchNativeTestExecutionDetail(identity, execution.test_execution_id),
        staleTime: Number.POSITIVE_INFINITY,
      })),
    };
  }, client);
  const focus = $derived.by(() => {
    const hydrated = new Map(executions.flatMap((item) => (item.data ? [[item.data.test_execution_id, item.data] as const] : [])));
    return activeRunFocus(
      cycleDetails.map((detail) => ({
        ...detail,
        executions: detail.executions.map((execution) => hydrated.get(execution.test_execution_id) ?? execution),
      })),
    );
  });
  const first = $derived(focus[0]);
  const failed = $derived(cycles.isError || details.some((detail) => detail.isError));
  const pending = $derived(cycles.isPending || details.some((detail) => detail.isPending));
  const startedAt = $derived(first?.execution.started_at);
  const ticking = $derived(first !== undefined && first.percent === null);
  const waiting = $derived(
    first?.execution.controls?.paused ||
      (prompts.data ?? []).some((prompt) => prompt.scope?.executionId === first?.execution.test_execution_id),
  );
  let minutes = $state<number | null>(null);

  $effect(() => {
    const start = startedAt;
    if (!ticking) {
      return;
    }
    const tick = () => {
      minutes = elapsedMinutes(start, Date.now());
    };
    tick();
    // The Clock Updates The Label. Runtime Data Arrives From Its Stream.
    const timer = window.setInterval(tick, 1000);
    window.addEventListener("focus", tick);
    document.addEventListener("visibilitychange", tick);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener("focus", tick);
      document.removeEventListener("visibilitychange", tick);
    };
  });

  function navigate() {
    const destination = runDestination(focus);
    if (destination) {
      onNavigate(destination);
    }
  }
</script>

{#if first}
  <Button
    variant="plain"
    size="inline"
    type="button"
    class="flex min-w-0 flex-1 items-center gap-2 text-left text-foreground"
    onclick={navigate}
  >
    {#if first.percent === null}
      <LoaderCircle aria-hidden="true" class={cn("size-4 shrink-0", !waiting && "animate-spin motion-reduce:animate-none")} />
    {/if}
    <span class="truncate">
      {first.cycleLabel}
      / {first.executionLabel} / {first.percent === null ? runElapsed(minutes) : `${first.percent}%`}
    </span>
    {#if focus.length > 1}
      <span class="shrink-0">+{focus.length - 1}</span>
    {/if}
  </Button>
{:else}
  <span class="flex min-w-0 flex-1 items-center truncate text-muted-foreground">
    {failed ? "Execution Status Unavailable" : pending ? "Loading Executions…" : "No Active Execution"}
  </span>
{/if}
