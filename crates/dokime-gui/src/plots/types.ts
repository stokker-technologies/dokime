/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { SeriesSummary } from "@/lib/statistics";
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
import type { PromptMarker } from "@/plots/prompt-markers";

/* === Types === */

export type PlotCardSpec = {
  key: string;
  stream: string;
  title: string;
  /** The heading above already prints this title; the card keeps it for assistive tech only. */
  titleRepeated?: boolean;
  description?: string | null;
  series: PlotSeries[];
  cartesian?: PlotCartesianFrame | null;
  analysis?: PlotAnalysis | null;
  computed?: EvidenceAnalysisResult | null;
  markers: PlotMarker[];
  /** When the execution asked and the operator answered; time series only. */
  promptMarkers?: PromptMarker[];
  plotType: DocumentPlotType;
  style: DocumentPlotStyle;
  xScale: DocumentAxisScale;
  yScale: DocumentAxisScale;
  fft: DocumentFftOptions;
  histogram?: DocumentHistogramOptions | null;
  /** A histogram's axes: the declared value on x, the host's count on y. */
  histogramAxes?: HistogramAxes | null;
  timeAxis: boolean;
  /** Declared Boolean Fields: False = 0, True = 1, Null = A Gap. */
  booleanAxis?: boolean;
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
  /** The backend's statistics of each trace over the shown window. */
  summaries?: SeriesSummary[] | null;
  /** A new window's statistics are in flight, so the shown ones are dimmed. */
  summaryPending?: boolean;
};

export type PlotViewportMode = "fit" | "live";
