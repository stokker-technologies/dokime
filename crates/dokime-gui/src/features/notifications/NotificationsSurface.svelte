<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { onMount } from "svelte";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import Button from "../../components/ui/Button.svelte";
  import { failureOf } from "../../lib/notification-details";
  import {
    clearNotifications,
    getNotificationSummary,
    markNotificationsRead,
    type NotificationCursor,
    type NotificationPage,
    notificationPage,
    subscribeNotificationSummary,
    toast,
  } from "../../lib/notifications";
  import NotificationHistoryEntry from "./NotificationHistoryEntry.svelte";

  type Selection = { cursor?: NotificationCursor; direction: "older" | "newer" };

  let { onNavigate }: { onNavigate: (href: string) => void } = $props();
  let summary = $state.raw(getNotificationSummary());
  let selection = $state.raw<Selection>({ direction: "older" });
  let page = $state.raw<NotificationPage>({ records: [], older: false, newer: false });
  let error = $state.raw<unknown>(null);
  let loading = $state(true);
  let now = $state(Date.now());

  onMount(() => {
    const unsubscribe = subscribeNotificationSummary(() => (summary = getNotificationSummary()));
    const timer = setInterval(() => (now = Date.now()), 30_000);
    return () => {
      unsubscribe();
      clearInterval(timer);
    };
  });

  $effect(() => {
    if (!summary.revision || !summary.unread) {
      return;
    }
    void markNotificationsRead().catch((failure: unknown) =>
      toast.error(failureOf(failure, "Notifications Could Not Be Marked Read"), { context: "background" }),
    );
  });

  $effect(() => {
    if (!summary.revision) {
      return;
    }
    let active = true;
    loading = true;
    void notificationPage(selection.cursor, selection.direction)
      .then((value) => {
        if (active) {
          page = value;
          error = null;
        }
      })
      .catch((failure: unknown) => {
        if (active) {
          error = failure;
        }
      })
      .finally(() => {
        if (active) {
          loading = false;
        }
      });
    return () => {
      active = false;
    };
  });

  async function clearHistory() {
    try {
      await clearNotifications();
      selection = { direction: "older" };
    } catch (failure: unknown) {
      toast.error(failureOf(failure, "Notifications Could Not Be Cleared"));
    }
  }
</script>

<div class="flex flex-wrap items-center justify-between gap-4">
  <span class="text-sm text-muted-foreground">{summary.count} Notifications</span>
  <Button variant="outline" disabled={!summary.count || loading} onclick={clearHistory}>Clear Notifications</Button>
</div>
{#if error ?? summary.error}
  <ErrorCard failure={failureOf(error ?? summary.error, "Notification History Unavailable")} />
{/if}
<ol class="divide-y rounded-lg border bg-card" aria-busy={loading}>
  {#each page.records as record (record.id)}
    <NotificationHistoryEntry {record} {now} {onNavigate} />
  {:else}
    <li class="px-4 py-8 text-center text-muted-foreground">{loading ? "Loading Notifications" : "No Notifications"}</li>
  {/each}
</ol>
<div class="flex justify-end gap-2">
  <Button variant="outline" disabled={!page.newer || loading} onclick={() => (selection = { cursor: page.records[0], direction: "newer" })}>
    Newer
  </Button>
  <Button
    variant="outline"
    disabled={!page.older || loading}
    onclick={() => (selection = { cursor: page.records.at(-1), direction: "older" })}
  >
    Older
  </Button>
</div>
