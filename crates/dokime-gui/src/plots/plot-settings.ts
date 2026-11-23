// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ApplicationSettings } from "@generated/typescript/application-settings";
import defaults from "@generated/typescript/application-settings.json";
import type { DocumentDownsampleTechnique } from "../lib/types";

export type PlotDisplaySettings = {
  /** Null Restores The Host Default. Zero Keeps Every Retained Sample. */
  display_points: number | null;
  downsample: DocumentDownsampleTechnique;
};

export type ReportDefaults = {
  plotPoints: ApplicationSettings["report"]["plot_points"];
  plotDownsample: Exclude<ApplicationSettings["report"]["plot_downsample"], "unspecified">;
};

export const BUILT_IN_REPORT_DEFAULTS: ReportDefaults = {
  plotPoints: defaults.report.plot_points,
  plotDownsample: defaults.report.plot_downsample as ReportDefaults["plotDownsample"],
};
