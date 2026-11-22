/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Shared Native Response Projection
 */

/* === Imports === */

import type * as appearance from "@/lib/types/appearance";

import type { RuntimeAppLogo } from "@/lib/types/appearance";

import { LogoVariant } from "@generated/host/model";

import { defaultBrand } from "./brand";

import type { BackendAppearanceResponse } from "@generated/host/model";
import type { Wire } from "@/lib/rpc/canonical";

import type { RuntimeAppearanceFrame } from "@/lib/types/appearance";

import { ThemeModePolicy } from "@generated/host/model";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

type AppearanceSnapshot = Wire<NonNullable<BackendAppearanceResponse["appearance"]>>;

export interface AppearanceResource {
  appearance: AppearanceSnapshot;
  logos: RuntimeAppLogo[];
  unavailableReason?: string;
  dispose(): void;
}

/* === Classes === */

/* === Public Functions === */

export function appearanceFrameFromNative(resource: AppearanceResource): RuntimeAppearanceFrame {
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

export function logoVariant(logo: AppearanceSnapshot["logos"][number]): RuntimeAppLogo["variant"] {
  switch (logo.variant) {
    case LogoVariant.TRANSPARENT_ON_LIGHT:
      return "transparent_on_light";
    case LogoVariant.TRANSPARENT_ON_DARK:
      return "transparent_on_dark";
    case LogoVariant.CONTAINED_ON_LIGHT:
      return "contained_on_light";
    case LogoVariant.CONTAINED_ON_DARK:
      return "contained_on_dark";
    default:
      throw new Error("Open Appearance: Select a Declared Logo Variant");
  }
}

/* === Private Functions === */

function nativeModePolicy(policy: AppearanceResource["appearance"]["modePolicy"]): appearance.ThemeModePolicy {
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

/* === Entry Point === */
