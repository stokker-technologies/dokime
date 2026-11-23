<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  let { text }: { text: string } = $props();
  const blocks = $derived(markdownBlocks(text));

  type MarkdownBlock = { kind: "paragraph"; text: string } | { kind: "list"; items: string[] };
  function markdownBlocks(text: string): MarkdownBlock[] {
    const blocks: MarkdownBlock[] = [];
    let listItems: string[] = [];
    text
      .trim()
      .split(/\r?\n/)
      .forEach((line) => {
        const trimmed = line.trim();
        if (!trimmed) {
          flushList(blocks, listItems);
          listItems = [];
          return;
        }
        if (trimmed.startsWith("- ") || trimmed.startsWith("* ")) {
          listItems.push(trimmed.slice(2));
          return;
        }
        flushList(blocks, listItems);
        listItems = [];
        blocks.push({ kind: "paragraph", text: trimmed });
      });
    flushList(blocks, listItems);
    return blocks;
  }

  function flushList(blocks: MarkdownBlock[], listItems: string[]) {
    if (listItems.length) {
      blocks.push({ kind: "list", items: [...listItems] });
    }
  }
</script>

{#snippet inlineMarkdown(
  content: string,
)}
  {#each content.split(/(`[^`]+`|\*\*[^*]+\*\*|\*[^*]+\*)/g) as part}
    {#if part.startsWith("`") && part.endsWith("`")}
      <code class="rounded bg-muted px-1 py-1 text-xs">{part.slice(1, -1)}</code>
    {:else if part.startsWith("**") && part.endsWith("**")}
      <strong>{part.slice(2, -2)}</strong>
    {:else if part.startsWith("*") && part.endsWith("*")}
      <em>{part.slice(1, -1)}</em>
    {:else}
      {part}
    {/if}
  {/each}
{/snippet}

<div class="min-w-0 space-y-1 text-sm leading-6">
  {#each blocks as block}
    {#if block.kind === "list"}
      <ul class="list-disc space-y-1 pl-4">
        {#each block.items as item}
          <li>{@render inlineMarkdown(item)}</li>
        {/each}
      </ul>
    {:else}
      <p>{@render inlineMarkdown(block.text)}</p>
    {/if}
  {:else}
    <p class="text-muted-foreground">Empty note.</p>
  {/each}
</div>
