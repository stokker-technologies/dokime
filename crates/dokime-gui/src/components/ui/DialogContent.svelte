<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Dialog, type DialogContentProps } from "bits-ui";
  import { XIcon } from "lucide-svelte";
  import { cn } from "../../lib/utils";
  import { buttonVariants } from "./button-variants";

  let { children, class: style, showCloseButton = true, ...attributes }: DialogContentProps & { showCloseButton?: boolean } = $props();
</script>

<Dialog.Portal>
  <Dialog.Overlay
    data-slot="dialog-overlay"
    class="fixed inset-0 isolate z-50 bg-black/10 duration-100 data-open:animate-in data-open:fade-in-0 data-closed:animate-out data-closed:fade-out-0"
  />
  <Dialog.Content
    data-slot="dialog-content"
    class={cn(
      "fixed top-1/2 left-1/2 z-50 grid w-full max-w-[calc(100%_-_--spacing(8))] grid-cols-[minmax(0,1fr)] -translate-x-1/2 -translate-y-1/2 gap-4 rounded-lg bg-popover p-4 text-sm text-popover-foreground ring-1 ring-border outline-none sm:max-w-sm",
      style,
    )}
    {...attributes}
  >
    {@render children?.()}
    {#if showCloseButton}
      <Dialog.Close
        data-slot="dialog-close"
        class={cn(buttonVariants({ variant: "ghost", size: "icon-sm" }), "absolute top-2 right-2")}
        aria-label="Close"
      >
        <XIcon />
      </Dialog.Close>
    {/if}
  </Dialog.Content>
</Dialog.Portal>
