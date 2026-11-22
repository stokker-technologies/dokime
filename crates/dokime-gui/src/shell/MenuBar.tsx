// Dokime — SPDX-License-Identifier: Apache-2.0
import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { useNativeSessionController, useNativeSessionOptional } from "@/app/native-session/context";
import { SvelteMount } from "@/components/SvelteMount";
import { NewCycleDialog } from "@/features/cycles-home/NewCycleDialog";
import { useAppearance } from "@/lib/appearance";
import { useApplicationSettings, useSaveAppearance } from "@/lib/application-settings";
import { useApplicationTheme } from "@/lib/application-theme";
import { failureOf } from "@/lib/notification-details";
import { toast } from "@/lib/notifications";
import { isTauriDesktop } from "@/lib/surface-environment";
import type { DensityPreference, ThemeModePreference } from "@/lib/theme";
import { useUiStore } from "@/stores/ui-store";
import { useFocusMode } from "./focus-mode";
import ApplicationMenu from "./MenuBar.svelte";
import { useRouteCycleId } from "./route-scope";
import { useSidebarState } from "./use-sidebar-state";

export function MenuBar() {
  const navigate = useNavigate();
  const session = useNativeSessionOptional();
  const controller = useNativeSessionController();
  const sidebar = useSidebarState();
  const { focused, setFocused } = useFocusMode();
  const cycleId = useRouteCycleId();
  const consoleOpen = useUiStore((state) => state.console.open);
  const setConsoleState = useUiStore((state) => state.setConsoleState);
  const density = useUiStore((state) => state.density);
  const setUiState = useUiStore((state) => state.setUiState);
  const appearance = useAppearance();
  const theme = useApplicationTheme();
  const settings = useApplicationSettings();
  const save = useSaveAppearance();
  const desktop = isTauriDesktop();
  const [creating, setCreating] = useState(false);

  function setDensity(next: DensityPreference) {
    if (!desktop) {
      setUiState({ density: next });
      return;
    }
    save.mutate({ density: next }, { onError: (error) => toast.error(failureOf(error, "Appearance Could Not Be Saved")) });
  }

  return (
    <>
      <SvelteMount
        component={ApplicationMenu}
        props={{
          connected: Boolean(session),
          local: session?.snapshot.ownership === "owned_local",
          desktop,
          focused,
          focusDisabled: cycleId === null,
          consoleOpen,
          density,
          densityDisabled: desktop && (!settings.data || save.isPending),
          theme: (theme.theme as ThemeModePreference | undefined) ?? "system",
          themeDisabled: Boolean(theme.saving || appearance.mode_locked || theme.forcedTheme),
          newCycle: () => setCreating(true),
          settings: () => void navigate({ to: "/settings" }),
          disconnect: () => void controller.disconnect(),
          quit: () => void closeDesktopWindow(),
          toggleSidebar: sidebar.toggle,
          setFocused,
          setConsoleOpen: (open) => setConsoleState(open ? { lastOpenedAt: Date.now(), open: true } : { open: false }),
          setDensity,
          setTheme: theme.setTheme,
          diagnostics: () => void navigate({ params: { section: "diagnostics" }, to: "/settings/$section" }),
          about: () => void navigate({ params: { section: "about" }, to: "/settings/$section" }),
        }}
      />
      {session ? (
        <NewCycleDialog
          onCreated={(cycleId) => void navigate({ params: { cycleId }, to: "/cycles/$cycleId" })}
          onOpenChange={setCreating}
          open={creating}
        />
      ) : null}
    </>
  );
}

/** Quit closes the one main window; the desktop shell exits with it. */
async function closeDesktopWindow(): Promise<void> {
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().close();
  } catch (error) {
    toast.error(failureOf(error, "The Window Could Not Be Closed"));
  }
}
