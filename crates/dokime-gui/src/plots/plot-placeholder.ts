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

export type PlotPlaceholderText = {
  label: string;
  detail: string;
  /** `error` when the plot failed rather than waits: the box reads as a failure. */
  tone?: "error";
};

/** The label of every plot whose backend payload has no samples yet. */
export const WAITING_FOR_SAMPLES = "Waiting For Samples";

/** The label a plot shows when its stream reported an error and nothing is drawn. */
export const STREAM_ERROR = "Stream Error";

/** The detail of a numeric plot with no samples in its window. */
export const NO_NUMERIC_SAMPLES = "No numeric samples are available in the selected window.";

/** The placeholder of a plot the backend could not produce, with its reason. */
export function plotUnavailableText(message: string): PlotPlaceholderText {
  return { label: "Plot Unavailable", detail: message, tone: "error" };
}

/**
 * The placeholder text for a plot without data: the stream state's label and
 * detail when the live source stated one, otherwise the shared wait.
 */
export function plotPlaceholderText(
  state: { stateLabel?: string | null; stateDetail?: string | null },
  detail: string = NO_NUMERIC_SAMPLES,
): PlotPlaceholderText {
  const label = state.stateLabel ?? WAITING_FOR_SAMPLES;
  const text = { label, detail: state.stateDetail ?? detail };
  return label === STREAM_ERROR ? { ...text, tone: "error" } : text;
}
