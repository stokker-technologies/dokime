<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Dialog } from "bits-ui";
  import type { VariantProps } from "class-variance-authority";
  import { Check } from "lucide-svelte";
  import ErrorCard from "../../components/ErrorCard.svelte";
  import MarkdownText from "../../components/MarkdownText.svelte";
  import { badgeVariants } from "../../components/ui/badge-variants";
  import Badge from "../../components/ui/Badge.svelte";
  import { buttonVariants } from "../../components/ui/button-variants";
  import Button from "../../components/ui/Button.svelte";
  import DialogContent from "../../components/ui/DialogContent.svelte";
  import { inputClass } from "../../components/ui/input-style";
  import { cn } from "../../lib/utils";
  import { failureOf } from "../../lib/notification-details";
  import { toast } from "../../lib/notifications";
  import type { TestExecution } from "../../lib/types";
  import { isActiveExecutionStatus, STATUS_PILL_CLASS } from "./cycle-workspace-model";
  import { executionProgressText, observeElapsedSeconds } from "./elapsed";
  import { CLOSED_LABEL_DRAFT, labelEditorKey, openLabelEdit, refuseLabel, updateLabelText, type LabelDraft } from "./label-editor-model";

  type Props = {
    title: string;
    prefix?: string;
    subject: "Cycle" | "Execution";
    onRename: ((label: string) => Promise<unknown>) | null;
    status: { label: string; variant: VariantProps<typeof badgeVariants>["variant"] };
    progress?: string | null;
    executionProgress?: { execution: TestExecution | null; steps: readonly { status: string }[] };
    description?: string | null;
    markdown?: boolean;
    rerun?: { href: string; label: string; open: () => void };
    live: boolean;
  };

  let { description, executionProgress, live, markdown, onRename, prefix = "", progress, rerun, status, subject, title }: Props = $props();
  const inputId = $props.id();
  let open = $state(false);
  let draft = $state<LabelDraft>(CLOSED_LABEL_DRAFT);
  let saving = $state(false);
  let elapsed = $state<number | null>(null);
  const execution = $derived(executionProgress?.execution ?? null);
  const startedAt = $derived(execution?.started_at);
  const running = $derived(live && execution !== null && isActiveExecutionStatus(execution.status));
  const progressText = $derived(
    executionProgress ? executionProgressText(execution, executionProgress.steps, live, elapsed) : (progress ?? ""),
  );
  const renameButton = buttonVariants({ size: "sm", variant: "outline", class: "ml-auto" });

  $effect(() =>
    observeElapsedSeconds(startedAt, running, (seconds) => {
      elapsed = seconds;
    }),
  );

  function onRerun(event: MouseEvent) {
    if (event.button === 0 && !event.ctrlKey && !event.metaKey && !event.altKey && !event.shiftKey && rerun) {
      event.preventDefault();
      rerun.open();
    }
  }

  async function submit(key: string) {
    const action = labelEditorKey(draft, key);
    switch (action.kind) {
      case "cancel":
        open = false;
        break;
      case "refuse":
        draft = refuseLabel(draft, action.error);
        break;
      case "save":
        if (!onRename || saving) {
          return;
        }
        saving = true;
        try {
          await onRename(action.label);
          open = false;
        } catch (error: unknown) {
          toast.error(failureOf(error, "Label Could Not Be Saved"));
        } finally {
          saving = false;
        }
        break;
      default:
    }
  }
</script>

<Dialog.Root bind:open>
  <section class="grid min-w-0 gap-2" aria-label={`${subject} Status`}>
    {#if !live}
      <h2 class="font-sans text-lg font-semibold">{prefix}{title}</h2>
    {/if}
    <div class="flex items-center gap-3">
      <Badge class={STATUS_PILL_CLASS} variant={status.variant}>{status.label}</Badge>
      <span class="text-xs text-muted-foreground">
        {progressText}
        {#if rerun}
          {" · Rerun of "}
          <a class="underline" href={rerun.href} onclick={onRerun}>{rerun.label}</a>
        {/if}
      </span>
      {#if live && onRename}
        <Dialog.Trigger
          class={renameButton}
          data-slot="button"
          data-size="sm"
          data-variant="outline"
          onclick={() => {
            draft = openLabelEdit(title);
          }}
          >Rename</Dialog.Trigger
        >
      {/if}
    </div>
    {#if description}
      <div class="text-sm text-muted-foreground">
        {#if markdown}
          <MarkdownText text={description} />
        {:else}
          {description}
        {/if}
      </div>
    {/if}
  </section>
  <DialogContent>
    <div data-slot="dialog-header" class="flex flex-col gap-2 text-center sm:text-left">
      <Dialog.Title data-slot="dialog-title" class="text-base leading-none font-medium">Rename {subject}</Dialog.Title>
    </div>
    {#if draft.mode === "editing"}
      <form
        class="grid min-w-0 grid-cols-[auto_minmax(0,auto)_auto_minmax(0,auto)] items-center gap-x-2"
        onsubmit={(event) => {
          event.preventDefault();
          void submit("Enter");
        }}
      >
        <span></span>
        <input
          aria-invalid={Boolean(draft.error)}
          aria-label={`${subject} Label`}
          data-slot="input"
          class={cn(inputClass, "h-[var(--bar-control-height)] min-w-[16ch] font-sans text-lg leading-7 font-semibold")}
          disabled={saving}
          oninput={(event) => {
            draft = updateLabelText(draft, event.currentTarget.value);
          }}
          value={draft.text}
        >
        <Button
          aria-label={`Save ${subject} Label`}
          class="size-[var(--bar-control-height)]"
          disabled={saving}
          size="icon"
          title="Save (Enter) · Cancel (Escape)"
          type="submit"
          variant="outline"
        >
          <Check />
        </Button>
        {#if draft.error}
          <ErrorCard notificationContext="action" failure={failureOf(draft.error, "Label Could Not Save", `${inputId}:${draft.error}`)} />
        {:else}
          <span></span>
        {/if}
      </form>
    {/if}
  </DialogContent>
</Dialog.Root>
