<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import Button from "@/components/ui/Button.svelte";
  let { next, cursors = $bindable() }: { next: string | null; cursors: (string | null)[] } = $props();
</script>

<div class="flex items-center gap-2">
  <Button size="sm" variant="outline" disabled={cursors.length === 1} onclick={() => (cursors = cursors.slice(0, -1))}>
    Previous Page
  </Button>
  <span class="text-xs">Page {cursors.length}</span>
  <Button size="sm" variant="outline" disabled={!next} onclick={() => next && (cursors = [...cursors, next])}> Next Page </Button>
</div>
