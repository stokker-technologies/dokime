<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  export type PathPillProps = {
    value: string;
    align?: "left" | "right";
    copy?: boolean;
    compact?: boolean;
    onOpen?: () => void;
  };
</script>

<script lang="ts">
  import { Clipboard, ExternalLink } from "lucide-svelte";
  import { compactPath } from "../lib/path-display";
  import Button from "./ui/Button.svelte";

  let { value, align = "left", copy = true, compact = false, onOpen }: PathPillProps = $props();
  const action = $derived(`${compact ? "size-4" : "size-5"} text-muted-foreground hover:bg-background hover:text-foreground`);

  function copyPath(event: MouseEvent) {
    event.stopPropagation();
    void navigator.clipboard?.writeText(value);
  }

  function openPath(event: MouseEvent) {
    event.stopPropagation();
    onOpen?.();
  }
</script>

<span
  class={[
    "inline-flex min-w-0 max-w-full items-center gap-1 overflow-hidden rounded-none border bg-muted/60 py-1 pl-2 pr-1 font-mono text-foreground ring-1 ring-border/40",
    compact ? "w-full text-xs leading-4" : "text-xs leading-5",
    align === "right" && "justify-self-end",
  ]}
  title={value}
>
  <span class="min-w-0 truncate">{compactPath(value)}</span>
  {#if copy}
    <Button variant="plain" size="inline" aria-label="Copy Full Path" class={action} title="Copy Full Path" onclick={copyPath}>
      <Clipboard size={12} />
    </Button>
  {/if}
  {#if onOpen}
    <Button
      variant="plain"
      size="inline"
      aria-label="Open with Default Application"
      class={action}
      title="Open with Default Application"
      onclick={openPath}
    >
      <ExternalLink size={12} />
    </Button>
  {/if}
</span>
