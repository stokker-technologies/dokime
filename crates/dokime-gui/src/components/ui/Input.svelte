<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import { cn } from "../../lib/utils";
  import { inputClass } from "./input-style";

  let { class: style, ...props }: HTMLInputAttributes = $props();
</script>

<input data-slot="input" class={cn(inputClass, style)} {...props}>
