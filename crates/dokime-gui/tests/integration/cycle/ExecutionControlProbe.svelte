<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createExecutionProgress } from "@/app/live/execution-progress.svelte";
  import type { ConnectedSession } from "@/app/native-session/session";
  import { createCycleCommands } from "@/features/cycle/cycle-queries";
  import { type CycleExecutionRow, isUnstartedExecution } from "@/features/cycle/cycle-workspace-model";
  import { type ExecutionActions, executionActions } from "@/features/cycle/execution-actions";
  import ExecutionControl from "@/features/cycle/ExecutionControl.svelte";
  import type { TestExecution } from "@/lib/types";

  let {
    session,
    cycleActive,
    execution,
    onActions,
  }: {
    session: ConnectedSession;
    cycleActive: boolean;
    execution: TestExecution;
    onActions: (actions: ExecutionActions | null) => void;
  } = $props();
  const commands = createCycleCommands(
    () => session,
    () => "cycle",
  );
  const progress = createExecutionProgress(
    () => session,
    () => (isUnstartedExecution(execution.status) ? null : execution.test_execution_id),
  );
  const actions = $derived(
    executionActions(
      {
        cycleActive,
        execution,
        onSelectExecution: () => {},
        prompts: [],
        row: { execution, case: { actions: null } } as CycleExecutionRow,
      },
      commands,
      progress(),
    ),
  );
  $effect(() => onActions(actions));
</script>

{#if actions}
  <ExecutionControl {actions} ordinal={1} />
{/if}
