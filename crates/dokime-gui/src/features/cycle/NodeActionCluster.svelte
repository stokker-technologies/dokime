<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import { FileArchive, Maximize2, Minimize2 } from "lucide-svelte";
  import type { Snippet } from "svelte";
  import Button from "../../components/ui/Button.svelte";
  import { failureOf } from "../../lib/notification-details";
  import { toast } from "../../lib/notifications";
  import { focusPresentation, type FocusMode } from "../../shell/focus-mode";
  import type { createCycleCommands } from "./cycle-queries";
  import DownloadReportAction from "./DownloadReportAction.svelte";
  import type { CycleMenu, ExecutionMenu } from "./node-menu";
  import NodeMenu from "./NodeMenu.svelte";

  let {
    commands,
    focus,
    menu,
    primary,
    reportSealed,
    subject,
  }: {
    commands: Pick<ReturnType<typeof createCycleCommands>, "downloadReport" | "exportEvidence">;
    focus: FocusMode;
    menu: CycleMenu | ExecutionMenu;
    primary?: Snippet;
    reportSealed: boolean;
    subject: { executionId: string | null; label: string; sealed: boolean } | null;
  } = $props();

  const focusLabel = $derived(focusPresentation(focus.focused).toggleLabel);
  const itemClass =
    "flex cursor-default items-center gap-2 rounded-lg px-2 py-1 text-sm outline-none data-highlighted:bg-accent data-disabled:pointer-events-none data-disabled:opacity-50 [&>svg]:size-4";

  async function exportEvidence() {
    if (!subject?.sealed || commands.exportEvidence.isPending) {
      return;
    }
    try {
      const saved = await commands.exportEvidence.mutateAsync({ executionId: subject.executionId });
      if (saved) {
        toast.success("Evidence Saved");
      }
    } catch (error: unknown) {
      toast.error(failureOf(error, "Evidence Could Not Be Saved"));
    }
  }
</script>

<div class="grid auto-cols-max grid-flow-col items-center gap-x-[var(--bar-gap)]" data-node-actions>
  {@render primary?.()}
  <Button
    aria-label={focusLabel}
    aria-pressed={focus.focused}
    class={focus.focused ? "h-[var(--bar-control-height)] px-3" : "size-[var(--bar-control-height)]"}
    onclick={() => focus.setFocused(!focus.focused)}
    size={focus.focused ? "default" : "icon"}
    title={focus.focused ? `${focusLabel} (Escape)` : focusLabel}
    variant={focus.focused ? "secondary" : "outline"}
  >
    {#if focus.focused}
      <Minimize2 />{focusLabel}
    {:else}
      <Maximize2 />
    {/if}
  </Button>
  <NodeMenu {menu}>
    {#if subject}
      <DropdownMenu.Item
        class={itemClass}
        data-testid="evidence-export"
        disabled={!subject.sealed || commands.exportEvidence.isPending}
        onSelect={exportEvidence}
        title={subject.sealed
          ? "Download Samples, Records, Files, Notes and Report"
          : subject.executionId
            ? "Wait for This Execution to Seal"
            : "Seal This Cycle Before Exporting"}
      >
        <FileArchive data-icon="inline-start" />
        {commands.exportEvidence.isPending ? "Exporting Evidence…" : "Export Evidence"}
      </DropdownMenu.Item>
      <DownloadReportAction
        mutation={commands.downloadReport}
        executionId={subject.executionId}
        label={subject.label}
        sealed={reportSealed}
      />
    {/if}
  </NodeMenu>
</div>
