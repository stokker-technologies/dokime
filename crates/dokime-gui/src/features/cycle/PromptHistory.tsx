// Dokime — SPDX-License-Identifier: Apache-2.0
import { SvelteMount } from "../../components/SvelteMount";
import type { HubRecord } from "../../lib/types";
import { ExecutionSectionCard } from "./ExecutionSectionCard";
import { answeredPrompts } from "./prompt-banner-model";
import PromptAnswers from "./PromptAnswers.svelte";

export function PromptHistory({ records }: { records: readonly HubRecord[] }) {
  const answered = answeredPrompts(records);
  if (!answered.length) {
    return null;
  }
  return (
    <ExecutionSectionCard title="Prompts">
      <SvelteMount component={PromptAnswers} props={{ answered }} />
    </ExecutionSectionCard>
  );
}
