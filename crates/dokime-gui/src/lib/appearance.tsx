/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/** Appearance Context Shared by Live and Saved Views. */

/* === Imports === */

import { createContext, useContext } from "react";
import { defaultBrand, defaultTitleTemplate, organizationPack } from "@/lib/brand";
import type { RuntimeAppearanceFrame } from "@/lib/types/view-contracts";

/* === Re-Exports === */

export type { RuntimeAppearanceFrame, RuntimeAppLogo, RuntimeThemeFamily, ThemeModePolicy } from "@/lib/types/view-contracts";

/* === Constants === */

export const defaultAppearanceFrame: Appearance = {
  app_name: defaultBrand.label,
  organization: organizationPack?.organization ? packOrganization(organizationPack.organization) : null,
  title_template: defaultTitleTemplate,
  logos: [],
  themes: [],
  active_theme: null,
  theme_family_locked: false,
  mode_policy: "user_or_system",
  mode_locked: false,
  source_plugin_id: null,
};
export const AppearanceContext = createContext<Appearance>(defaultAppearanceFrame);
export const appearanceQueryKey = ["appearance"] as const;

/* === Types === */

export type Appearance = RuntimeAppearanceFrame & { themes: NonNullable<RuntimeAppearanceFrame["themes"]> };

/* === Public Functions === */

export function useAppearance(): Appearance {
  return useContext(AppearanceContext);
}

export function packOrganization(
  organization: NonNullable<NonNullable<typeof organizationPack>["organization"]>,
): RuntimeAppearanceFrame["organization"] {
  return { name: organization.name, unit: organization.unit ?? null, website: organization.website ?? null };
}
