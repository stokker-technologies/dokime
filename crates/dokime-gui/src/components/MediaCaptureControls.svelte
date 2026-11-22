<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Camera, Mic, Monitor } from "lucide-svelte";
  import Button from "./ui/Button.svelte";
</script>

<div class="flex flex-wrap items-center gap-2 border-b pb-3">
  <Button disabled size="sm" type="button" variant="outline">
    <Monitor data-icon="inline-start" />
    Screenshot
  </Button>
  <Button disabled size="sm" type="button" variant="outline">
    <Camera data-icon="inline-start" />
    Photo
  </Button>
  <Button disabled size="sm" type="button" variant="outline">
    <Mic data-icon="inline-start" />
    Record Audio
  </Button>
  <span class="text-xs text-muted-foreground" role="status">
    Host media capture is unavailable until the host supplies its capture operations.
  </span>
</div>
