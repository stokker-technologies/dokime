/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* What a plot shows before the backend has delivered anything to draw. Every
 * plot kind words and places its wait the same way. */

/* === Types === */

export type PlotPlaceholderText = {
  label: string;
  detail: string;
};

/* === Constants === */

/** The label of every plot whose backend payload has no samples yet. */
export const WAITING_FOR_SAMPLES = "Waiting For Samples";

/** The detail of a numeric plot with no samples in its window. */
export const NO_NUMERIC_SAMPLES = "No numeric samples are available in the selected window.";

/* === Public Functions === */

/** The placeholder of a plot the backend could not produce, with its reason. */
export function plotUnavailableText(message: string): PlotPlaceholderText {
  return { label: "Plot Unavailable", detail: message };
}

/**
 * The placeholder text for a plot without data: the stream state's label and
 * detail when the live source stated one, otherwise the shared wait.
 */
export function plotPlaceholderText(
  state: { stateLabel?: string | null; stateDetail?: string | null },
  detail: string = NO_NUMERIC_SAMPLES,
): PlotPlaceholderText {
  return {
    label: state.stateLabel ?? WAITING_FOR_SAMPLES,
    detail: state.stateDetail ?? detail,
  };
}
