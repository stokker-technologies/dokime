<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import Button from "../../components/ui/Button.svelte";

  type Navigation = {
    hasPrevious: boolean;
    hasNext: boolean;
    fixed: boolean;
    busy: boolean;
    previous: () => void;
    next: () => void;
    latest: () => void;
  };
  let { navigation }: { navigation: Navigation } = $props();
</script>

{#if navigation.hasPrevious || navigation.hasNext || navigation.fixed}
  <nav aria-label="Document Pages" class="flex items-center gap-2">
    <Button variant="outline" size="sm" disabled={!navigation.hasPrevious || navigation.busy} onclick={navigation.previous}>
      Previous Page
    </Button>
    <Button variant="outline" size="sm" disabled={!navigation.hasNext || navigation.busy} onclick={navigation.next}> Next Page </Button>
    {#if navigation.fixed}
      <Button variant="ghost" size="sm" onclick={navigation.latest}>Latest</Button>
    {/if}
  </nav>
{/if}
