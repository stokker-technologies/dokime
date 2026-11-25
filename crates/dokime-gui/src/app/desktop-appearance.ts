// Dokime — SPDX-License-Identifier: Apache-2.0
import { type AppearanceSnapshot, brandFromAppearance, defaultBrand, defaultTitleTemplate } from "../lib/brand";

/** The host's appearance, read only by a shell without a local pack; otherwise the boot or build-time brand stands. */
export function desktopAppearance(host: AppearanceSnapshot | null) {
  const brand = host ? brandFromAppearance(host) : defaultBrand;
  const template = host ? host.titleTemplate : defaultTitleTemplate;
  const title = (template?.trim() || "{app}").replaceAll("{app}", brand.label);
  return { brand, title, stylesheet: host?.stylesheet ?? null };
}
