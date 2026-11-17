// Dokime — SPDX-License-Identifier: Apache-2.0

/* === Imports === */

import { appearanceFrameFromNative } from "./native-appearance";

import { failureOf } from "@/lib/notification-details";
import { toast } from "@/lib/notifications";
import { useEffect, useMemo, type ReactNode } from "react";

/* === Local Imports === */

import { useNativeAppearance } from "@/app/native-session/context";
import { BrandProvider, bootOrganization, brandFromLogos, defaultBrand, organizationPack, packLogos } from "@/lib/brand";

import { setDesktopWindowTitle } from "@/lib/desktop-title";
import { applyBrandFavicon } from "@/lib/favicon";
import type { RuntimeAppLogo, RuntimeAppearanceFrame } from "@/lib/types/view-contracts";
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
