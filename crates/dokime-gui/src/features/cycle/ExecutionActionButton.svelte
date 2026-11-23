<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { Ban, Pause, Play, RotateCcw, Square } from "lucide-svelte";
  import Button from "../../components/ui/Button.svelte";
  import type { ExecutionAction } from "./control-model";
  import type { ExecutionActions } from "./execution-actions";

  const icons = { Cancel: Ban, Pause, "Re-Run": RotateCcw, Resume: Play, Start: Play, Stop: Square };
  let { action, actions }: { action: ExecutionAction; actions: ExecutionActions } = $props();
  const disabled = $derived(actions.disabled(action));
  const Icon = $derived(icons[action]);
</script>

<Button
  class="h-[var(--bar-control-height)] px-3"
  data-execution-action={action}
  {disabled}
  onclick={() => actions.run(action)}
  title={disabled ? actions.refusal : undefined}
  variant={action === "Start" ? "default" : action === "Stop" ? "danger" : "outline"}
>
  <Icon data-icon="inline-start" />
  {action === "Start" || action === "Re-Run" ? action : `${action} Execution`}
</Button>
