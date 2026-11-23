<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  let { class: style, label, title }: { class?: string; label: string; title?: string } = $props();
</script>

<span
  class={[
    "inline-flex h-5 shrink-0 items-center rounded-lg border px-2 text-xs leading-none whitespace-nowrap text-muted-foreground [[data-density=touch]_&]:h-6",
    style,
  ]}
  {title}
  >{label}</span
>
