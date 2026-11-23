<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import type { Snippet } from "svelte";
  import CycleSidebarCasesView from "../features/cycle/CycleSidebarCasesView.svelte";
  import type { CycleSelection } from "../features/cycle/cycle-workspace-model";
  import type { CycleSnapshotSource } from "../features/cycle/snapshot-source";
  import type { Brand } from "../lib/brand";
  import AppShellLayout from "../shell/AppShellLayout.svelte";
  import { navigationMode, sidebarFrameWidth, sidebarWidth } from "../shell/navigation-model";
  import PanelHeader from "../shell/PanelHeader.svelte";
  import SidebarFrame from "../shell/SidebarFrame.svelte";

  type Props = {
    brand: Brand;
    children?: Snippet;
    selection: CycleSelection;
    source: CycleSnapshotSource;
    onSelect?: (selection: CycleSelection) => void;
  };

  let { brand, children, selection, source, onSelect }: Props = $props();
  let viewportWidth = $state(1024);
  let collapsed = $state(false);
  let width = $state(256);
  const mode = $derived(navigationMode(viewportWidth));
  const shownCollapsed = $derived(mode !== "desktop" || collapsed);
  const title = $derived(frozenReportTitle(source.generatedAt));
  const back = $derived(onSelect && (selection.caseId || selection.executionId) ? () => onSelect?.({}) : undefined);

  function frozenReportTitle(generatedAt: string | null): string {
    const at = generatedAt ? new Date(generatedAt) : null;
    return at && !Number.isNaN(at.getTime()) ? `Frozen Report · Generated ${at.toLocaleString()}` : "Frozen Report";
  }
</script>

<svelte:window bind:innerWidth={viewportWidth} />

{#snippet sidebar()}
  {#if onSelect}
    <SidebarFrame
      {brand}
      {mode}
      collapsed={shownCollapsed}
      width={sidebarFrameWidth(mode, shownCollapsed, width)}
      toggle={() => {
        collapsed = !collapsed;
      }}
      resize={(next) => {
        width = sidebarWidth(next);
      }}
    >
      <ul class="grid min-h-0 content-start gap-1 overflow-y-auto px-[var(--shell-gap)] pt-2 pb-2">
        <CycleSidebarCasesView
          {selection}
          detailQuery={{ data: { plan: source.plan, executions: source.executions }, isError: false, error: null }}
          live={false}
          {onSelect}
        />
      </ul>
    </SidebarFrame>
  {/if}
{/snippet}

<AppShellLayout {sidebar} {children}>
  {#snippet header()}
    <PanelHeader {title} onBack={back} />
  {/snippet}
</AppShellLayout>
