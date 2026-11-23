<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { VariantProps } from "class-variance-authority";
  import type { HTMLAttributes } from "svelte/elements";
  import { cn } from "../../lib/utils";
  import { badgeVariants } from "./badge-variants";

  type Props = HTMLAttributes<HTMLSpanElement> & VariantProps<typeof badgeVariants>;
  let { children, class: className, variant = "default", ...attributes }: Props = $props();
</script>

<span data-slot="badge" data-variant={variant} class={cn(badgeVariants({ variant }), className)} {...attributes}>
  {@render children?.()}
</span>
