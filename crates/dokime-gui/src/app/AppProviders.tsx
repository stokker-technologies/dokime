/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { QueryClientProvider } from "@tanstack/react-query";
import { ThemeProvider } from "next-themes";
import { type ReactNode, useEffect, useState } from "react";

/* === Local Imports === */

import { DokimeToaster } from "@/components/DokimeToaster";
import { AppearanceProvider, type ThemeModePolicy, useAppearance } from "@/lib/appearance";
import { useApplicationTheme } from "@/lib/application-theme";
import { DesktopThemeProvider } from "@/lib/DesktopThemeProvider";
import { isTauriDesktop } from "@/lib/desktop-open";
import { useNativeSessionOptional } from "./native-session/context";
import { createDokimeQueryClient } from "./query-client";

/* === Types === */

type AppProvidersProps = { children: ReactNode };
type ForcedTheme = "light" | "dark";

/* === Constants === */

const THEME_STORAGE_KEY = "dokime-test-hub.theme";

/* === Components === */

/**
 * The connected native session owns the query client used by route data. The
 * fallback client exists only for the disconnected connection screen; keeping
 * it conditional prevents a persistent shell client from shadowing a new
 * backend generation.
 */
export function AppProviders({ children }: AppProvidersProps) {
  const session = useNativeSessionOptional();
  const [queryClient] = useState(createDokimeQueryClient);
  const content = (
    <AppearanceProvider>
      <ThemeProviderFromAppearance>
        <ThemeModePolicyController />
        {children}
        <DokimeToaster />
      </ThemeProviderFromAppearance>
    </AppearanceProvider>
  );

  return session ? content : <QueryClientProvider client={queryClient}>{content}</QueryClientProvider>;
}

function ThemeProviderFromAppearance({ children }: AppProvidersProps) {
  const appearance = useAppearance();

  if (isTauriDesktop()) return <DesktopThemeProvider>{children}</DesktopThemeProvider>;

  return (
    <ThemeProvider
      attribute="class"
      defaultTheme="system"
      disableTransitionOnChange
      enableColorScheme
      enableSystem
      forcedTheme={forcedThemeFromPolicy(appearance.mode_policy)}
      storageKey={THEME_STORAGE_KEY}
    >
      {children}
    </ThemeProvider>
  );
}

function ThemeModePolicyController() {
  const appearance = useAppearance();
  const { setTheme, theme } = useApplicationTheme();

  useEffect(() => {
    if (!isTauriDesktop() && appearance.mode_policy === "force_system" && theme !== "system") {
      setTheme("system");
    }
  }, [appearance.mode_policy, setTheme, theme]);

  return null;
}

/* === Private Functions === */

function forcedThemeFromPolicy(policy: ThemeModePolicy): ForcedTheme | undefined {
  switch (policy) {
    case "force_dark":
      return "dark";
    case "force_light":
      return "light";
    case "force_system":
    case "user_or_system":
      return undefined;
  }
}
