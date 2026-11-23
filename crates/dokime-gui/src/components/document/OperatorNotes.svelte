<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Tabs } from "bits-ui";
  import { Pencil, Plus } from "lucide-svelte";
  import {
    CLOSED_NOTE_DRAFT,
    type NoteDraft,
    type NoteEditorTab,
    type NoteSubmission,
    noteSubmission,
    notesNewestFirst,
    openNewNote,
    openNoteEdit,
    selectNoteTab,
    updateNoteMarkdown,
  } from "../../features/cycle/operator-notes-model";
  import { failureOf } from "../../lib/notification-details";
  import { timeOfDayLabel } from "../../lib/time-format";
  import type { OperatorNote } from "../../lib/types";
  import ErrorCard from "../ErrorCard.svelte";
  import MarkdownText from "../MarkdownText.svelte";
  import Button from "../ui/Button.svelte";

  let {
    notes,
    pending = false,
    failure,
    onReset,
    onSubmit,
  }: {
    notes: OperatorNote[];
    pending?: boolean;
    failure?: Error | null;
    onReset?: () => void;
    onSubmit?: (submission: NoteSubmission, saved: () => void) => void;
  } = $props();
  let draft = $state.raw<NoteDraft>(CLOSED_NOTE_DRAFT);
  const ordered = $derived(notesNewestFirst(notes));
  const ready = $derived(noteSubmission(draft) !== null);
  const tabs: { id: NoteEditorTab; label: string }[] = [
    { id: "code", label: "Code" },
    { id: "preview", label: "Preview" },
  ];

  function close() {
    onReset?.();
    draft = CLOSED_NOTE_DRAFT;
  }

  function submit() {
    const submission = noteSubmission(draft);
    if (submission) {
      onSubmit?.(submission, () => {
        draft = CLOSED_NOTE_DRAFT;
      });
    }
  }

  function changeTab(tab: string) {
    if (tab === "code" || tab === "preview") {
      draft = selectNoteTab(draft, tab);
    }
  }
</script>

{#snippet editor()}
  {#if draft.mode !== "closed"}
    <Tabs.Root data-slot="tabs" class="grid gap-2 border border-border/70 p-3" value={draft.tab} onValueChange={changeTab}>
      <Tabs.List data-slot="tabs-list" aria-label="Note Editor" class="flex gap-1">
        {#each tabs as { id, label } (id)}
          <Tabs.Trigger
            value={id}
            data-slot="tabs-trigger"
            class="h-(--control-height) rounded-lg px-3 text-xs font-semibold uppercase text-muted-foreground outline-none hover:text-foreground data-[state=active]:bg-muted data-[state=active]:text-foreground focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
            >{label}</Tabs.Trigger
          >
        {/each}
      </Tabs.List>
      <Tabs.Content data-slot="tabs-content" value="code" class="min-w-0 outline-none focus-visible:ring-3 focus-visible:ring-ring/50">
        <textarea
          data-slot="textarea"
          aria-label="Note Markdown"
          class="min-h-28 w-full min-w-0 rounded-lg border border-input bg-transparent px-3 py-2 font-mono text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 dark:bg-input/30"
          disabled={pending}
          oninput={(event) => {
            draft = updateNoteMarkdown(draft, event.currentTarget.value);
          }}
          placeholder="Write the note in Markdown."
          value={draft.markdown}
        ></textarea>
      </Tabs.Content>
      <Tabs.Content
        data-slot="tabs-content"
        value="preview"
        class="min-h-28 min-w-0 border border-border/40 px-3 py-2 outline-none focus-visible:ring-3 focus-visible:ring-ring/50"
      >
        <MarkdownText text={draft.markdown} />
      </Tabs.Content>
      {#if failure}
        <ErrorCard notificationContext="action" failure={failureOf(failure, "Operation Failed")} />
      {/if}
      <div class="flex justify-end gap-2">
        <Button disabled={pending} onclick={close} size="sm" variant="ghost">Cancel</Button>
        <Button disabled={!ready || pending} onclick={submit} size="sm">{draft.mode === "adding" ? "Add" : "Save"}</Button>
      </div>
    </Tabs.Root>
  {/if}
{/snippet}

{#snippet list()}
  <div class="grid gap-2">
    {#each ordered as note (note.note_id)}
      {#if draft.mode === "editing" && draft.noteId === note.note_id}
        <div>{@render editor()}</div>
      {:else}
        <article class="grid min-w-0 gap-2 border-l-2 border-border/70 pl-3">
          <div class="mb-1 flex flex-wrap items-center gap-2 text-xs font-semibold uppercase text-muted-foreground">
            <span class="normal-case text-foreground">{note.author || "Operator"}</span>
            <time datetime={note.timestamp}>{timeOfDayLabel(note.timestamp)}</time>
            {#if note.elapsed_since_start_label}
              <span title="Elapsed Since Test Start">T+{note.elapsed_since_start_label}</span>
            {/if}
            {#if onSubmit && draft.mode === "closed"}
              <Button
                aria-label="Edit Note"
                class="ml-auto"
                onclick={() => {
                  draft = openNoteEdit(note);
                }}
                size="icon-xs"
                variant="ghost"
                ><Pencil /></Button
              >
            {/if}
          </div>
          <MarkdownText text={note.text} />
        </article>
      {/if}
    {/each}
  </div>
{/snippet}

{#if onSubmit}
  <div class="grid gap-3">
    {#if draft.mode === "adding"}
      {@render editor()}
    {:else}
      <Button
        variant="outline"
        class="justify-start border-dashed text-muted-foreground"
        disabled={draft.mode !== "closed"}
        onclick={() => {
          draft = openNewNote();
        }}
      >
        <Plus size={16} />
        Add a Note
      </Button>
    {/if}
    {#if ordered.length}
      {@render list()}
    {/if}
  </div>
{:else if ordered.length}
  {@render list()}
{:else}
  <p class="text-sm text-muted-foreground">No operator notes recorded.</p>
{/if}
