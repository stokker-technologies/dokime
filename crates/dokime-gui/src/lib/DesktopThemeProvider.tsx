/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Connected Desktop Theme Preferences.
 */

import { type ReactNode, useEffect, useState } from "react";

import { useAppearance } from "@/lib/appearance";
import { applicationSettingsError, useApplicationSettings, useSaveAppearance } from "@/lib/application-settings";
import { toast } from "@/lib/notifications";
import type { ResolvedTheme, ThemeModePreference } from "@/lib/theme";
import { useUiStore } from "@/stores/ui-store";
import { ApplicationThemeProvider } from "./application-theme";

export function DesktopThemeProvider({ children }: { children: ReactNode }) {
  const query = useApplicationSettings();
  const save = useSaveAppearance();
  const appearance = useAppearance();
  const [system, setSystem] = useState<ResolvedTheme>(() => (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"));
  const settings = query.data?.appearance;
  const theme = settings?.themeMode ?? "system";
  const policy = appearance.mode_policy;
  const forcedTheme =
    policy === "force_dark" ? "dark" : policy === "force_light" ? "light" : policy === "force_system" ? system : undefined;
  const resolvedTheme = forcedTheme ?? (theme === "system" ? system : theme);

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const changed = () => setSystem(media.matches ? "dark" : "light");
    media.addEventListener("change", changed);
    return () => media.removeEventListener("change", changed);
  }, []);
  useEffect(() => {
    document.documentElement.classList.toggle("dark", resolvedTheme === "dark");
    document.documentElement.classList.toggle("light", resolvedTheme === "light");
    document.documentElement.style.colorScheme = resolvedTheme;
  }, [resolvedTheme]);
  useEffect(() => {
    if (settings)
      useUiStore.getState().setUiState({
        density: settings.density,
        themeFamily: settings.themeFamily,
      });
  }, [settings]);

  return (
    <ApplicationThemeProvider
      value={{
        saving: save.isPending,
        theme,
        resolvedTheme,
        forcedTheme,
        setTheme: (value) => {
          save.mutate(
            { themeMode: value as ThemeModePreference },
            {
              onError: (error) =>
                toast.error("Appearance Could Not Be Saved", {
                  description: applicationSettingsError(error),
                }),
            },
          );
        },
      }}
    >
      {children}
    </ApplicationThemeProvider>
  );
}
