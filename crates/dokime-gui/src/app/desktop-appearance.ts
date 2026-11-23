// Dokime — SPDX-License-Identifier: Apache-2.0
import { brandFromLogos, defaultBrand, defaultTitleTemplate, organizationPack, packLogos } from "../lib/brand";
import { appearanceFrameFromNative, type AppearanceResource } from "../lib/native-appearance";
import type { RuntimeAppearanceFrame, RuntimeAppLogo } from "../lib/types/appearance";

export const appearanceQueryKey = ["appearance"] as const;

const organization = organizationPack?.organization;

export function desktopAppearance(resource: AppearanceResource | null) {
  const native = resource ? appearanceFrameFromNative(resource) : undefined;
  const appearance: RuntimeAppearanceFrame = {
    app_name: organizationPack?.label ?? native?.app_name ?? defaultBrand.label,
    organization: organizationPack
      ? organization
        ? { name: organization.name, unit: organization.unit ?? null, website: organization.website ?? null }
        : null
      : (native?.organization ?? null),
    title_template: organizationPack?.titleTemplate ?? native?.title_template ?? defaultTitleTemplate,
    logos:
      organizationPack && native
        ? Object.entries(packLogos(organizationPack)).map(([variant, logo]) => ({
            variant: variant as RuntimeAppLogo["variant"],
            plugin_id: "brand_pack",
            asset_id: variant,
            url: logo.src,
            alt: logo.alt ?? null,
          }))
        : (native?.logos ?? []),
    themes: native?.themes ?? [],
    active_theme: native?.active_theme ?? null,
    theme_family_locked: native?.theme_family_locked ?? false,
    mode_policy: native?.mode_policy ?? "user_or_system",
    mode_locked: native?.mode_locked ?? false,
    source_plugin_id: native?.source_plugin_id ?? null,
  };
  const brand = resource && !organizationPack ? brandFromLogos(appearance.app_name, appearance.logos) : defaultBrand;
  const title = (appearance.title_template?.trim() || "{app}").replaceAll("{app}", appearance.app_name);
  return { appearance, brand, title };
}
