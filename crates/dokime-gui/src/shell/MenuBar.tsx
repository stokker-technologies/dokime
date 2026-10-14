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
 * its title and the actions the page publishes. The frozen report keeps the
 * same row with only what a file can honour: how it is displayed, and what it
 * is. It never navigates and never reaches the desktop bridge.
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
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
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
import { useAppearance } from "@/lib/appearance";
import {
  applicationSettingsError,
  useApplicationSettings,
  useSaveAppearance,
} from "@/lib/application-settings";
import { useApplicationTheme } from "@/lib/application-theme";
import { useBrand } from "@/lib/brand";
import { isTauriDesktop } from "@/lib/desktop-open";
import { applyDensity, type DensityPreference, type ThemeModePreference } from "@/lib/theme";
import { applyReportColorMode } from "@/report/report-appearance";
import { useFocusMode } from "@/shell/focus-mode";
import { useRouteCycleId } from "@/shell/route-scope";
import { useShellDensity } from "@/shell/use-shell-density";
import { useSidebarState } from "@/shell/use-sidebar-state";
import { useUiStore } from "@/stores/ui-store";

/* === Types === */

export type MenuBarProps = { live?: true } | { live: false; generatedAt: string | null };

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

/** The menu bar row. `live` is the only switch, as for the shell around it. */
export function MenuBar(props: MenuBarProps) {
  return (
    <Menubar
      aria-label="Application"
      className="h-[var(--menu-bar-height)] gap-0 rounded-none border-0 border-b bg-background p-0 px-[calc(var(--shell-inset)/2)]"
    >
      {props.live === false ? <ReportMenus generatedAt={props.generatedAt} /> : <LiveMenus />}
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

  return (
    <>
      <MenubarMenu>
        <MenuTrigger>File</MenuTrigger>
        <MenubarContent>
          <MenubarItem onSelect={() => void navigate({ to: "/library" })}>New Cycle</MenubarItem>
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

/** The report's menus: display and identity only. */
function ReportMenus({ generatedAt }: { generatedAt: string | null }) {
  const density = useShellDensity();
  const [theme, setTheme] = useState<ThemeModePreference>("system");
  const [aboutOpen, setAboutOpen] = useState(false);

  return (
    <>
      <MenubarMenu>
        <MenuTrigger>View</MenuTrigger>
        <MenubarContent>
          <DensityGroup onChange={applyDensity} value={density} />
          <MenubarSeparator />
          <ThemeGroup
            onChange={(value) => {
              applyReportColorMode(value);
              setTheme(value);
            }}
            value={theme}
          />
        </MenubarContent>
      </MenubarMenu>

      <MenubarMenu>
        <MenuTrigger>Help</MenuTrigger>
        <MenubarContent>
          <MenubarItem onSelect={() => setAboutOpen(true)}>About</MenubarItem>
        </MenubarContent>
      </MenubarMenu>

      <ReportAboutDialog generatedAt={generatedAt} onOpenChange={setAboutOpen} open={aboutOpen} />
    </>
  );
}

function ReportAboutDialog({
  generatedAt,
  onOpenChange,
  open,
}: {
  generatedAt: string | null;
  onOpenChange: (open: boolean) => void;
  open: boolean;
}) {
  const brand = useBrand();
  const at = generatedAt ? new Date(generatedAt) : null;
  const generated = at && !Number.isNaN(at.getTime()) ? at.toLocaleString() : "Unknown";

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>About This Report</DialogTitle>
          <DialogDescription>
            A frozen report: a read-only record written by {brand.label}.
          </DialogDescription>
        </DialogHeader>
        <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
          <dt className="text-muted-foreground">Application</dt>
          <dd>{brand.label}</dd>
          <dt className="text-muted-foreground">Version</dt>
          <dd className="font-mono">v{__APP_VERSION__}</dd>
          <dt className="text-muted-foreground">Generated</dt>
          <dd>{generated}</dd>
        </dl>
      </DialogContent>
    </Dialog>
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
