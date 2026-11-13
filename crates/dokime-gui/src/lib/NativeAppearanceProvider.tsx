// Dokime — SPDX-License-Identifier: Apache-2.0

/* === Imports === */

import { ThemeModePolicy } from "@host/dokime/v1/appearance_pb.js";

import { failureOf } from "@/lib/notification-details";
import { type ReactNode, useEffect, useMemo } from "react";
import { toast } from "@/lib/notifications";

/* === Local Imports === */

import { useNativeAppearance } from "@/app/native-session/context";
import { BrandProvider, bootOrganization, brandFromLogos, defaultBrand, organizationPack, packLogos } from "@/lib/brand";

import { setDesktopWindowTitle } from "@/lib/desktop-title";
import { applyBrandFavicon } from "@/lib/favicon";
import type { AppearanceResource } from "@/lib/rpc/appearance";
import type { RuntimeAppearanceFrame, RuntimeAppLogo, ThemeModePolicy as RuntimeThemeModePolicy } from "@/lib/types/view-contracts";
import { isReportBuild } from "@/lib/ui-mode";

import { AppearanceContext, defaultAppearanceFrame, packOrganization, type Appearance } from "./appearance";

/* === Components === */

let reportedOrganizationError = false;

export function AppearanceProvider({ children }: { children: ReactNode }) {
  const nativeResource = useNativeAppearance();
  const appearance = useMemo<Appearance>(() => {
    const frame = nativeResource ? withOrganizationPack(appearanceFrameFromNative(nativeResource)) : defaultAppearanceFrame;
    return { ...frame, themes: frame.themes ?? [] };
  }, [nativeResource]);
  const brand = useMemo(
    () => (nativeResource && !organizationPack ? brandFromLogos(appearance.app_name, appearance.logos) : defaultBrand),
    [appearance, nativeResource],
  );

  useEffect(() => {
    applyBrandFavicon(brand);
  }, [brand]);

  useEffect(() => {
    const error = bootOrganization.error;
    if (!error || reportedOrganizationError) return undefined;
    reportedOrganizationError = true;
    // After The Commit, So The Toaster Mounted In The Same Render Is Listening.
    const handle = window.setTimeout(() => toast.error(failureOf(error, "Organization Pack Not Applied"), { context: "background" }), 0);
    return () => window.clearTimeout(handle);
  }, []);

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

/* === Private Functions === */

function titleFromAppearance(appearance: RuntimeAppearanceFrame): string {
  const template = appearance.title_template?.trim() || "{app}";
  return template.replaceAll("{app}", appearance.app_name);
}

/** The installation's organization pack is the brand: it replaces the identity the backend reports. */
function withOrganizationPack(frame: RuntimeAppearanceFrame): RuntimeAppearanceFrame {
  if (!organizationPack) return frame;
  return {
    ...frame,
    app_name: organizationPack.label,
    organization: organizationPack.organization ? packOrganization(organizationPack.organization) : null,
    title_template: organizationPack.titleTemplate,
    logos: Object.entries(packLogos(organizationPack)).map(([variant, logo]) => ({
      variant: variant as RuntimeAppLogo["variant"],
      plugin_id: "brand_pack",
      asset_id: variant,
      url: logo.src,
      alt: logo.alt ?? null,
    })),
  };
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
    logos: resource.logos,
    themes: snapshot.themes.map((theme) => ({
      plugin_id: theme.pluginId,
      theme_id: theme.themeId,
      label: theme.label,
      description: theme.description ?? null,
      stylesheet: theme.stylesheet ?? null,
      variables: { ...theme.variables },
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
        variables: { ...theme.light?.variables },
        plot_palette: [...(theme.light?.plotPalette ?? [])],
      },
      dark: {
        variables: { ...theme.dark?.variables },
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

function nativeModePolicy(policy: AppearanceResource["appearance"]["modePolicy"]): RuntimeThemeModePolicy {
  switch (policy) {
    case ThemeModePolicy.FORCE_SYSTEM:
      return "force_system";
    case ThemeModePolicy.FORCE_LIGHT:
      return "force_light";
    case ThemeModePolicy.FORCE_DARK:
      return "force_dark";
    default:
      return "user_or_system";
  }
}
