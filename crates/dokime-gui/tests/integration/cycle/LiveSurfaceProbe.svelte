<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { EvidenceScope } from "@generated/host/model";
  import type { ConnectedSession } from "@/app/native-session/session";
  import { createScopedDocumentQuery } from "@/features/cycle/document-query.svelte";
  import { createLiveDocumentSurface } from "@/features/cycle/live-document.svelte";
  import LiveCycleDocument, { type LiveCycleContext } from "@/features/cycle/LiveCycleDocument.svelte";
  import type { PlotWindow } from "@/features/cycle/plot-window-state";
  import type { Wire } from "@/lib/rpc/canonical";
  import type { HubLiveState, TestExecution } from "@/lib/types";

  // A Desktop Context Also Draws The Document.
  let { session, plotWindow, desktop }: { session: ConnectedSession; plotWindow: PlotWindow; desktop?: LiveCycleContext } = $props();
  let snapshot = $state.raw<HubLiveState | null>(null);
  let execution = $state.raw<TestExecution | null>(null);
  let scope = $state.raw<Wire<EvidenceScope>>({ cycleId: "cycle", executionId: "execution" });
  const query = createScopedDocumentQuery(
    () => session,
    () => ({ cycleId: scope.cycleId ?? null, executionId: scope.executionId ?? null, enabled: true }),
  );
  const surface = createLiveDocumentSurface(
    () => session,
    {
      get document() {
        return query.data ?? null;
      },
      get execution() {
        return execution;
      },
      live: true,
      get snapshot() {
        return snapshot;
      },
      get scope() {
        return scope;
      },
      parameters: { case: undefined, execution: undefined },
      noteElapsedStart: null,
      get plotWindow() {
        return plotWindow;
      },
    },
    {
      fetch: async () => ({ records: [], cursor: null }),
      openStream: async () => ({ cancel: async () => {}, async *[Symbol.asyncIterator]() {} }),
    },
  );

  export function setSnapshot(next: HubLiveState) {
    snapshot = next;
  }
  export function setExecution(next: TestExecution) {
    execution = next;
  }
  export function setScope(next: Wire<EvidenceScope>) {
    scope = next;
  }
  export function streams() {
    return surface.view.streams;
  }
  export function sections() {
    return surface.view.sections;
  }
</script>

{#if desktop}
  <LiveCycleDocument context={desktop} {surface} />
{/if}
