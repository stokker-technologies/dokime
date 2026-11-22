<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import ControlCard from "./ControlCard.svelte";
  import ExecutionActionButton from "./ExecutionActionButton.svelte";
  import type { ExecutionActions } from "./execution-actions";

  let { actions, ordinal }: { actions: ExecutionActions; ordinal: number } = $props();
</script>

{#snippet buttons()}
  {#each actions.state.actions as action (action)}
    <ExecutionActionButton {action} {actions} />
  {/each}
{/snippet}

<ControlCard
  detail={actions.state.detail}
  label={`Execution #${ordinal} Controls`}
  progress={actions.progress}
  state={actions.state.line}
  actions={buttons}
/>
