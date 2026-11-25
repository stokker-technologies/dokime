// Dokime — SPDX-License-Identifier: Apache-2.0
import { brandFromLogos, defaultBrand, defaultTitleTemplate, organizationPack } from "../lib/brand";
import { appearanceFrameFromNative, type AppearanceSnapshot } from "../lib/native-appearance";
import type { RuntimeAppearanceFrame } from "../lib/types/appearance";

const organization = organizationPack?.organization;

/** The installation's boot pack already styled the page; only a shell without one takes the host's appearance. */
export function desktopAppearance(snapshot: AppearanceSnapshot | null) {
  const native = organizationPack || !snapshot ? null : appearanceFrameFromNative(snapshot);
  const appearance: RuntimeAppearanceFrame = native ?? {
    app_name: defaultBrand.label,
    organization: organization ? { name: organization.name, unit: organization.unit ?? null, website: organization.website ?? null } : null,
    title_template: defaultTitleTemplate,
    logos: [],
    themes: [],
    active_theme: null,
    theme_family_locked: false,
    mode_policy: "user_or_system",
    mode_locked: false,
  };
  const brand = native ? brandFromLogos(appearance.app_name, appearance.logos) : defaultBrand;
  const title = (appearance.title_template?.trim() || "{app}").replaceAll("{app}", appearance.app_name);
  return { appearance, brand, title };
}
