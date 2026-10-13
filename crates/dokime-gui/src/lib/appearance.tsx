/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { createContext, type ReactNode, useContext, useEffect, useMemo } from "react";

/* === Local Imports === */

import { useNativeAppearance } from "@/app/native-session/NativeSessionProvider";
import { type Brand, BrandProvider, defaultBrand } from "@/lib/brand";
import { setDesktopWindowTitle } from "@/lib/desktop-title";
import type { AppearanceResource } from "@/lib/rpc/appearance";

import { isReportBuild } from "@/lib/ui-mode";

/* === Types === */

import type {
  RuntimeAppearanceFrame,
  ThemeModePolicy as RuntimeThemeModePolicy,
} from "@/lib/types/view-contracts";

export type {
  RuntimeAppearanceFrame,
  RuntimeAppLogo,
  RuntimeThemeFamily,
  ThemeModePolicy,
} from "@/lib/types/view-contracts";

/* === Constants === */

type Appearance = RuntimeAppearanceFrame & {
  themes: NonNullable<RuntimeAppearanceFrame["themes"]>;
};

const defaultAppearanceFrame: Appearance = {
  app_name: defaultBrand.label,
  organization: null,
  title_template: "{app}",
  logo: null,
  themes: [],
  active_theme: null,
  theme_family_locked: false,
  mode_policy: "user_or_system",
  mode_locked: false,
  source_plugin_id: null,
};

const AppearanceContext = createContext<Appearance>(defaultAppearanceFrame);

export const appearanceQueryKey = ["appearance"] as const;

/* === Components === */

export function AppearanceProvider({ children }: { children: ReactNode }) {
  const nativeResource = useNativeAppearance();
  const appearance = useMemo<Appearance>(() => {
    const frame = nativeResource
      ? appearanceFrameFromNative(nativeResource)
      : defaultAppearanceFrame;
    return { ...frame, themes: frame.themes ?? [] };
  }, [nativeResource]);
  const brand = useMemo(() => brandFromAppearance(appearance), [appearance]);

  useEffect(() => {
    const title = titleFromAppearance(appearance);
    document.title = title;
    /* A report has no window of its own to name, and the entry never mounts
       this provider; the guard keeps the call unreachable rather than relying
       on that. */
    if (!isReportBuild()) void setDesktopWindowTitle(title);
  }, [appearance]);

  return (
    <AppearanceContext.Provider value={appearance}>
      <BrandProvider brand={brand}>{children}</BrandProvider>
    </AppearanceContext.Provider>
  );
}

/* === Public Functions === */

export function useAppearance(): Appearance {
  return useContext(AppearanceContext);
}

/* === Private Functions === */

function brandFromAppearance(appearance: RuntimeAppearanceFrame): Brand {
  if (!appearance.logo) {
    return {
      label: appearance.app_name,
      logo: defaultBrand.logo,
    };
  }

  return {
    label: appearance.app_name,
    logo: {
      kind: "image",
      src: appearance.logo.url,
      alt: appearance.logo.alt ?? appearance.app_name,
    },
  };
}

function titleFromAppearance(appearance: RuntimeAppearanceFrame): string {
  const template = appearance.title_template?.trim() || "{app}";
  return template.replaceAll("{app}", appearance.app_name);
}

function appearanceFrameFromNative(resource: AppearanceResource): RuntimeAppearanceFrame {
  const snapshot = resource.appearance;
  return {
    app_name: snapshot.appName || defaultBrand.label,
    organization: snapshot.organization
      ? {
          name: snapshot.organization.name,
          unit: snapshot.organization.unit ?? null,
          website: snapshot.organization.website ?? null,
        }
      : null,
    title_template: snapshot.titleTemplate ?? "{app}",
    logo:
      snapshot.logo && resource.logoUrl
        ? {
            plugin_id: snapshot.logo.asset?.reference?.pluginId ?? "",
            asset_id: snapshot.logo.asset?.reference?.assetId ?? "",
            url: resource.logoUrl,
            alt: snapshot.logo.alt ?? null,
          }
        : null,
    themes: snapshot.themes.map((theme) => ({
      plugin_id: theme.pluginId,
      theme_id: theme.themeId,
      label: theme.label,
      description: theme.description ?? null,
      stylesheet: theme.stylesheet ?? null,
      variables: Object.fromEntries(theme.variables.map(({ key, value }) => [key, value])),
      corner_radii: theme.cornerRadii
        ? {
            base: theme.cornerRadii.base ?? null,
            sm: theme.cornerRadii.sm ?? null,
            md: theme.cornerRadii.md ?? null,
            lg: theme.cornerRadii.lg ?? null,
            xl: theme.cornerRadii.xl ?? null,
            "2xl": theme.cornerRadii.twoXl ?? null,
            "3xl": theme.cornerRadii.threeXl ?? null,
            "4xl": theme.cornerRadii.fourXl ?? null,
          }
        : undefined,
      light: {
        variables: Object.fromEntries(
          (theme.light?.variables ?? []).map(({ key, value }) => [key, value]),
        ),
        plot_palette: [...(theme.light?.plotPalette ?? [])],
      },
      dark: {
        variables: Object.fromEntries(
          (theme.dark?.variables ?? []).map(({ key, value }) => [key, value]),
        ),
        plot_palette: [...(theme.dark?.plotPalette ?? [])],
      },
    })),
    active_theme: snapshot.activeTheme ?? null,
    theme_family_locked: snapshot.themeFamilyLocked,
    mode_policy: nativeModePolicy(snapshot.modePolicy),
    mode_locked: snapshot.modeLocked,
    source_plugin_id: snapshot.sourcePluginId ?? null,
  };
}

function nativeModePolicy(
  policy: AppearanceResource["appearance"]["modePolicy"],
): RuntimeThemeModePolicy {
  switch (policy?.HarmosVariant.case) {
    case "ForceSystem":
      return "force_system";
    case "ForceLight":
      return "force_light";
    case "ForceDark":
      return "force_dark";
    default:
      return "user_or_system";
  }
}
