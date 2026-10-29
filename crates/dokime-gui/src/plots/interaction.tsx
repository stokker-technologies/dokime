/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { createContext, type ReactNode, useContext, useMemo } from "react";

/**
 * How a reader may interact with the plots inside one surface.
 *
 * Interaction is a property of the surface, not of one card: every plot in a
 * report is navigated, live plots follow the run until the reader selects a zoomed viewport. The
 * surface states it once here and the plot components read it, so nothing has
 * to be threaded through the grid and the catalog.
 */
export type PlotInteraction = {
  /**
   * Whether this surface is a report, where the viewport control resets zoom.
   * Live surfaces label it Follow and resume their streaming viewport instead.
   * Both surfaces support box zoom on both axes.
   */
  zoom: boolean;
  /** Compatible plots in one group share only cursor movement. */
  syncKey?: string;
};

const PlotInteractionContext = createContext<PlotInteraction>({ zoom: false });

export function PlotInteractionProvider({ children, zoom, syncKey }: PlotInteraction & { children: ReactNode }) {
  const value = useMemo(() => ({ zoom, syncKey }), [zoom, syncKey]);
  return <PlotInteractionContext.Provider value={value}>{children}</PlotInteractionContext.Provider>;
}

export function usePlotInteraction(): PlotInteraction {
  return useContext(PlotInteractionContext);
}
