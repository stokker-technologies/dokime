/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type {
  DocumentAxisScale,
  DocumentFftOptions,
  DocumentHistogramOptions,
  DocumentPlotStyle,
  DocumentPlotType,
  DocumentReferenceBand,
  DocumentReferenceLine,
  EvidenceAnalysisResult,
  PlotAnalysis,
  PlotCartesianFrame,
  PlotMarker,
  PlotSeries,
  ValueFormat,
} from "@/lib/types";
import type { HistogramAxes } from "@/plots/histogram-axes";

/* === Types === */

export type PlotCardSpec = {
  key: string;
  stream: string;
  title: string;
  description?: string | null;
  series: PlotSeries[];
  cartesian?: PlotCartesianFrame | null;
  analysis?: PlotAnalysis | null;
  computed?: EvidenceAnalysisResult | null;
  markers: PlotMarker[];
  plotType: DocumentPlotType;
  style: DocumentPlotStyle;
  xScale: DocumentAxisScale;
  yScale: DocumentAxisScale;
  fft: DocumentFftOptions;
  histogram?: DocumentHistogramOptions | null;
  /** A histogram's axes: the declared value on x, the host's count on y. */
  histogramAxes?: HistogramAxes | null;
  timeAxis: boolean;
  unit?: string | null;
  colors: string[];
  referenceLines: DocumentReferenceLine[];
  referenceBands: DocumentReferenceBand[];
  stateLabel?: string;
  stateDetail?: string;
  contextSummary?: string;
  context?: Record<string, unknown>;
  liveWindowS?: number | null;
  xFormat?: ValueFormat | null;
  /** Set when the x column declares a duration: the axis prints elapsed time. */
  xElapsedUnit?: string | null;
  yFormat?: ValueFormat | null;
  runStartedAt?: number | null;
  cycleStartedAt?: number | null;
};

export type PlotViewportMode = "fit" | "live";
