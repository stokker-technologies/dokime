<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { currentPageChrome } from "./page-chrome.svelte";
  import PanelHeader from "./PanelHeader.svelte";

  let {
    title,
    actions,
    onBack,
    focused,
  }: {
    title: string;
    actions?: Snippet;
    onBack?: () => void;
    focused: boolean;
  } = $props();
  const native = $derived(currentPageChrome());
</script>

<PanelHeader title={focused ? "" : (native?.title ?? title)} actions={native?.actions ?? actions} {onBack} />
