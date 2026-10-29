/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Local Appearance And Backend Report Preferences From Their Native Scopes.
 */

/* === Imports === */

import { useMutation } from "@tanstack/react-query";
import { useMemo } from "react";
import type { ApplicationSettings } from "@generated/application-settings";
import defaults from "@generated/application-settings.json";
import { preferencesError, usePreferences, useSavePreferences } from "@/features/settings/application-preferences";

/* === Types === */

export type ThemeMode = ApplicationSettings["appearance"]["theme_mode"];
export type Density = ApplicationSettings["appearance"]["density"];
export type Appearance = {
  themeMode: ThemeMode;
  themeFamily: ApplicationSettings["appearance"]["theme_family"];
  density: Density;
};
export type ReportDefaults = {
  plotPoints: ApplicationSettings["report"]["plot_points"];
  plotDownsample: Exclude<ApplicationSettings["report"]["plot_downsample"], "unspecified">;
};

/* === Constants === */

export const BUILT_IN_REPORT_DEFAULTS: ReportDefaults = {
  plotPoints: defaults.report.plot_points,
  plotDownsample: defaults.report.plot_downsample as ReportDefaults["plotDownsample"],
};

/* === Public Functions === */

export function useApplicationSettings() {
  const local = usePreferences("local");
  const backend = usePreferences("backend");
  const appearance = local.data?.settings.appearance;
  const report = backend.data?.settings.report;
  const data = useMemo(
    () =>
      appearance
        ? {
            appearance: { themeMode: appearance.theme_mode, themeFamily: appearance.theme_family, density: appearance.density },
            report: report
              ? { plotPoints: report.plot_points, plotDownsample: report.plot_downsample as ReportDefaults["plotDownsample"] }
              : BUILT_IN_REPORT_DEFAULTS,
          }
        : undefined,
    [appearance, report],
  );
  return {
    data,
    error: local.error,
    isError: local.isError,
    isPending: local.isPending,
    refetch: () => Promise.all([local.refetch(), backend.refetch()]),
  };
}

export function useSaveAppearance() {
  const query = usePreferences("local");
  const save = useSavePreferences("local");
  return useMutation({
    mutationFn: async (patch: Partial<Appearance>) => {
      const previous = query.data;
      if (!previous) throw new Error("Local settings are unavailable");
      const appearance = { ...previous.settings.appearance };
      if (patch.themeMode !== undefined) appearance.theme_mode = patch.themeMode;
      if (patch.themeFamily !== undefined) appearance.theme_family = patch.themeFamily;
      if (patch.density !== undefined) appearance.density = patch.density;
      return save.mutateAsync({ previous, settings: { ...previous.settings, appearance } });
    },
  });
}

export function useReportDefaults(): ReportDefaults {
  return useApplicationSettings().data?.report ?? BUILT_IN_REPORT_DEFAULTS;
}

export const applicationSettingsError = preferencesError;

/* === Private Functions === */

/* === Entry Point === */
