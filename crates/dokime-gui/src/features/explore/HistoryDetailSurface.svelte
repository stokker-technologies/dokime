<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { ConnectedSession } from "@/app/native-session/session";
  import type { ExploreTarget } from "./explore-contracts";

  export type HistoryDetailProps = {
    session: ConnectedSession;
    target: ExploreTarget;
    onOpenCycle?: (cycleId: string) => void;
  };
</script>

<PageChrome
  actions={historyActions}
  title={selected?.metadata.title ?? entry?.title ?? (target.kind === "cycle" ? `Cycle ${target.id}` : `Execution ${target.id}`)}
/>

<script lang="ts">
  import { Dialog } from "bits-ui";
  import { LoaderCircle } from "lucide-svelte";
  import ErrorCard from "@/components/ErrorCard.svelte";
  import Button from "@/components/ui/Button.svelte";
  import DialogContent from "@/components/ui/DialogContent.svelte";
  import { checkpointKey, deleteNativeExecution } from "@/lib/api/native-checkpoints";
  import { failureOf } from "@/lib/notification-details";
  import type { ExecutionBrowseEntry } from "@/lib/types/browse";
  import PageChrome from "@/shell/PageChrome.svelte";
  import CheckpointInspection from "./CheckpointInspection.svelte";
  import { exploreEntryKey, type ExploreEntry, isDeletableExecution, unrunCaption } from "./explore-contracts";
  import { createHistoryQuery, exploreQueryKeys } from "./explore-queries";
  import HistoryExecutionsTable from "./HistoryExecutionsTable.svelte";
  import HistoryReportAction from "./HistoryReportAction.svelte";

  let { session, target, onOpenCycle }: HistoryDetailProps = $props();
  const query = createHistoryQuery(
    () => session,
    () => "All",
  );
  const history = $derived(query.data);
  const entry = $derived(history.find((cycle) => exploreEntryKey(cycle.target) === exploreEntryKey(target)));
  const unrun = $derived(target.kind === "execution" ? findUnrunExecution(history, target.id) : null);
  const unrunCycleId = $derived(unrun?.metadata.cycle_id ?? null);
  const emptyCycle = $derived(entry?.target.kind === "cycle" && entry.execution_count === 0 && entry.unrun_count === 0);
  let selected = $state.raw<ExecutionBrowseEntry | null>(null);
  let deleteTarget = $state.raw<ExecutionBrowseEntry | null>(null);
  let deleteRetries = $state.raw<Record<string, string>>({});
  let deletePending = $state(false);
  let deleteError = $state.raw<unknown>(null);
  let deleteNotice = $state<string | null>(null);

  function closeDelete(open: boolean) {
    if (!open && !deletePending) {
      deleteTarget = null;
      deleteError = null;
    }
  }

  function retryKey(execution: ExecutionBrowseEntry): string {
    return JSON.stringify([session.identity.generation, execution.metadata.execution_id]);
  }

  async function confirmDeletion() {
    const execution = deleteTarget;
    if (!execution?.checkpoint || !isDeletableExecution(execution) || deletePending) {
      return;
    }
    const { identity, queryClient } = session;

    const key = retryKey(execution);
    const requestIdentifier = deleteRetries[key] ?? crypto.randomUUID();
    deleteRetries = { ...deleteRetries, [key]: requestIdentifier };
    deletePending = true;
    deleteError = null;
    deleteNotice = null;

    let receipt: Awaited<ReturnType<typeof deleteNativeExecution>>;
    try {
      receipt = await deleteNativeExecution(identity, execution.checkpoint, requestIdentifier);
    } catch (error: unknown) {
      if (!identity.isCurrent()) {
        return;
      }
      deleteError = error;
      deletePending = false;
      return;
    }

    if (!identity.isCurrent()) {
      return;
    }
    selected = null;
    deleteTarget = null;
    deletePending = false;
    const remaining = { ...deleteRetries };
    delete remaining[key];
    deleteRetries = remaining;
    const notice = receipt.local_complete
      ? `${execution.metadata.title} was deleted from execution history.`
      : `${execution.metadata.title} was removed from history. Local cleanup is still in progress.`;
    deleteNotice = notice;
    await queryClient.invalidateQueries({ queryKey: exploreQueryKeys.all }).catch(() => {
      if (identity.isCurrent()) {
        deleteNotice = `${notice} History could not refresh; reload to confirm the latest state.`;
      }
    });
  }

  function findUnrunExecution(entries: ExploreEntry[] | undefined, executionId: string): ExecutionBrowseEntry | null {
    for (const entry of entries ?? []) {
      const found = entry.unrun.find((execution) => execution.metadata.execution_id === executionId);
      if (found) {
        return found;
      }
    }
    return null;
  }
