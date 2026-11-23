// Dokime — SPDX-License-Identifier: Apache-2.0
import { useNavigate, useRouter, useRouterState } from "@tanstack/react-router";
import { type ReactNode, useRef } from "react";
import { useNativeSession, useNativeSessionController } from "@/app/native-session/context";
import { SvelteMount, useSvelteSlots } from "@/components/SvelteMount";
import type { CycleSelection } from "@/features/cycle/cycle-workspace-model";
import { usePendingPromptsQuery } from "@/features/cycles-home/cycles-queries";
import { useActiveCycle } from "@/features/library/use-active-cycle";
import { useInstalledPluginsQuery } from "@/features/settings/installed-plugins/installed-plugin-queries";
import { PluginStartNotifications } from "@/features/settings/plugins/plugin-queries";
import { useAppearance } from "@/lib/appearance";
import { useApplicationSettings, useSaveAppearance } from "@/lib/application-settings";
import { useApplicationTheme } from "@/lib/application-theme";
import { useBrand } from "@/lib/brand-context";
import { failureOf } from "@/lib/notification-details";
import { toast } from "@/lib/notifications";
import { isTauriDesktop } from "@/lib/surface-environment";
import type { DensityPreference, ThemeModePreference } from "@/lib/theme";
import { useHubStore } from "@/stores/hub-store";
import { useUiStore } from "@/stores/ui-store";
import { useBackendStatus } from "./backend-status";
import DesktopShell, { type DesktopShellProps } from "./DesktopShell.svelte";
import { useFocusMode } from "./focus-mode";
import { type NavigationMode, settingsParent } from "./navigation-model";
import { PageChromeProvider, usePageChromeState } from "./page-chrome";
import type { AttentionEntry } from "./StatusBar.svelte";
import { useSidebarState } from "./use-sidebar-state";

const TOP_LEVEL = new Set(["/cycles", "/library", "/history", "/settings"]);

export function AppShell({ children }: { children: ReactNode }) {
  return (
    <PageChromeProvider>
      <ConnectedShell>{children}</ConnectedShell>
    </PageChromeProvider>
  );
}

function ConnectedShell({ children }: { children: ReactNode }) {
  const navigate = useNavigate();
  const router = useRouter();
  const { pathname, search } = useRouterState({ select: (state) => state.location });
  const selection: CycleSelection = search;
  const session = useNativeSession();
  const owner = useNativeSessionController();
  const brand = useBrand();
  const focus = useFocusMode();
  const sidebar = useSidebarState();
  const parent = useParentRoute(pathname, Boolean(selection.caseId || selection.executionId), sidebar.mode);
  const chrome = usePageChromeState();
  const status = useBackendStatus();
  const active = useActiveCycle();
  const operations = useHubStore((state) => state.operations);
  const prompts = usePendingPromptsQuery();
  const artifacts = useInstalledPluginsQuery();
  const appearance = useAppearance();
  const theme = useApplicationTheme();
  const settings = useApplicationSettings();
  const save = useSaveAppearance();
  const desktop = isTauriDesktop();
  const { rendered, portals } = useSvelteSlots({
    actions: chrome.actions,
    overflow: chrome.overflow,
    children: (
      <>
        <PluginStartNotifications />
        {children}
      </>
    ),
  });

  const attention: AttentionEntry[] = [
    ...(prompts.data ?? []).map((prompt) => ({
      detail: prompt.message?.trim() || "Waiting for an operator response.",
      id: `prompt:${prompt.correlation_id}`,
      label: prompt.title,
      open: () => {
        const cycleId = prompt.scope?.cycleId;
        if (cycleId) {
          openCycle(cycleId);
        } else {
          void navigate({ to: "/cycles" });
        }
      },
    })),
    ...(artifacts.data ?? [])
      .filter((artifact) => artifact.status === "failed" || artifact.error)
      .map((artifact) => ({
        detail: "Open Plugin Settings",
        id: `plugin:${artifact.plugin_id}`,
        label: artifact.label,
        open: () => void navigate({ params: { pluginId: artifact.plugin_id }, to: "/settings/plugins/$pluginId" }),
      })),
  ];

  function openCycle(id: string) {
    void navigate({ to: "/cycles/$cycleId", params: { cycleId: id } });
  }

  function chooseDensity(next: DensityPreference) {
    if (!desktop) {
      useUiStore.getState().setUiState({ density: next });
      return;
    }
    save.mutate({ density: next }, { onError: (error) => toast.error(failureOf(error, "Appearance Could Not Be Saved")) });
  }

  const shell: DesktopShellProps = {
    ...rendered,
    session,
    brand,
    sidebar,
    pathname,
    selection,
    focus,
    title: chrome.title,
    back: parent ? () => router.history.push(parent) : undefined,
    status,
    operations,
    active,
    attention,
    desktop,
    chooseTheme: theme.setTheme,
    theme: (theme.theme as ThemeModePreference | undefined) ?? "system",
    themeDisabled: Boolean(theme.saving || appearance.mode_locked || theme.forcedTheme),
    chooseDensity,
    densityDisabled: desktop && (!settings.data || save.isPending),
    onDisconnect: () => void owner.disconnect(),
    navigate: (to) => void navigate({ to }),
    openCycle,
    selectCycleMember: (cycleId, next) =>
      void navigate({
        params: { cycleId },
        replace: true,
        search: (current: Record<string, unknown>) => ({
          ...current,
          caseId: next.caseId,
          executionId: next.executionId,
          view: next.view,
        }),
        to: "/cycles/$cycleId",
      }),
    addCases: (cycleId) => void navigate({ search: { appendTo: cycleId }, to: "/library" }),
    openSettings: (section) => void navigate({ params: { section }, to: "/settings/$section" }),
    openRun: (destination) => void navigate(destination),
  };
  return (
    <>
      <SvelteMount component={DesktopShell} props={shell} />
      {portals}
    </>
  );
}

/**
 * The panel the Back chevron leaves to, or null on a top-level destination.
 *
 * A settings section has no structural parent — the operator came to it from
 * whatever they were doing — so it goes back to the last route outside
 * Settings, and to Cycles when this session has none.
 */
function useParentRoute(pathname: string, member: boolean, mode: NavigationMode): string | null {
  const lastOutsideSettings = useRef<string | null>(null);

  const inSettings = pathname === "/settings" || pathname.startsWith("/settings/");
  if (!inSettings) {
    lastOutsideSettings.current = pathname;
  }

  if (TOP_LEVEL.has(pathname)) {
    return null;
  }
  if (/^\/settings\/plugins\/.+/.test(pathname)) {
    return "/settings/plugins";
  }
  if (inSettings) {
    return mode === "phone" ? settingsParent(pathname) : (lastOutsideSettings.current ?? "/cycles");
  }
  // An execution page goes back to its cycle; the cycle page to Cycles.
  if (pathname.startsWith("/cycles/")) {
    return member ? `${pathname}?view=cycle` : "/cycles";
  }
  if (pathname.startsWith("/history/")) {
    return "/history";
  }
  if (pathname.startsWith("/library/")) {
    return "/library";
  }
  return null;
}
