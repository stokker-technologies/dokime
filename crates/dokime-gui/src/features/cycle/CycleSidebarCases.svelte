<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createMutation, createQuery } from "@tanstack/svelte-query";
  import type { ConnectedSession } from "../../app/native-session/session";
  import { storeState } from "../../stores/store-state";
  import { useSelectionStore } from "../../stores/selection-store";
  import { appendCaseAttemptOptions, cycleDetailQueryOptions, cyclePromptQueryOptions } from "./cycle-queries";
  import type { CycleSelection } from "./cycle-workspace-model";
  import CycleSidebarCasesView from "./CycleSidebarCasesView.svelte";

  let {
    session,
    cycleId,
    selection,
    select,
    addCases,
  }: {
    session: Pick<ConnectedSession, "identity" | "queryClient">;
    cycleId: string;
    selection: CycleSelection;
    select: (next: CycleSelection) => void;
    addCases: () => void;
  } = $props();
  const client = () => session.queryClient;
  const detailQuery = createQuery(() => cycleDetailQueryOptions(session.identity, cycleId), client);
  const prompts = createQuery(() => cyclePromptQueryOptions(session.identity, cycleId), client);
  const append = createMutation(() => {
    const { identity, queryClient } = session;
    return appendCaseAttemptOptions(() => identity, queryClient, cycleId);
  }, client);
  const follow = storeState(useSelectionStore, (state) => state.followActive);
  const inputNeeded = $derived(
    new Set((prompts.data ?? []).flatMap((prompt) => (prompt.scope?.executionId ? [prompt.scope.executionId] : []))),
  );
</script>

<CycleSidebarCasesView
  {detailQuery}
  {inputNeeded}
  {selection}
  followActive={follow()}
  live
  onSelect={select}
  actions={{ addCases, rerunCase: append.mutateAsync }}
/>
