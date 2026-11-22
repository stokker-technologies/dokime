<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { HTMLTableAttributes } from "svelte/elements";
  import { cn } from "../../lib/utils";

  type Props = HTMLTableAttributes & { containerClass?: string };
  let { class: className, containerClass, children, ...attributes }: Props = $props();
</script>

<div data-slot="table-container" class={cn("relative w-full overflow-x-auto", containerClass)}>
  <table data-slot="table" class={cn("w-full caption-bottom text-sm", className)} {...attributes}>
    {@render children?.()}
  </table>
</div>
