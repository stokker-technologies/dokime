<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import type { ComponentProps } from "svelte";
  import DownloadReportAction from "@/features/cycle/DownloadReportAction.svelte";

  let props: Omit<ComponentProps<typeof DownloadReportAction>, "presentation"> = $props();
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger>Actions</DropdownMenu.Trigger>
  <DropdownMenu.Portal>
    <DropdownMenu.Content><DownloadReportAction {...props} /></DropdownMenu.Content>
  </DropdownMenu.Portal>
</DropdownMenu.Root>
