<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Dialog, Tooltip } from "bits-ui";
  import { Braces, Copy, Download } from "lucide-svelte";
  import { failureOf } from "../../lib/notification-details";
  import { toast } from "../../lib/notifications";
  import { traceabilityJson, traceabilityJsonFilename, type TraceabilityExportName } from "../../lib/traceability";
  import type { TraceabilitySnapshot } from "../../lib/types";
  import { buttonVariants } from "../ui/button-variants";
  import Button from "../ui/Button.svelte";
  import DialogContent from "../ui/DialogContent.svelte";
  import { copyTraceabilityText } from "./clipboard";

  type Props = {
    exportName?: TraceabilityExportName | null;
    traceability: TraceabilitySnapshot | null | undefined;
    save?: ((json: string, filename: string) => Promise<void>) | null;
  };

  let { exportName, traceability, save }: Props = $props();
  let open = $state(false);
  let saving = $state(false);
  const json = $derived(open ? traceabilityJson(traceability) : "");
  const filename = $derived(traceabilityJsonFilename(exportName));
  const revokeAfterMs = 1_000;

  async function download() {
    saving = true;
    try {
      if (save) {
        await save(json, filename);
        return;
      }
      const url = URL.createObjectURL(new Blob([json], { type: "application/json" }));
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = filename;
      anchor.rel = "noopener";
      document.body.append(anchor);
      anchor.click();
      anchor.remove();
      window.setTimeout(() => URL.revokeObjectURL(url), revokeAfterMs);
    } catch (error: unknown) {
      toast.error(failureOf(error, "Could Not Download Traceability"));
    } finally {
      saving = false;
    }
  }
</script>

<Dialog.Root bind:open>
  <Tooltip.Provider>
    <Tooltip.Root>
      <Tooltip.Trigger
        class={buttonVariants({ size: "icon-xs", variant: "ghost" })}
        aria-label="View Traceability as JSON"
        disabled={!traceability?.groups.length}
        onclick={() => {
          open = true;
        }}
      >
        <Braces />
      </Tooltip.Trigger>
      <Tooltip.Portal>
        <Tooltip.Content side="top" sideOffset={4} class="z-50 max-w-xs rounded-md bg-foreground px-3 py-1.5 text-xs text-background">
          View as JSON
        </Tooltip.Content>
      </Tooltip.Portal>
    </Tooltip.Root>
  </Tooltip.Provider>
  <DialogContent class="grid max-h-[min(--spacing(192),calc(100dvh_-_--spacing(16)))] grid-rows-[auto_minmax(0,1fr)_auto] sm:max-w-3xl">
    <div data-slot="dialog-header" class="flex flex-col gap-2 text-center sm:text-left">
      <Dialog.Title data-slot="dialog-title" class="text-base leading-none font-medium">Traceability JSON</Dialog.Title>
      <Dialog.Description data-slot="dialog-description" class="truncate text-sm text-muted-foreground">{filename}</Dialog.Description>
    </div>
    <pre class="min-h-0 overflow-auto rounded-lg border bg-muted/30 p-3 font-mono text-xs leading-5">{json}</pre>
    <div data-slot="dialog-footer" class="flex flex-col-reverse gap-2 sm:flex-row sm:justify-end">
      <Button onclick={() => copyTraceabilityText(json)} size="sm" variant="outline">
        <Copy data-icon="inline-start" />
        Copy
      </Button>
      <Button disabled={saving} onclick={download} size="sm">
        <Download data-icon="inline-start" />
        Download
      </Button>
    </div>
  </DialogContent>
</Dialog.Root>
