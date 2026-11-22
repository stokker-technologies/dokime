<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Toaster } from "svelte-sonner";
  import NotificationDetailsDialog from "../features/notifications/NotificationDetailsDialog.svelte";

  let { theme = "system" }: { theme?: "light" | "dark" | "system" } = $props();
  const classes = {
    toast:
      "relative flex w-full items-start gap-3 rounded-lg border border-border bg-popover p-4 pr-10 font-sans text-sm text-popover-foreground",
    title: "font-medium text-popover-foreground",
    description: "text-xs text-muted-foreground",
    content: "grid min-w-0 gap-1",
    icon: "mt-1 shrink-0",
    success: "border-success bg-popover",
    error: "border-destructive bg-popover",
    warning: "border-warning bg-popover",
    info: "border-info bg-popover",
    loading: "border-border bg-muted",
    actionButton: "relative z-10 shrink-0 whitespace-nowrap rounded-lg bg-primary px-3 py-2 text-primary-foreground",
    cancelButton: "relative z-10 rounded-lg bg-secondary px-2 py-1 text-secondary-foreground",
    closeButton:
      "absolute top-2 right-2 z-10 grid size-6 place-items-center rounded-lg text-muted-foreground hover:bg-muted hover:text-popover-foreground [&>svg]:size-4",
  };
</script>

<Toaster
  class="z-[100]"
  closeButton
  offset={{ bottom: "calc(var(--status-bar-height) + var(--shell-gap))", right: "var(--shell-gap)" }}
  position="bottom-right"
  style="--width: 28rem"
  {theme}
  toastOptions={{ unstyled: true, classes }}
/>
<NotificationDetailsDialog />
