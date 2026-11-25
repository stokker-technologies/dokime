// Dokime — SPDX-License-Identifier: Apache-2.0

export type OrganizationIdentity = {
  name: string;
  unit?: string | null;
  website?: string | null;
};

export type RuntimeAppLogo = {
  variant: "transparent_on_light" | "transparent_on_dark" | "contained_on_light" | "contained_on_dark";
  url: string;
  alt?: string | null;
};

export type RuntimeAppearanceFrame = {
  app_name: string;
  organization?: OrganizationIdentity | null;
  title_template?: string | null;
  logos: RuntimeAppLogo[];
  themes?: RuntimeThemeFamily[];
  active_theme?: string | null;
  theme_family_locked: boolean;
  mode_policy: ThemeModePolicy;
  mode_locked: boolean;
};

export type RuntimeThemeFamily = {
  theme_id: string;
  label: string;
  description?: string | null;
  stylesheet?: string | null;
  variables?: { [key in string]: string };
  corner_radii?: ThemeCornerRadii;
  light: ThemeVariant;
  dark: ThemeVariant;
};

export type ThemeCornerRadii = {
  base?: string | null;
  sm?: string | null;
  md?: string | null;
  lg?: string | null;
  xl?: string | null;
  "2xl"?: string | null;
  "3xl"?: string | null;
  "4xl"?: string | null;
};

export type ThemeModePolicy = "user_or_system" | "force_system" | "force_light" | "force_dark";

export type ThemeVariant = {
  variables?: { [key in string]: string };
  plot_palette?: string[];
};
