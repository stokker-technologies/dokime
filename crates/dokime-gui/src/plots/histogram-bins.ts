/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The bars a histogram draws, read from the backend's distribution. The
 * browser never bins samples itself. */

/** The backend's distribution of one series: its bins and their total. */
export type HistogramDistribution = {
  bins: { lower: number; upper: number; count: number }[];
  count: number;
};

export type HistogramBar = {
  slot: string;
  min: number;
  max: number;
  count: number;
};

/** One bar per backend bin, in bin order. */
export function histogramBars(distribution: HistogramDistribution): HistogramBar[] {
  return distribution.bins.map((bin, index) => ({
    slot: String(index),
    min: bin.lower,
    max: bin.upper,
    count: bin.count,
  }));
}
