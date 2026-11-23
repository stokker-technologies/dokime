<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { NotificationRecord } from "../../lib/notifications";

  let { record, openDetails }: { record: NotificationRecord; openDetails: () => void } = $props();
</script>

<div class="grid min-w-0 gap-1 text-left">
  <span class="font-medium">{record.title}</span>
  {#if record.description}
    <span class="select-text whitespace-pre-wrap break-words text-xs font-normal text-muted-foreground">{record.description}</span>
  {/if}
  <button
    type="button"
    class="w-fit rounded text-sm underline outline-none focus-visible:ring-2 focus-visible:ring-ring"
    aria-haspopup="dialog"
    aria-label={`Details for ${record.title}`}
    onclick={openDetails}
  >
    Details
  </button>
</div>
