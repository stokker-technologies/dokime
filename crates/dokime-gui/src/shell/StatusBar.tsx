// Dokime — SPDX-License-Identifier: Apache-2.0
import { useNavigate } from "@tanstack/react-router";
import { useNativeSession } from "../app/native-session/context";
import { SvelteMount } from "../components/SvelteMount";
import { useRecordsQuery } from "../features/cycle/server-state";
import { usePendingPromptsQuery } from "../features/cycles-home/cycles-queries";
import { ActiveCyclePicker } from "../features/library/ActiveCycle";
import { useActiveCycle } from "../features/library/use-active-cycle";
import { useInstalledPluginsQuery } from "../features/settings/installed-plugins/installed-plugin-queries";
import { useHubStore } from "../stores/hub-store";
import { useUiStore } from "../stores/ui-store";
import { useBackendStatus } from "./backend-status";
import { consoleAttentionCount } from "./console-view-model";
import { useRouteCycleId } from "./route-scope";
import StatusBarView, { type AttentionEntry, type StatusBarProps } from "./StatusBar.svelte";

export function StatusBar() {
  const session = useNativeSession();
  const navigate = useNavigate();
  const active = useActiveCycle();
  const status = useBackendStatus();
  const operations = useHubStore((state) => state.operations);
  const drawer = useUiStore((state) => state.console);
  const setConsoleState = useUiStore((state) => state.setConsoleState);
  const cycleId = useRouteCycleId();
  // The Console And Its Badge Share The Cycle's Record Tail.
  const records = useRecordsQuery(cycleId !== null, cycleId ? { cycleId } : null);
  const unseen = cycleId ? consoleAttentionCount(records.data ?? [], drawer.lastOpenedAt) : 0;
  const prompts = usePendingPromptsQuery();
  const artifacts = useInstalledPluginsQuery();

  const attention: AttentionEntry[] = [
    ...(prompts.data ?? []).map((prompt) => ({
      detail: prompt.message?.trim() || "Waiting for an operator response.",
      id: `prompt:${prompt.correlation_id}`,
      label: prompt.title,
      open: () => {
        const cycleId = prompt.scope?.cycleId;
        void (cycleId ? navigate({ params: { cycleId }, to: "/cycles/$cycleId" }) : navigate({ to: "/cycles" }));
      },
    })),
    ...(artifacts.data ?? [])
      .filter((artifact) => artifact.status === "failed" || artifact.error)
      .map((artifact) => ({
        detail: "Open Plugin Settings",
        id: `plugin:${artifact.plugin_id}`,
        label: artifact.label,
        open: () => {
          void navigate({ params: { pluginId: artifact.plugin_id }, to: "/settings/plugins/$pluginId" });
        },
      })),
  ];

  const view: StatusBarProps = {
    session,
    status,
    operations,
    attention,
    cycle: active.cycle,
    hasOpenCycles: active.cycles.length > 0,
    onOpenCyclePicker: active.openPicker,
    consoleOpen: drawer.open,
    unseen,
    onConsole: () => setConsoleState(drawer.open ? { open: false } : { lastOpenedAt: Date.now(), open: true }),
    onBackends: () => void navigate({ params: { section: "backends" }, to: "/settings/$section" }),
    onRunNavigate: (destination) => void navigate(destination),
  };

  return (
    <>
      <SvelteMount component={StatusBarView} props={view} />
      <ActiveCyclePicker />
    </>
  );
}
