<!-- Dokime — SPDX-License-Identifier: Apache-2.0 -->
<script module lang="ts">
  import type { Snippet } from "svelte";
  import type { ConnectedSession } from "../app/native-session/session";
  import type { CycleSelection } from "../features/cycle/cycle-workspace-model";
  import type { ActiveCycleState } from "../features/library/active-cycle-model";
  import type { Brand } from "../lib/brand";
  import type { DensityPreference, ThemeModePreference } from "../lib/theme";
  import type { BackendStatus } from "./backend-status";
  import type { FocusMode } from "./focus-mode";
  import type { runDestination } from "./run-focus";
  import type { ShellDestination } from "./Sidebar.svelte";
  import type { BackendOperation } from "./status-bar-operations";
  import type { AttentionEntry } from "./StatusBar.svelte";
  import type { SidebarState } from "./navigation-model";

  export type DesktopShellProps = {
    session: ConnectedSession;
    brand: Brand;
    sidebar: SidebarState;
    pathname: string;
    selection: CycleSelection;
    focus: FocusMode;
    title: string;
    back?: () => void;
    actions?: Snippet;
    children?: Snippet;
    status: BackendStatus;
    operations: readonly BackendOperation[];
    active: ActiveCycleState;
    attention: AttentionEntry[];
    desktop: boolean;
    chooseTheme: (next: ThemeModePreference) => void;
    theme: ThemeModePreference;
    themeDisabled: boolean;
    chooseDensity: (next: DensityPreference) => void;
    densityDisabled: boolean;
    onDisconnect: () => void;
    navigate: (to: ShellDestination) => void;
    openCycle: (cycleId: string) => void;
    selectCycleMember: (cycleId: string, next: CycleSelection) => void;
    addCases: (cycleId: string) => void;
    openSettings: (section: string) => void;
    openRun: (destination: NonNullable<ReturnType<typeof runDestination>>) => void;
  };
</script>

<script lang="ts">
  import { observeRuntime } from "../app/live/runtime-stream";
  import { createRecordsQuery } from "../features/cycle/live-document.svelte";
  import NewCycleDialog from "../features/cycles-home/NewCycleDialog.svelte";
  import ActiveCyclePicker from "../features/library/ActiveCyclePicker.svelte";
  import { failureOf } from "../lib/notification-details";
  import { toast } from "../lib/notifications";
  import { storeState } from "../stores/store-state";
  import { uiStore } from "../stores/ui-store";
  import AppShellLayout from "./AppShellLayout.svelte";
  import ConsoleDrawer from "./ConsoleDrawer.svelte";
  import { consoleAttentionCount } from "./console-view-model";
  import MenuBar from "./MenuBar.svelte";
  import { pageActionsSlot, pageTitle } from "./page-chrome.svelte";
  import PanelHeader from "./PanelHeader.svelte";
  import { cycleIdOfPathname } from "./route-scope";
  import Sidebar from "./Sidebar.svelte";
  import StatusBar from "./StatusBar.svelte";

  let props: DesktopShellProps = $props();
  let creating = $state(false);
  const session = $derived(props.session);
  const cycleId = $derived(cycleIdOfPathname(props.pathname));
  const executionId = $derived(props.selection.executionId ?? null);
  const records = createRecordsQuery(
    () => session,
    () => ({ enabled: Boolean(cycleId), scope: cycleId ? { cycleId } : null }),
  );
  const readDrawer = storeState(uiStore, (state) => state.console);
  const readDensity = storeState(uiStore, (state) => state.density);
  const { setConsoleState } = uiStore.getState();
  const drawer = $derived(readDrawer());
  const unseen = $derived(cycleId ? consoleAttentionCount(records.data ?? [], drawer.lastOpenedAt) : 0);

  $effect(() => observeRuntime(session));
  $effect(() => {
    if (props.focus.focused) {
      document.documentElement.dataset.focus = "on";
      return () => {
        delete document.documentElement.dataset.focus;
      };
    }
  });

  function leaveFocus(event: KeyboardEvent) {
    if (props.focus.focused && event.key === "Escape" && !event.defaultPrevented) {
      event.preventDefault();
      props.focus.setFocused(false);
    }
  }

  function setConsoleOpen(open: boolean) {
    setConsoleState(open ? { lastOpenedAt: Date.now(), open: true } : { open: false });
  }

  async function closeDesktopWindow() {
    try {
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      await getCurrentWindow().close();
    } catch (error: unknown) {
      toast.error(failureOf(error, "The Window Could Not Be Closed"));
    }
  }
</script>

<svelte:window onkeydown={leaveFocus} />
<AppShellLayout>
  {#snippet header()}
    <PanelHeader
      title={props.focus.focused ? "" : (pageTitle() ?? props.title)}
      onBack={props.focus.focused ? undefined : props.back}
    >
      {#snippet actions()}
        {@render props.actions?.()}
        <div class="contents" {@attach pageActionsSlot}></div>
      {/snippet}
    </PanelHeader>
  {/snippet}
  {#snippet menuBar()}
    <MenuBar
      connected
      local={session.snapshot.ownership === "owned_local"}
      desktop={props.desktop}
      focused={props.focus.focused}
      focusDisabled={cycleId === null}
      consoleOpen={drawer.open}
      density={readDensity()}
      densityDisabled={props.densityDisabled}
      theme={props.theme}
      themeDisabled={props.themeDisabled}
      newCycle={() => (creating = true)}
      settings={() => props.navigate("/settings")}
      disconnect={props.onDisconnect}
      quit={() => void closeDesktopWindow()}
      toggleSidebar={props.sidebar.toggle}
      setFocused={props.focus.setFocused}
      {setConsoleOpen}
      setDensity={props.chooseDensity}
      setTheme={props.chooseTheme}
      diagnostics={() => props.openSettings("diagnostics")}
      about={() => props.openSettings("about")}
    />
  {/snippet}
  {#snippet sidebar()}
    <Sidebar
      {session}
      brand={props.brand}
      sidebar={props.sidebar}
      pathname={props.pathname}
      {cycleId}
      selection={props.selection}
      selectCycleMember={props.selectCycleMember}
      addCases={props.addCases}
      navigate={props.navigate}
      openCycle={props.openCycle}
      openSettings={props.openSettings}
    />
  {/snippet}
  {#snippet console()}
    <ConsoleDrawer identity={session.identity} {cycleId} {executionId} />
  {/snippet}
  {#snippet statusBar()}
    <StatusBar
      {session}
      status={props.status}
      operations={props.operations}
      attention={props.attention}
      cycle={props.active.cycle}
      hasOpenCycles={props.active.cycles.length > 0}
      onOpenCyclePicker={props.active.openPicker}
      consoleOpen={drawer.open}
      {unseen}
      onConsole={() => setConsoleOpen(!drawer.open)}
      onBackends={() => props.openSettings("backends")}
      onRunNavigate={props.openRun}
    />
  {/snippet}
  {@render props.children?.()}
</AppShellLayout>

<ActiveCyclePicker {session} active={props.active} />
<NewCycleDialog {session} open={creating} onOpenChange={(open) => (creating = open)} onCreated={props.openCycle} />
