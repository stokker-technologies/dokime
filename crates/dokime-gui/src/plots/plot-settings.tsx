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

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { createContext, type ComponentType, useContext } from "react";
// Local Imports
import type { DocumentDownsampleTechnique, DocumentViewPlot } from "@/lib/types";

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

export type PlotDisplaySettings = {
  /** `null` restores the host default; `0` keeps every retained sample. */
  display_points: number | null;
  downsample: DocumentDownsampleTechnique;
};

export type PlotSettingsButtonProps = {
  plot: DocumentViewPlot;
  onSubmit: (settings: PlotDisplaySettings) => Promise<void>;
};

/* === Classes === */

/* === Public Functions === */

/** Only a connected workspace supplies an editor; frozen plots retain their captured settings. */
export const PlotSettingsEditorContext = createContext<ComponentType<PlotSettingsButtonProps> | null>(null);

export function usePlotSettingsEditor() {
  return useContext(PlotSettingsEditorContext);
}

/* === Private Functions === */

/* === Entry Point === */
