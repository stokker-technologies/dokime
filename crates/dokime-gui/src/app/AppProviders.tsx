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
import { Toaster } from "sonner";

/* === Local Imports === */

import { AppearanceProvider, type ThemeModePolicy, useAppearance } from "@/lib/appearance";
import { DesktopThemeProvider, useApplicationTheme } from "@/lib/application-theme";
import { isTauriDesktop } from "@/lib/desktop-open";
import { useNativeSessionOptional } from "./native-session/NativeSessionProvider";
import { createDokimeQueryClient } from "./query-client";

/* === Types === */

type AppProvidersProps = { children: ReactNode };
type ForcedTheme = "light" | "dark";

/* === Constants === */

const THEME_STORAGE_KEY = "dokime-test-hub.theme";
const DOKIME_TOAST_CLASSES = {
  toast:
    "flex w-full items-start gap-3 rounded-2xl border border-border bg-popover p-4 font-sans text-sm text-popover-foreground",
  title: "font-medium text-popover-foreground",
  description: "text-xs text-muted-foreground",
  content: "grid min-w-0 gap-1",
  icon: "mt-0.5 shrink-0",
  success: "border-success bg-popover",
  error: "border-destructive bg-popover",
  warning: "border-warning bg-popover",
  info: "border-info bg-popover",
  loading: "border-border bg-muted",
  actionButton: "rounded-md bg-primary px-2 py-1 text-primary-foreground",
  cancelButton: "rounded-md bg-secondary px-2 py-1 text-secondary-foreground",
  closeButton:
    "rounded-md border border-border bg-popover text-muted-foreground hover:text-popover-foreground",
} as const;

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

  return session ? (
    content
  ) : (
    <QueryClientProvider client={queryClient}>{content}</QueryClientProvider>
  );
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

function DokimeToaster() {
  const { resolvedTheme } = useApplicationTheme();

  return (
    <Toaster
      position="top-right"
      className="z-[100]"
      theme={resolvedTheme === "dark" ? "dark" : "light"}
      toastOptions={{ unstyled: true, classNames: DOKIME_TOAST_CLASSES }}
    />
  );
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
