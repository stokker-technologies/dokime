<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { ConnectedSession } from "../../app/native-session/session";
  import { stepRecordLabel } from "../../components/document/step-records";
  import Badge from "../../components/ui/Badge.svelte";
  import { recordLevelLabel, recordLevelVariant } from "../../lib/record-display";
  import type { HubRecord, PromptRequest, StepExecutionRecord } from "../../lib/types";
  import ActivePrompt from "./ActivePrompt.svelte";
  import { STATUS_PILL_CLASS } from "./cycle-workspace-model";
  import ExecutionSectionCard from "./ExecutionSectionCard.svelte";
  import PromptAnswers from "./PromptAnswers.svelte";
  import { answeredPrompts, pendingPromptCountLabel, queuePrompts } from "./prompt-banner-model";

  // Keep The Current Prompt's Draft Through Polls And Reset It When The Queue Advances.
  let {
    prompts,
    records,
    session,
    steps = [],
  }: {
    prompts: PromptRequest[];
    records: readonly HubRecord[];
    session: ConnectedSession | null;
    steps?: readonly StepExecutionRecord[];
  } = $props();
  const queue = $derived(queuePrompts(prompts));
  const answered = $derived(answeredPrompts(records));
</script>

{#if queue.length || answered.length}
  <ExecutionSectionCard title="Prompts" action={queue.length ? pending : undefined}>
    {#if queue.length}
      <ol aria-label="Pending Prompts" aria-live="polite" class="grid min-w-0 gap-3">
        {#each queue as prompt, index (prompt.correlation_id)}
          {@const open = index === 0}
          {@const step = steps.find((record) => record.record_type === "step" && record.step_id === prompt.scope?.stepId)}
          <li
            class={[
              "grid min-w-0 gap-2 rounded-lg border p-3 outline-none focus-visible:ring-2 focus-visible:ring-ring",
              open ? "bg-background" : "bg-muted/40",
            ]}
            data-prompt-id={prompt.correlation_id}
            data-prompt-level={prompt.level}
            data-prompt-state={open ? "open" : "waiting"}
            tabindex="-1"
            {@attach (element) => {
              if (open) {
                element.focus();
              }
            }}
          >
            <p class="grid min-w-0 grid-cols-[auto_minmax(0,1fr)] items-baseline gap-2 text-xs text-muted-foreground">
              <Badge class={STATUS_PILL_CLASS} variant={recordLevelVariant(prompt.level)}>{recordLevelLabel(prompt.level)}</Badge>
              <span class="min-w-0 [overflow-wrap:anywhere]">
                {step ? stepRecordLabel(step, false) : prompt.scope?.stepId ? "" : "Asked by the Run"}
                {#if prompt.scope?.stepId}
                  <span class="ml-2 font-mono">{prompt.scope.stepId}</span>
                {/if}
              </span>
            </p>
            {#if open && session}
              <ActivePrompt {prompt} {session} />
            {:else}
              <div class="grid min-w-0 gap-1">
                <p class="font-sans text-sm font-semibold [overflow-wrap:anywhere]">{prompt.title}</p>
                {#if prompt.message}
                  <p class="text-sm text-muted-foreground [overflow-wrap:anywhere]">{prompt.message}</p>
                {/if}
                <p class="text-xs text-muted-foreground">Opens once the prompt above is answered.</p>
              </div>
            {/if}
          </li>
        {/each}
      </ol>
    {/if}
    <PromptAnswers {answered} />
  </ExecutionSectionCard>
{/if}

{#snippet pending()}
  <span class="text-xs text-muted-foreground">{pendingPromptCountLabel(queue.length)}</span>
{/snippet}