</script>

{#snippet historyActions()}
  {#if target.kind === "cycle"}
    <HistoryReportAction {session} cycleId={target.id} label={entry?.title ?? target.id} sealed={entry?.phase === "sealed"} />
  {/if}
{/snippet}

<div aria-busy={query.isPending} class="grid content-start">
  <div class="grid w-full gap-[var(--section-gap)] p-[var(--shell-inset)]">
    {#if query.coverage && !query.coverage.discovery_complete}
      <p role="status" class="whitespace-pre-wrap text-sm text-muted-foreground">
        History is incomplete. Some executions may be unavailable.
      </p>
    {/if}
    {#if deleteNotice}
      <p role="status" class="whitespace-pre-wrap text-sm text-muted-foreground">
        {deleteNotice}
      </p>
    {/if}
    {#if unrun}
      <section aria-label="Case Never Run" class="grid gap-2 rounded-lg border p-3">
        <h2 class="font-medium">This Case Was Never Run</h2>
        <p class="max-w-[var(--prose-measure)] text-sm text-muted-foreground">
          {unrun.metadata.title}
          was planned and never started, so this execution has no acquisition, no steps and no records behind it.
        </p>
        {#if unrunCycleId && onOpenCycle}
          <Button
            class="justify-self-start px-0"
            onclick={() => {
              if (unrunCycleId) {
                onOpenCycle?.(unrunCycleId);
              }
            }}
            variant="link"
          >
            Open the Cycle It Belongs To
          </Button>
        {/if}
      </section>
    {/if}
    <HistoryExecutionsTable
      rows={entry?.executions ?? []}
      emptyCycle={Boolean(emptyCycle)}
      loading={query.isPending}
      failure={query.isError ? failureOf(query.error, "Data Unavailable") : undefined}
      onInspect={(entry) => (selected = entry)}
      onDelete={(entry) => {
        deleteError = null;
        deleteNotice = null;
        deleteTarget = entry;
      }}
    />
    {#if !unrun && entry?.unrun_count}
      <p role="status" class="whitespace-pre-wrap text-sm text-muted-foreground">
        {unrunCaption(entry.unrun_count)}
      </p>
    {/if}
    {#if selected?.checkpoint}
      {#key checkpointKey(selected.checkpoint)}
        <CheckpointInspection {session} entry={selected} />
      {/key}
    {/if}
  </div>
  <Dialog.Root open={deleteTarget !== null} onOpenChange={closeDelete}>
    <DialogContent showCloseButton={!deletePending}>
      <div data-slot="dialog-header" class="flex flex-col gap-2 pr-8">
        <Dialog.Title class="font-sans text-base leading-none font-medium">Delete Execution</Dialog.Title>
        <Dialog.Description class="text-sm text-muted-foreground">
          Permanently delete {deleteTarget?.metadata.title ?? "this execution"} from history across configured sources? This cannot be
          undone.
        </Dialog.Description>
      </div>
      {#if deleteTarget?.checkpoint && deleteRetries[retryKey(deleteTarget)]}
        <p class="text-sm text-muted-foreground">Retry this deletion to confirm its result.</p>
      {/if}
      {#if deleteError}
        <ErrorCard notificationContext="action" failure={failureOf(deleteError, "Operation Failed")} />
      {/if}
      <div
        data-slot="dialog-footer"
        class="-mx-4 -mb-4 flex flex-col-reverse gap-2 rounded-b-lg border-t bg-muted/50 p-4 sm:flex-row sm:justify-end"
      >
        <Button variant="outline" disabled={deletePending} onclick={() => closeDelete(false)}> Cancel </Button>
        <Button variant="destructive" disabled={deletePending || !deleteTarget} onclick={() => void confirmDeletion()}>
          {#if deletePending}
            <LoaderCircle class="animate-spin" data-icon="inline-start" />
          {/if}
          {deletePending
            ? "Deleting…"
            : deleteError || (deleteTarget?.checkpoint && deleteRetries[retryKey(deleteTarget)])
              ? "Retry Delete"
              : "Delete Execution"}
        </Button>
      </div>
    </DialogContent>
  </Dialog.Root>
</div>
