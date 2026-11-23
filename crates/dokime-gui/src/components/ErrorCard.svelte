<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { AlertCircle } from "lucide-svelte";
  import { untrack } from "svelte";
  import { routeFailure } from "../lib/failure-routing";
  import { failureOf, failureText, isWaitingFailure, type NotificationFailure } from "../lib/notification-details";
  import { toast } from "../lib/notifications";
  import { getErrorRecovery } from "./error-recovery";
  import FailureDetails from "./FailureDetails.svelte";
  import Button from "./ui/Button.svelte";

  type Props = {
    retryFirst?: boolean;
    notificationContext?: "action" | "background";
    failure: NotificationFailure;
    onRetry?: () => void;
    retryLabel?: string;
    retryDisabled?: boolean;
  };

  let { failure, notificationContext, onRetry, retryDisabled = false, retryFirst = false, retryLabel = "Reload" }: Props = $props();
  const resolveRecovery = getErrorRecovery();
  const recovery = $derived(resolveRecovery?.(failure));
  const recoveryLabel = $derived(
    retryFirst && onRetry ? retryLabel : recovery ? (failure.action ?? retryLabel) : onRetry ? retryLabel : "Reload",
  );
  const recoverable = $derived(Boolean(onRetry || recovery || !failure.action || failure.action === "Reload"));
  const message = $derived(
    [...new Set([failure.message, ...(failure.causes ?? [])])]
      .filter((cause, index, causes) => !causes.slice(0, index).some((parent) => parent.includes(cause)))
      .join("\n"),
  );
  let recovering = $state(false);

  $effect(() => {
    const details = failure;
    const context = notificationContext;
    untrack(() => {
      if (context) {
        toast.error(details, { context });
      } else {
        routeFailure(details, "view");
      }
    });
  });

  async function recover() {
    if (onRetry && retryLabel === recoveryLabel) {
      onRetry();
      return;
    }
    if (!recovery) {
      window.location.reload();
      return;
    }
    recovering = true;
    try {
      await recovery();
    } catch (error: unknown) {
      toast.error(failureOf(error, "Plugin Restart Failed"));
    } finally {
      recovering = false;
    }
  }

  async function copyDetails() {
    try {
      await navigator.clipboard.writeText(failureText(failure));
    } catch (error: unknown) {
      toast.error(failureOf(error, "Could Not Copy Failure"));
    }
  }
</script>

{#if isWaitingFailure(failure)}
  <p role="status" class="col-span-full w-full text-sm text-muted-foreground">
    Waiting for Plugin{failure.plugin ? ` ${failure.plugin}` : ""}
  </p>
{:else}
  <section
    role="alert"
    aria-label={failure.title}
    class="col-span-full grid w-full min-w-0 gap-3 rounded-md border border-destructive bg-card p-4 text-left text-card-foreground"
  >
    <div class="flex items-center gap-2">
      <AlertCircle aria-hidden="true" class="size-5 shrink-0 text-destructive" />
      <h3 class="font-sans text-sm font-semibold">{failure.title}</h3>
    </div>
    <p class="select-text whitespace-pre-wrap break-words text-sm">{message}</p>
    {#if failure.action && (!recoverable || failure.action !== recoveryLabel)}
      <p class="whitespace-pre-wrap text-sm">{failure.action}</p>
    {/if}
    <div class="flex gap-2">
      {#if recoverable}
        <Button type="button" variant="outline" size="sm" disabled={retryDisabled || recovering} onclick={recover}>
          {recoveryLabel}
        </Button>
      {/if}
      <Button type="button" variant="outline" size="sm" onclick={copyDetails}>Copy Details</Button>
    </div>
    <FailureDetails {failure} />
  </section>
{/if}
