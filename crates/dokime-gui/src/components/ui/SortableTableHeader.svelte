<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { ArrowDown, ArrowUp } from "lucide-svelte";
  import Button from "./Button.svelte";

  let {
    label,
    direction = "none",
    onclick,
    class: style,
  }: {
    label: string;
    direction?: "none" | "ascending" | "descending";
    onclick: () => void;
    class?: string;
  } = $props();
</script>

<th scope="col" aria-sort={direction} class={["h-(--list-row-height) font-medium", style]}>
  <Button variant="plain" size="inline" {onclick}>
    {label}
    {#if direction === "ascending"}
      <ArrowUp aria-hidden="true" />
    {:else if direction === "descending"}
      <ArrowDown aria-hidden="true" />
    {/if}
  </Button>
</th>
