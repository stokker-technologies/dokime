<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { VariantProps } from "class-variance-authority";
  import type { HTMLButtonAttributes } from "svelte/elements";
  import { cn } from "../../lib/utils";
  import { buttonVariants } from "./button-variants";

  type Props = HTMLButtonAttributes & VariantProps<typeof buttonVariants>;
  let { children, class: style, size = "default", type = "button", variant = "default", ...attributes }: Props = $props();
</script>

<button
  {type}
  data-slot="button"
  data-variant={variant}
  data-size={size}
  class={cn(buttonVariants({ variant, size }), style)}
  {...attributes}
>
  {@render children?.()}
</button>
