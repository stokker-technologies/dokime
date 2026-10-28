/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * TODO: Describe this module's responsibility.
 */

import { type ComponentType, createContext, useContext } from "react";

import type { DocumentDownsampleTechnique, DocumentViewPlot } from "@/lib/types";

export type PlotDisplaySettings = {
  /** `null` restores the host default; `0` keeps every retained sample. */
  display_points: number | null;
  downsample: DocumentDownsampleTechnique;
};

export type PlotSettingsButtonProps = {
  plot: DocumentViewPlot;
  onSubmit: (settings: PlotDisplaySettings) => Promise<void>;
};

/** Only a connected workspace supplies an editor; frozen plots retain their captured settings. */
export const PlotSettingsEditorContext = createContext<ComponentType<PlotSettingsButtonProps> | null>(null);

export function usePlotSettingsEditor() {
  return useContext(PlotSettingsEditorContext);
}
