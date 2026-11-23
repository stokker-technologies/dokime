<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { onMount } from "svelte";
  import Button from "../components/ui/Button.svelte";
  import CycleWorkspace from "../features/cycle/CycleWorkspace.svelte";
  import type { CycleSelection } from "../features/cycle/cycle-workspace-model";
  import CyclesHome from "../features/cycles-home/CyclesHome.svelte";
  import { cycleRows, promptCountsByCycle } from "../features/cycles-home/cycles-model";
  import { cyclesQueryOptions, pendingPromptsQueryOptions } from "../features/cycles-home/cycles-query-options";
  import NewCycleDialog from "../features/cycles-home/NewCycleDialog.svelte";
  import type { ExploreTarget } from "../features/explore/explore-contracts";
  import { exploreSearchSchema, searchFromSelectedIds, selectedIdsFromSearch, toggleSelectedId } from "../features/explore/explore-search";
  import ExploreSurface from "../features/explore/ExploreSurface.svelte";
  import HistoryDetailSurface from "../features/explore/HistoryDetailSurface.svelte";
  import { openCycles, resolveActiveCycle } from "../features/library/active-cycle-model";
  import LibrarySurface from "../features/library/LibrarySurface.svelte";
  import type { ActiveCycleState } from "../features/library/active-cycle-model";
  import NotificationsSurface from "../features/notifications/NotificationsSurface.svelte";
  import { installedPluginsListOptions } from "../features/settings/installed-plugins/installed-plugin-queries";
  import { pluginsListQueryOptions, pluginStartFailure, startNotification } from "../features/settings/plugins/plugin-queries";
  import PluginSettingsSurface from "../features/settings/PluginSettingsSurface.svelte";
  import SettingsSurface from "../features/settings/SettingsSurface.svelte";
  import { saveNativeDocument } from "../lib/api/native-document-delivery";
  import { statusGet } from "../lib/host/client";
  import { failureOf } from "../lib/notification-details";
  import { clearPluginNotifications, toast } from "../lib/notifications";
  import type { DensityPreference } from "../lib/theme";
  import type { BackendStatus } from "../shell/backend-status";
  import DesktopShell, { type DesktopShellProps } from "../shell/DesktopShell.svelte";
  import { navigationMode, settingsParent, sidebarFrameWidth, sidebarWidth } from "../shell/navigation-model";
  import { pageContentClass } from "../shell/page-layout";
  import { cycleIdOfPathname } from "../shell/route-scope";
  import type { AttentionEntry } from "../shell/StatusBar.svelte";
  import { hubStore } from "../stores/hub-store";
  import { storeState } from "../stores/store-state";
  import { uiStore } from "../stores/ui-store";
  import type { Presentation } from "./DesktopPresentation.svelte";
  import {
    cyclePath,
    cycleSelection,
    focusMode,
    historyPath,
    libraryPath,
    navigate,
    optionalSearch,
    pathParts,
    pluginPath,
    readLocation,
  } from "./desktop-router.svelte";
  import type { ConnectedSession, NativeSession } from "./native-session/session";

  let { owner, session, presentation }: { owner: NativeSession; session: ConnectedSession; presentation: Presentation } = $props();
  const location = $derived(readLocation());
  const parts = $derived(pathParts(location));
  const pathname = $derived(location.pathname.replace(/\/$/, "") || "/");
  const category = $derived(parts[0]);
  const selection = $derived(cycleSelection(location));
  const cycleId = $derived(parts.length === 2 ? (cycleIdOfPathname(pathname) ?? undefined) : undefined);
  const focus = $derived(focusMode(location, cycleId));
  const member = $derived(Boolean(selection.caseId || selection.executionId));
  const plugin = $derived(category === "settings" && parts[1] === "plugins" && parts.length === 3 ? parts[2] : undefined);
  const compare = $derived(category === "history" && parts[1] === "compare" && parts.length === 2);
  const historyTarget = $derived<ExploreTarget | null>(
    category === "history" && !compare
      ? parts.length === 2
        ? { kind: "cycle", id: parts[1] }
        : parts.length === 3 && parts[1] === "executions"
          ? { kind: "execution", id: parts[2] }
          : null
      : null,
  );

  let width = $state(1024);
  let density = $state<DensityPreference>("comfortable");
  let creating = $state(false);
  let lastOutsideSettings = $state<string | null>(null);
  const mode = $derived(navigationMode(width));
  const ui = storeState(uiStore, (state) => state);
  const hub = storeState(hubStore, (state) => state);
  const connection = $derived(hub().connection);
  const collapse = $derived(mode !== "desktop" || Boolean(ui().sidebarCollapsed[density]) || focus.focused);
  const parent = $derived(
    plugin
      ? "/settings/plugins"
      : category === "settings" && parts.length > 1
        ? mode === "phone"
          ? settingsParent(pathname)
          : (lastOutsideSettings ?? "/cycles")
        : cycleId
          ? member
            ? `${pathname}?view=cycle`
            : "/cycles"
          : historyTarget || compare
            ? "/history"
            : undefined,
  );

  const client = () => session.queryClient;
  const cycles = createQuery(() => cyclesQueryOptions(session.identity, {}), client);
  const prompts = createQuery(() => pendingPromptsQueryOptions(session.identity), client);
  const artifacts = createQuery(() => installedPluginsListOptions(session), client);
  const installed = createQuery(() => pluginsListQueryOptions(session), client);
  const backend = createQuery(
    () => ({
      queryFn: () => statusGet(session.identity, {}),
      queryKey: ["shell", "backend-identity", session.identity.generation],
      staleTime: Infinity,
    }),
    client,
  );
  const status = $derived<BackendStatus>({
    connection: connection === "connecting" ? "unknown" : connection,
    detail: hub().error,
    label: backend.data?.status?.identity?.applicationName || backend.data?.status?.identity?.hostId || null,
  });
  const availableCycles = $derived(openCycles(cycleRows(cycles.data ?? [], promptCountsByCycle(prompts.data ?? []))));
  const active = $derived<ActiveCycleState>({
    choose: (activeCycleId) => ui().setUiState({ activeCycleId }),
    cycle: resolveActiveCycle(ui().activeCycleId, availableCycles),
    cycles: availableCycles,
    failed: cycles.isError,
    error: cycles.error,
    loading: cycles.isPending,
    openPicker: () => ui().setUiState({ activeCyclePickerOpen: true }),
  });
  const attention = $derived<AttentionEntry[]>([
    ...(prompts.data ?? []).map((prompt) => ({
      detail: prompt.message?.trim() || "Waiting for an operator response.",
      id: `prompt:${prompt.correlation_id}`,
      label: prompt.title,
      open: () => navigate(prompt.scope?.cycleId ? cyclePath(prompt.scope.cycleId) : "/cycles"),
    })),
    ...(artifacts.data ?? [])
      .filter((item) => item.status === "failed" || item.error)
      .map((item) => ({
        detail: "Open Plugin Settings",
        id: `plugin:${item.plugin_id}`,
        label: item.label,
        open: () => openPlugin(item.plugin_id),
      })),
  ]);
  const seen = new Set<string>();
  $effect(() => {
    const entries = installed.data;
    if (!entries || !session.identity.isCurrent()) {
      return;
    }
    for (const entry of entries) {
      if (!startNotification(entry)) {
        continue;
      }
      const failure = pluginStartFailure(entry, session.identity.generation);
      if (seen.has(failure.id)) {
        continue;
      }
      seen.add(failure.id);
      toast.error(failure, {
        context: "background",
        description: entry.failure?.message,
        source: entry.description?.id || entry.name,
        pluginSettings: entry.name,
        action: { label: "Open Plugin Settings", onClick: () => openPlugin(entry.name) },
      });
    }
    void clearPluginNotifications(session.identity.generation, new Set(entries.map((entry) => entry.name))).catch((error: unknown) =>
      toast.error(failureOf(error, "Notification History Unavailable"), { context: "background" }),
    );
  });

  onMount(() => {
    const root = document.documentElement;
    const update = () => {
      const value = root.dataset.density;
      density = value === "touch" || value === "compact" ? value : "comfortable";
    };
    update();
    const observer = new MutationObserver(update);
    observer.observe(root, { attributeFilter: ["data-density"] });
    return () => observer.disconnect();
  });
  $effect(() => {
    if (pathname === "/") {
      navigate("/cycles", undefined, true);
    } else if (category !== "settings") {
      lastOutsideSettings = pathname;
    }
  });
  $effect(() => {
    if (category === "cycles" && parts.length === 1 && location.hash === "#active" && cycles.data) {
      document.getElementById("active")?.scrollIntoView();
    }
  });

  const selectedIds = $derived(selectedIdsFromSearch(exploreSearchSchema.parse({ selected: location.searchParams.getAll("selected") })));
  const title = $derived(
    category === "cycles"
      ? "Cycles"
      : category === "library"
        ? "Library"
        : category === "history"
          ? compare
            ? "History › Compare"
            : "History"
          : category === "settings"
            ? "Settings"
            : category === "notifications"
              ? "Notifications"
              : "Not Found",
  );
  const shell = $derived<DesktopShellProps>({
    ...presentation,
    session,
    pathname,
    selection,
    focus,
    title,
    status,
    attention,
    active,
    sidebar: {
      mode,
      collapsed: collapse,
      width: sidebarFrameWidth(mode, collapse, ui().sidebarWidth),
      toggle: () => ui().setUiState({ sidebarCollapsed: { ...ui().sidebarCollapsed, [density]: !collapse } }),
      resize: (width) => ui().setUiState({ sidebarWidth: sidebarWidth(width) }),
    },
    actions: category === "cycles" && !cycleId ? newCycle : undefined,
    operations: hub().operations,
    back: parent
      ? () => {
          if (parent) navigate(parent);
        }
      : undefined,
    onDisconnect: () => void owner.disconnect(),
    navigate,
    openCycle,
    selectCycleMember,
    addCases,
    openSettings,
    openRun: (destination) => {
      if ("hash" in destination) navigate("/cycles#active");
      else navigate(cyclePath(destination.params.cycleId), destination.search);
    },
  });

  function openCycle(id: string) {
    navigate(cyclePath(id));
  }
  function addCases(id: string) {
    navigate(libraryPath(id));
  }
  function openPlugin(id: string) {
    navigate(pluginPath(id));
  }
  function openSettings(section: string) {
    navigate(`/settings/${encodeURIComponent(section)}`);
  }
  function selectCycleMember(id: string, next: CycleSelection) {
    const retained = cycleId === id ? location.search : "";
    navigate(cyclePath(id) + retained, next, true);
  }
  function openHistory(target: ExploreTarget) {
    navigate(historyPath(target), searchFromSelectedIds(selectedIds));
  }
  function toggleSelected(id: string) {
    navigate(compare ? "/history/compare" : "/history", searchFromSelectedIds(toggleSelectedId(selectedIds, id)), true);
  }
  async function saveTraceability(json: string, filename: string) {
    await saveNativeDocument(session.identity, { kind: "json_document", filename, bytes: new TextEncoder().encode(json) });
  }
