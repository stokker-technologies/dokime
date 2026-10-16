/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The application menu bar: the one thin row above the sidebar and page bar.
 *
 * Application-level commands live here so the page bar holds only the page:
 * its title and the actions the page publishes. A frozen report has no menu
 * bar: a file has no application to command.
 */

/* === External Imports === */

import { useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { toast } from "sonner";

/* === Local Imports === */

import {
  useNativeSessionController,
  useNativeSessionOptional,
} from "@/app/native-session/NativeSessionProvider";
import {
  Menubar,
  MenubarCheckboxItem,
  MenubarContent,
  MenubarItem,
  MenubarLabel,
  MenubarMenu,
  MenubarRadioGroup,
  MenubarRadioItem,
  MenubarSeparator,
  MenubarTrigger,
} from "@/components/ui/menubar";
import { NewCycleDialog } from "@/features/cycles-home/NewCycleDialog";
import { useAppearance } from "@/lib/appearance";
import {
  applicationSettingsError,
  useApplicationSettings,
  useSaveAppearance,
} from "@/lib/application-settings";
import { useApplicationTheme } from "@/lib/application-theme";
import { isTauriDesktop } from "@/lib/desktop-open";
import type { DensityPreference, ThemeModePreference } from "@/lib/theme";
import { useFocusMode } from "@/shell/focus-mode";
import { useRouteCycleId } from "@/shell/route-scope";
import { useSidebarState } from "@/shell/use-sidebar-state";
import { useUiStore } from "@/stores/ui-store";

/* === Constants === */

const DENSITY_OPTIONS: { value: DensityPreference; label: string }[] = [
  { value: "comfortable", label: "Comfortable" },
  { value: "compact", label: "Compact" },
  { value: "touch", label: "Touch" },
];

const THEME_OPTIONS: { value: ThemeModePreference; label: string }[] = [
  { value: "system", label: "System" },
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
];

/* === Components === */

/** The menu bar row. The live shell is the only one that has it. */
export function MenuBar() {
  return (
    <Menubar
      aria-label="Application"
      className="h-[var(--menu-bar-height)] gap-0 rounded-none border-0 border-b bg-background p-0 px-[calc(var(--shell-inset)/2)]"
    >
      <LiveMenus />
    </Menubar>
  );
}

/* === Private Components === */

function LiveMenus() {
  const navigate = useNavigate();
  const session = useNativeSessionOptional();
  const controller = useNativeSessionController();
  const sidebar = useSidebarState();
  const { focused, setFocused } = useFocusMode();
  const cycleId = useRouteCycleId();
  const consoleOpen = useUiStore((state) => state.console.open);
  const setConsoleState = useUiStore((state) => state.setConsoleState);
  const desktop = isTauriDesktop();
  const [creating, setCreating] = useState(false);

  return (
    <>
      <MenubarMenu>
        <MenuTrigger>File</MenuTrigger>
        <MenubarContent>
          <MenubarItem disabled={!session} onSelect={() => setCreating(true)}>
            New Cycle
          </MenubarItem>
          <MenubarSeparator />
          <MenubarItem onSelect={() => void navigate({ to: "/settings" })}>Settings…</MenubarItem>
          {session || desktop ? <MenubarSeparator /> : null}
          {session ? (
            <MenubarItem onSelect={() => void controller.disconnect()}>Sign Out</MenubarItem>
          ) : null}
          {desktop ? (
            <MenubarItem onSelect={() => void closeDesktopWindow()}>Quit</MenubarItem>
          ) : null}
        </MenubarContent>
      </MenubarMenu>

      <MenubarMenu>
        <MenuTrigger>View</MenuTrigger>
        <MenubarContent>
          <MenubarItem onSelect={sidebar.toggle}>Toggle Sidebar</MenubarItem>
          <MenubarCheckboxItem
            checked={focused}
            disabled={cycleId === null}
            onCheckedChange={(next) => setFocused(next === true)}
          >
            Focus Mode
          </MenubarCheckboxItem>
          <MenubarCheckboxItem
            checked={consoleOpen}
            onCheckedChange={(next) =>
              /* Opening stamps the time the attention badge counts from, as
                 the status bar's toggle does. */
              setConsoleState(next ? { lastOpenedAt: Date.now(), open: true } : { open: false })
            }
          >
            Console
          </MenubarCheckboxItem>
          <MenubarSeparator />
          <LiveDensityGroup />
          <MenubarSeparator />
          <LiveThemeGroup />
        </MenubarContent>
      </MenubarMenu>

      <MenubarMenu>
        <MenuTrigger>Help</MenuTrigger>
        <MenubarContent>
          <MenubarItem
            onSelect={() =>
              void navigate({ params: { section: "diagnostics" }, to: "/settings/$section" })
            }
          >
            Diagnostics
          </MenubarItem>
          <MenubarItem
            onSelect={() =>
              void navigate({ params: { section: "about" }, to: "/settings/$section" })
            }
          >
            About
          </MenubarItem>
        </MenubarContent>
      </MenubarMenu>

      {/* Outside every menu: the menu closing on select must not take the
          dialog with it. */}
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

/**
 * Density. The desktop saves it to application settings, which then refresh
 * the store; the browser keeps it in the store alone.
 */
function LiveDensityGroup() {
  const density = useUiStore((state) => state.density);
  const setUiState = useUiStore((state) => state.setUiState);
  const desktop = isTauriDesktop();
  const settings = useApplicationSettings();
  const save = useSaveAppearance();

  const choose = (value: DensityPreference) => {
    if (!desktop) {
      setUiState({ density: value });
      return;
    }
    save.mutate(
      { density: value },
      {
        onError: (error) =>
          toast.error("Appearance could not be saved", {
            description: applicationSettingsError(error),
          }),
      },
    );
  };

  return (
    <DensityGroup
      disabled={desktop && (!settings.data || save.isPending)}
      onChange={choose}
      value={density}
    />
  );
}

/** Theme mode, under the same locks as the Appearance settings. */
function LiveThemeGroup() {
  const appearance = useAppearance();
  const { forcedTheme, saving, setTheme, theme } = useApplicationTheme();

  return (
    <ThemeGroup
      disabled={Boolean(saving) || appearance.mode_locked || Boolean(forcedTheme)}
      onChange={setTheme}
      value={(theme as ThemeModePreference | undefined) ?? "system"}
    />
  );
}

function DensityGroup({
  disabled = false,
  onChange,
  value,
}: {
  disabled?: boolean;
  onChange: (value: DensityPreference) => void;
  value: DensityPreference;
}) {
  return (
    <>
      <MenubarLabel className="text-xs text-muted-foreground">Density</MenubarLabel>
      <MenubarRadioGroup
        onValueChange={(next) => onChange(next as DensityPreference)}
        value={value}
      >
        {DENSITY_OPTIONS.map((option) => (
          <MenubarRadioItem disabled={disabled} key={option.value} value={option.value}>
            {option.label}
          </MenubarRadioItem>
        ))}
      </MenubarRadioGroup>
    </>
  );
}

function ThemeGroup({
  disabled = false,
  onChange,
  value,
}: {
  disabled?: boolean;
  onChange: (value: ThemeModePreference) => void;
  value: ThemeModePreference;
}) {
  return (
    <>
      <MenubarLabel className="text-xs text-muted-foreground">Theme</MenubarLabel>
      <MenubarRadioGroup
        onValueChange={(next) => onChange(next as ThemeModePreference)}
        value={value}
      >
        {THEME_OPTIONS.map((option) => (
          <MenubarRadioItem disabled={disabled} key={option.value} value={option.value}>
            {option.label}
          </MenubarRadioItem>
        ))}
      </MenubarRadioGroup>
    </>
  );
}

/** A trigger that fills the row's height, as a system menu bar's titles do. */
function MenuTrigger({ children }: { children: string }) {
  return (
    <MenubarTrigger className="h-full rounded-none px-3 font-normal [font-size:inherit]">
      {children}
    </MenubarTrigger>
  );
}

/* === Private Functions === */

/** Quit closes the one main window; the desktop shell exits with it. */
async function closeDesktopWindow(): Promise<void> {
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().close();
  } catch (error) {
    toast.error("The window could not be closed", {
      description: error instanceof Error ? error.message : String(error),
    });
  }
}
