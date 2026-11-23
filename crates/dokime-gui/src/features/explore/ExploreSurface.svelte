<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { ConnectedSession } from "../../app/native-session/session";
  import type { ExploreTarget } from "./explore-contracts";

  export type ExploreSurfaceProps = {
    session: ConnectedSession;
    comparing: boolean;
    onCompare: () => void;
    onOpenEntry: (target: ExploreTarget) => void;
    onToggleSelected: (entryKey: string) => void;
    selectedIds: Set<string>;
  };
</script>

<script lang="ts">
  import { createMutation } from "@tanstack/svelte-query";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import { setNativeCycleArchived, setNativeExecutionArchived } from "../../lib/api/native-execution";
  import { failureOf } from "../../lib/notification-details";
  import { toast } from "../../lib/notifications";
  import { exploreEntryKey, type ExploreEntry } from "./explore-contracts";
  import { createHistoryQuery, exploreQueryKeys } from "./explore-queries";
  import HistoryComparisonView from "./HistoryComparisonView.svelte";
  import HistoryLibrary from "./HistoryLibrary.svelte";

  let { session, comparing, onCompare, onOpenEntry, onToggleSelected, selectedIds }: ExploreSurfaceProps = $props();
  let archived = $state(false);
  const archive = $derived(archived ? "Archived" : "Unarchived");
  const client = () => session.queryClient;
  const query = createHistoryQuery(
    () => session,
    () => archive,
  );
  const cycles = $derived(query.data);
  const selected = $derived(cycles.filter((cycle) => selectedIds.has(exploreEntryKey(cycle.target))));
  const coverage = $derived(query.coverage);
  const loading = $derived(query.isPending);
  const failure = $derived(query.isError ? failureOf(query.error, "Cycles Unavailable") : undefined);
  const reasons = $derived.by(() => {
    if (!coverage) {
      return [];
    }
    const reasons = [...new Set(coverage.issues.map((issue) => `${issue.source ?? "Storage"}: ${issue.message}`))];
    const conflicts = coverage.conflicts.length + coverage.omitted_conflicts;
    if (conflicts > 0) {
      reasons.push(
        conflicts === 1
          ? "1 execution identity is claimed by more than one source."
          : `${conflicts} execution identities are claimed by more than one source.`,
      );
    }
    return reasons;
  });

  // Unarchive clears both the group mark and any independently archived execution.
  const unarchive = createMutation(
    () => ({
      mutationFn: async (entry: ExploreEntry) => {
        const { identity } = session;
        if (entry.target.kind === "cycle") {
          await setNativeCycleArchived(identity, entry.target.id, false);
        }
        for (const execution of [...entry.executions, ...entry.unrun]) {
          if (execution.archived) {
            await setNativeExecutionArchived(identity, execution.metadata.execution_id, false);
          }
        }
      },
      onSettled: refresh,
    }),
    client,
  );

  function refresh() {
    return client().invalidateQueries({ queryKey: exploreQueryKeys.all });
  }

  async function restore(entry: ExploreEntry) {
    try {
      await unarchive.mutateAsync(entry);
      toast.success("Unarchived");
    } catch (error: unknown) {
      toast.error(failureOf(error, "The Archive Mark Could Not Be Removed"));
    }
  }
</script>

{#if comparing && failure}
  <ErrorCard {failure} onRetry={refresh} />
{/if}
{#if coverage && !coverage.discovery_complete}
  <div role="status" class="text-sm text-muted-foreground">
    <p>History is incomplete: some sources or executions could not be read.</p>
    {#if reasons.length}
      <ul class="list-disc pl-4">
        {#each reasons as reason (reason)}
          <li>{reason}</li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}
{#if comparing}
  <HistoryComparisonView cycles={selected} />
{:else}
  <HistoryLibrary
    {archived}
    {cycles}
    {failure}
    {loading}
    {onCompare}
    {onOpenEntry}
    onRetry={refresh}
    onToggleArchived={() => (archived = !archived)}
    {onToggleSelected}
    onUnarchive={restore}
    {selectedIds}
  />
{/if}
