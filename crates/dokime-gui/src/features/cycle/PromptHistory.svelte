<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { HubRecord } from "../../lib/types";
  import ExecutionSectionCard from "./ExecutionSectionCard.svelte";
  import PromptAnswers from "./PromptAnswers.svelte";
  import { answeredPrompts } from "./prompt-banner-model";

  let { records }: { records: readonly HubRecord[] } = $props();
  const answered = $derived(answeredPrompts(records));
</script>

{#if answered.length}
  <ExecutionSectionCard title="Prompts">
    <PromptAnswers {answered} />
  </ExecutionSectionCard>
{/if}
