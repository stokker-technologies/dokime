// Dokime — SPDX-License-Identifier: Apache-2.0
import {
  type AppearanceSnapshot,
  appearanceStylesheet,
  brandFromAppearance,
  defaultBrand,
  defaultTitleTemplate,
  organizationPack,
} from "../lib/brand";

/** The installation's boot pack already styled the page; only a shell without one takes the host's appearance. */
export function desktopAppearance(snapshot: AppearanceSnapshot | null) {
  const host = organizationPack ? null : snapshot;
  const brand = host ? brandFromAppearance(host) : defaultBrand;
  const template = host ? host.titleTemplate : defaultTitleTemplate;
  const title = (template?.trim() || "{app}").replaceAll("{app}", brand.label);
  return { brand, title, stylesheet: host ? appearanceStylesheet(host) : null };
}
