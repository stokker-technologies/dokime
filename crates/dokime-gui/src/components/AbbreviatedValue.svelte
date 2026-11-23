<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { formatCompactDecimal } from "../lib/format";

  type Props = {
    /** The Complete Value Shown on Hover and Copied */
    exact: string;
    value?: string;
    class?: string;
  };

  let { exact, value, class: className }: Props = $props();
  const displayed = $derived(value ?? formatCompactDecimal(exact));

  function copy(event: ClipboardEvent) {
    if (!event.clipboardData || exact === displayed) {
      return;
    }
    event.clipboardData.setData("text/plain", exact);
    event.preventDefault();
  }
</script>

<span class={className} data-exact-value={exact} title={exact} oncopy={copy}>{displayed}</span>