</script>

<svelte:window bind:innerWidth={width} />

{#snippet newCycle()}
  <Button onclick={() => (creating = true)}>New Cycle</Button>
{/snippet}

<DesktopShell {...shell}>
  {#if cycleId}
    {#key cycleId}
      <CycleWorkspace
        {cycleId}
        {focus}
        {selection}
        {openCycle}
        {addCases}
        context={{
          session,
          plotPalette: presentation.plotPalette,
          reportDefaults: presentation.reportDefaults,
          traceabilitySave: saveTraceability,
        }}
        onSelectionChange={(next) => selectCycleMember(cycleId, next)}
      />
    {/key}
  {:else if category === "cycles" && parts.length === 1}
    <div class={pageContentClass}>
      <CyclesHome {session} onOpen={openCycle} />
      <NewCycleDialog {session} open={creating} onOpenChange={(open) => (creating = open)} onCreated={openCycle} />
    </div>
  {:else if category === "library" && parts.length === 1}
    <LibrarySurface
      {session}
      {active}
      label={presentation.brand.label}
      appendTo={optionalSearch(location, "appendTo")}
      {openCycle}
      {addCases}
    />
  {:else if historyTarget}
    {#key pathname}
      <HistoryDetailSurface {session} target={historyTarget} onOpenCycle={(id) => openHistory({ kind: "cycle", id })} />
    {/key}
  {:else if category === "history" && (parts.length === 1 || compare)}
    {#key compare}
      <div class={pageContentClass}>
        <ExploreSurface
          {session}
          comparing={compare}
          {selectedIds}
          onToggleSelected={toggleSelected}
          onOpenEntry={openHistory}
          onCompare={() => navigate("/history/compare", searchFromSelectedIds(selectedIds))}
        />
      </div>
    {/key}
  {:else if plugin}
    {#key plugin}
      <PluginSettingsSurface pluginId={plugin} {session} label={presentation.brand.label} />
    {/key}
  {:else if category === "settings" && parts.length <= 2}
    <SettingsSurface
      {owner}
      {session}
      {mode}
      selected={parts[1]}
      label={presentation.brand.label}
      hash={location.hash.slice(1)}
      clearHash={() => navigate(location.pathname + location.search, undefined, true)}
      choose={openSettings}
      openPluginSettings={openPlugin}
    />
  {:else if category === "notifications" && parts.length === 1}
    <div class={pageContentClass}><NotificationsSurface onNavigate={navigate} /></div>
  {:else if category}
    <div class={pageContentClass}><Button variant="outline" onclick={() => navigate("/cycles")}>Open Cycles</Button></div>
  {/if}
</DesktopShell>
