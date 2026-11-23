<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Progress, type ProgressRootProps } from "bits-ui";
  import { cn } from "../../lib/utils";

  let { class: style, value = null, ...attributes }: ProgressRootProps = $props();
</script>

<Progress.Root
  data-slot="progress"
  class={cn("relative flex h-1 w-full items-center overflow-x-hidden rounded-full bg-muted", style)}
  {value}
  {...attributes}
>
  <div
    data-slot="progress-indicator"
    data-state={value === null ? "indeterminate" : undefined}
    class="size-full flex-1 bg-primary transition-transform data-[state=indeterminate]:w-2/5 data-[state=indeterminate]:flex-none data-[state=indeterminate]:animate-progress-indeterminate motion-reduce:animate-none"
    style:transform={value === null ? undefined : `translateX(-${100 - value}%)`}
  ></div>
</Progress.Root>
