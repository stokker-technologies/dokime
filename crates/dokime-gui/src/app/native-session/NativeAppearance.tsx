// Dokime — SPDX-License-Identifier: Apache-2.0
import { useEffect } from "react";
import { useApplicationSettings } from "@/lib/application-settings";
import { brandPack, defaultBrand } from "@/lib/brand";
import { commands } from "@/lib/desktop/commands";
import type { AppearanceResource } from "@/lib/rpc/appearance";
import { applyThemeStylesheet, applyThemeVariables, themeCornerRadiusVariables } from "@/lib/theme";
import { useUiStore } from "@/stores/ui-store";

/** The compiled brand's theme family, as the runtime registers it. */
const BRAND_THEME_ID = "brand.theme";

/** Apply typed native appearance. Every backend's DOM resources leave with its generation. */
export function NativeAppearance({ resource }: { resource: AppearanceResource | null }) {
  const settings = useApplicationSettings();
  useEffect(() => {
    if (!resource) return;
    let active = true;
    const appearance = resource.appearance;
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const preference = settings.data?.appearance.themeMode ?? "system";
    const preferredFamily = settings.data?.appearance.themeFamily ?? null;
    const apply = () => {
      if (!active) return;
      const dark =
        appearance.modePolicy?.HarmosVariant.case === "ForceDark" ||
        (appearance.modePolicy?.HarmosVariant.case !== "ForceLight" &&
          (appearance.modePolicy?.HarmosVariant.case === "ForceSystem" || preference === "system" ? media.matches : preference === "dark"));
      document.documentElement.classList.toggle("dark", dark);
      document.documentElement.classList.toggle("light", !dark);
      document.documentElement.style.colorScheme = dark ? "dark" : "light";
      const selected = appearance.themeFamilyLocked ? appearance.activeTheme : (preferredFamily ?? appearance.activeTheme);
      const family =
        appearance.themes.find((theme) => theme.themeId === selected) ??
        appearance.themes.find((theme) => theme.themeId === appearance.activeTheme) ??
        appearance.themes[0];
      // An operator's branding pack replaces the compiled brand theme; its stylesheet is already in place.
      if (brandPack && family?.themeId === BRAND_THEME_ID) {
        applyThemeVariables(null);
        applyThemeStylesheet(null);
        return;
      }
      const variant = dark ? family?.dark : family?.light;
      const radii = family?.cornerRadii;
      applyThemeVariables({
        ...Object.fromEntries((family?.variables ?? []).map(({ key, value }) => [key, value])),
        ...Object.fromEntries((variant?.variables ?? []).map(({ key, value }) => [key, value])),
        ...themeCornerRadiusVariables(radii && { ...radii, "2xl": radii.twoXl, "3xl": radii.threeXl, "4xl": radii.fourXl }),
        ...Object.fromEntries((variant?.plotPalette ?? []).map((color, index) => [`chart-${index + 1}`, color])),
      });
      applyThemeStylesheet(family?.stylesheet);
    };
    apply();
    media.addEventListener("change", apply);
    /* Density has one writer: the store, applied by `ThemeController`. A
       second `applyDensity` here re-ran on every settings refetch and undid
       whatever the current density was between the cleanup and the reapply. */
    useUiStore.getState().setUiState({ density: settings.data?.appearance.density ?? "comfortable" });
    const title = brandPack
      ? brandPack.titleTemplate.replaceAll("{app}", brandPack.label).trim() || brandPack.label
      : (appearance.titleTemplate ?? "{app}").replaceAll("{app}", appearance.appName).trim() || defaultBrand.label;
    document.title = title;
    void commands.desktopSetTitle(title).catch(() => {});
    return () => {
      active = false;
      media.removeEventListener("change", apply);
      applyThemeStylesheet(null);
      applyThemeVariables(null);
      document.documentElement.classList.toggle("dark", media.matches);
      document.documentElement.classList.toggle("light", !media.matches);
      document.documentElement.style.colorScheme = media.matches ? "dark" : "light";
      document.title = defaultBrand.label;
      void commands.desktopSetTitle(defaultBrand.label).catch(() => {});
    };
  }, [resource, settings.data]);
  return null;
}
