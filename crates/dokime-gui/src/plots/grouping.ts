/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { ReferenceBand, ReferenceLine, ReferenceValue } from "@/lib/document-contracts";
import { formatTitleWord } from "@/lib/format";
import { numericValue } from "@/lib/utils";
import type {
  DocumentReferenceBand,
  DocumentReferenceLine,
  DocumentViewPlot,
  PlotPayload,
  TelemetryStream,
  ValueFormat,
} from "@/lib/types";
import { elapsedColumnUnit } from "@/plots/axis";
import { plotColor, referenceColor } from "@/plots/colors";
import { histogramAxes } from "@/plots/histogram-axes";
import type { PlotCardSpec } from "@/plots/types";

/* === Public Functions === */

/** The card spec of one document plot; its series take the plot palette in order. */
export function documentPlotSpec({
  parameters,
  payload,
  plot,
  plotPalette,
  telemetry,
}: {
  parameters?: Record<string, unknown> | null;
  payload: PlotPayload | null | undefined;
  plot: DocumentViewPlot;
  plotPalette?: readonly string[] | null;
  telemetry?: Record<string, TelemetryStream> | null;
}): PlotCardSpec {
  const series = labelSeries({
    plot,
    series: payload?.series.length ? payload.series : placeholderSeries(plot.y, plot),
  });
  const state = payload?.computed
    ? {}
    : plotState({
        stream: plot.stream,
        plotType: plot.plot_type,
        series,
        cartesian: payload?.cartesian ?? null,
        telemetry,
      });

  // The payload carries the resolved winner of the format precedence chain
  // (§2.3); the document plot's own declaration is the placeholder value
  // before a payload has arrived, so a plot never renders unformatted while
  // it waits for its first frame.
  const xFormat = payload?.x_format ?? plot.x_format ?? null;

  const spec: PlotCardSpec = {
    key: plot.plot_id,
    stream: plot.stream,
    title: plot.title,
    description: plot.description,
    series,
    cartesian: payload?.cartesian ?? null,
    analysis: payload?.analysis ?? null,
    computed: payload?.computed ?? null,
    markers: payload?.markers ?? [],
    plotType: plot.plot_type,
    arrangement: plot.arrangement,
    pareto: plot.pareto,
    hole: plot.hole,
    xyInterval: plot.xy_interval,
    render: plot.render ?? "cells",
    style: plot.style ?? "line",
    xScale: plot.x_scale ?? "linear",
    yScale: plot.y_scale ?? "linear",
    fft: plot.fft,
    histogram: plot.histogram,
    histogramAxes:
      plot.plot_type === "histogram"
        ? histogramAxes(
            plot,
            telemetry?.[plot.stream]?.spec?.columns.find((column) => column.name === plot.x),
          )
        : null,
    timeAxis: plot.plot_type === "time_series",
    booleanAxis:
      plot.plot_type === "time_series" &&
      plot.y.length > 0 &&
      !plot.y_expressions?.some((expression) => Array.isArray(expression.transforms) && expression.transforms.length > 0) &&
      plot.y.every((field) =>
        telemetry?.[plot.stream]?.spec?.columns.some((column) => column.name === field && column.value_type === "boolean"),
      ),
    unit: plot.unit ?? commonUnit(series),
    colors: series.map((_, index) => plotColor(index, plotPalette)),
    referenceLines: resolveReferenceLines(plot.reference_lines ?? [], plot.reference_bands?.length ?? 0, plot, telemetry, parameters),
    referenceBands: resolveReferenceBands(plot.reference_bands ?? [], plot, telemetry, parameters),
    stateLabel: state.label,
    stateDetail: state.detail,

    contextSummary:
      plot.plot_type === "contour" || plot.plot_type === "grid_chart"
        ? undefined
        : plotContextSummary({ stream: plot.stream, telemetry, context: payload?.context }),
    context: {
      ...(payload?.context ?? {}),
    },
    windowS: payload?.window_s ?? null,
    xFormat,
    xElapsedUnit: resolvedElapsedUnit(plot, xFormat),
    yFormat: plot.y_format ?? series[0]?.format ?? null,
    runStartedAt: epochSecondsOf(payload?.run_started_at),
    cycleStartedAt: epochSecondsOf(payload?.cycle_started_at),
    summaries: payload?.summaries ?? null,
    summaryPending: false,
  };
  return scalePresentation(spec, plot.scale ?? 1);
}

/**
 * The unit an elapsed x axis counts in, or `null` for a wall-time axis.
 *
 * Precedence, and the whole point of the declaration: a declared `ValueFormat`
 * for x wins, because an author who named one has said what the axis prints.
 * Then the plot's own `x_axis` — the wire now carries what the axis measures,
 * so nothing is inferred. The x column's *name* is consulted only for a
 * payload that carries no declaration at all, and only affirmatively; see
 * `elapsedColumnUnit`.
 */
export function resolvedElapsedUnit(plot: DocumentViewPlot, xFormat: ValueFormat | null): string | null {
  if (plot.plot_type !== "time_series" || xFormat) return null;
  const declared = plot.x_axis;
  if (declared) return declared.kind === "elapsed" ? declared.unit : null;
  return elapsedColumnUnit(plot.x);
}

/** Whether the group header would only repeat its single plot's own title. */
export function groupRepeatsPlotTitle(title: string, plots: readonly DocumentViewPlot[]): boolean {
  return plots.length === 1 && plots[0].title.trim().toLowerCase() === title.trim().toLowerCase();
}

export function documentPlotGroupDescription(title: string): string {
  return `Additional numeric telemetry for ${title}.`;
}

export { plotColor };

/* === Private Functions === */

/** Shared by live snapshots, zoom summaries and frozen HTML; never mutates query payloads. */
function scalePresentation(plot: PlotCardSpec, scale: number): PlotCardSpec {
  if (scale === 1) return plot;
  const scaled = (value: number | null | undefined) => (value == null ? value : value * scale);
  const cartesian = plot.cartesian;
  return {
    ...plot,
    booleanAxis: false,
    series: plot.series.map((series) => ({
      ...series,
      points: series.points.map((point) => ({ ...point, y: point.y === null ? null : point.y * scale })),
    })),
    cartesian: cartesian
      ? {
          ...cartesian,
          series: cartesian.series.map((series) => ({ ...series, y: series.y.map((value) => (value === null ? null : value * scale)) })),
          y_domain: cartesian.y_domain ? { min: cartesian.y_domain.min * scale, max: cartesian.y_domain.max * scale } : cartesian.y_domain,
          signature: `${cartesian.signature}:scale:${scale}`,
        }
      : cartesian,
    summaries: plot.summaries?.map((summary) =>
      Object.fromEntries(
        Object.entries(summary).map(([key, value]) => [key, key !== "count" && typeof value === "number" ? value * scale : value]),
      ),
    ) as PlotCardSpec["summaries"],
    referenceLines: plot.referenceLines.map((line) => ({ ...line, value: scaled(line.value) })),
    referenceBands: plot.referenceBands.map((band) => ({ ...band, min: scaled(band.min), max: scaled(band.max) })),
  };
}

/** An RFC3339 payload timestamp as epoch seconds, or `null` when absent or unparseable. */
function epochSecondsOf(timestamp: string | null | undefined): number | null {
  if (!timestamp) {
    return null;
  }
  const parsed = Date.parse(timestamp) / 1000;
  return Number.isFinite(parsed) ? parsed : null;
}

function commonUnit(series: PlotPayload["series"]): string | null {
  const units = new Set(series.map((item) => item.unit ?? null));
  return units.size === 1 ? (series[0]?.unit ?? null) : null;
}

/** Lines continue the reference palette after the plot's bands. */
function resolveReferenceLines(
  lines: ReferenceLine[],
  bandCount: number,
  plot: DocumentViewPlot,
  telemetry: Record<string, TelemetryStream> | null | undefined,
  parameters: Record<string, unknown> | null | undefined,
): DocumentReferenceLine[] {
  return lines.map((line, index) => ({
    ...line,
    color: referenceColor(line.color, bandCount + index),
    value: referenceValue(line.value, plot, telemetry, parameters),
  }));
}

function resolveReferenceBands(
  bands: ReferenceBand[],
  plot: DocumentViewPlot,
  telemetry: Record<string, TelemetryStream> | null | undefined,
  parameters: Record<string, unknown> | null | undefined,
): DocumentReferenceBand[] {
  return bands.map((band, index) => {
    let min = referenceValue(band.min, plot, telemetry, parameters);
    let max = referenceValue(band.max, plot, telemetry, parameters);
    if (band.center != null || band.tolerance != null) {
      const center = referenceValue(band.center, plot, telemetry, parameters);
      const tolerance = referenceValue(band.tolerance, plot, telemetry, parameters);
      min = center !== null && tolerance !== null && tolerance >= 0 ? center - tolerance : null;
      max = center !== null && tolerance !== null && tolerance >= 0 ? center + tolerance : null;
    }
    if (min === null || max === null || !Number.isFinite(min) || !Number.isFinite(max) || min > max) {
      min = null;
      max = null;
    }
    return { label: band.label, color: referenceColor(band.color, index), min, max };
  });
}

/** One resolution path for live views and serialized frozen documents. */
function referenceValue(
  value: ReferenceValue | null | undefined,
  plot: DocumentViewPlot,
  telemetry: Record<string, TelemetryStream> | null | undefined,
  parameters: Record<string, unknown> | null | undefined,
): number | null {
  if (typeof value === "number") return Number.isFinite(value) ? value : null;
  if (!value) return null;
  const source = value.source;
  let result: number | null = null;
  if (source.type === "telemetry") {
    const stream = source.stream ?? plot.stream;
    result = numberValue(latestSample(telemetry, stream)?.[source.field]);
  }
  if (source.type === "parameter") result = numberValue(parameters?.[source.name]);
  for (const transform of value.transforms ?? []) {
    if (result === null) return null;
    if (transform.type === "scale") result *= transform.factor;
    else if (transform.type === "offset") result += transform.value;
    else return null;
    if (!Number.isFinite(result)) return null;
  }
  return result;
}

function placeholderSeries(keys: string[], plot: DocumentViewPlot): PlotPayload["series"] {
  return keys.map((key) => ({
    key,
    label: fieldLabel(key, plot),
    points: [],
    raw_points: 0,
    downsampled: false,
  }));
}

function labelSeries({ plot, series }: { plot: DocumentViewPlot; series: PlotPayload["series"] }): PlotPayload["series"] {
  if (plot.series_by) {
    return series;
  }
  return series.map((item) => ({
    ...item,
    label: fieldLabel(item.key, plot),
  }));
}

function fieldLabel(key: string, plot: DocumentViewPlot): string {
  const explicit = FIELD_LABELS[key];
  if (explicit) {
    return explicit;
  }
  if (plot.y.length === 1 && key === plot.y[0]) {
    return plot.title;
  }
  return titleCase(key);
}

function plotState({
  stream,
  plotType,
  series,
  cartesian,
  telemetry,
}: {
  stream: string;
  plotType: DocumentViewPlot["plot_type"];
  series: PlotPayload["series"];
  /** A cartesian frame is the plot's only copy of its samples when present. */
  cartesian: PlotPayload["cartesian"];
  telemetry?: Record<string, TelemetryStream> | null;
}): { label?: string; detail?: string } {
  const valueField = plotType === "contour" ? "z" : "y";
  const hasFinitePoint = cartesian
    ? cartesian.point_count > 0
    : series.some((item) => item.points.some((point) => Number.isFinite(point[valueField])));
  if (hasFinitePoint) {
    return {};
  }
  if (plotType === "grid_chart" || plotType === "contour") return {};

  const latest = latestSample(telemetry, stream);
  if (stream === "host.wifi.scan") {
    const scanStatus = normalizedText(latest?.scan_status);
    const scanError = firstText(latest?.scan_error);
    if (scanStatus === "error") {
      return {
        label: "Scan Error",
        detail: scanError ?? "WiFi stream scan failed.",
      };
    }
    if (scanStatus === "unavailable") {
      return {
        label: "No Scan Results",
        detail: "No nearby WiFi networks were returned by the host scan.",
      };
    }
  }
  const status = normalizedText(
    latest?.[`${stream}_link_status`],
    latest?.wifi_state,
    latest?.wifi_link_status,
    latest?.bluetooth_state,
    latest?.bluetooth_link_status,
  );
  const note = firstText(latest?.diagnostic_note);
  if (status === "disconnected") {
    return {
      label: "Disconnected",
      detail: note ?? `${titleCase(stream)} telemetry is temporarily unavailable.`,
    };
  }
  if (status === "unknown") {
    return {
      label: "Unknown",
      detail: note ?? `${titleCase(stream)} status is unknown.`,
    };
  }
  if (status === "connected") {
    return {
      label: "Unavailable",
      detail: note ?? "No numeric samples are available in the selected window.",
    };
  }
  return {
    label: series.some((item) => item.raw_points > 0) ? "Unavailable" : "Waiting for Samples",
    detail: note ?? "No numeric samples are available in the selected window.",
  };
}

function latestSample(telemetry: Record<string, TelemetryStream> | null | undefined, stream: string) {
  const samples = telemetry?.[stream]?.samples ?? [];
  return samples[samples.length - 1];
}

function numberValue(value: unknown): number | null {
  const number = numericValue(value);
  return number !== null && Number.isFinite(number) ? number : null;
}

function firstText(...values: unknown[]): string | undefined {
  for (const value of values) {
    if (typeof value === "string" && value.trim()) {
      return value.trim();
    }
  }
}

function plotContextSummary({
  stream,
  telemetry,
  context,
}: {
  stream: string;
  telemetry?: Record<string, TelemetryStream> | null;
  context?: Record<string, unknown> | null;
}): string | undefined {
  const latest = latestSample(telemetry, stream) ?? context;
  if (!latest) {
    return undefined;
  }
  if (stream === "host.wifi.link") {
    return compactContext([
      textContext("SSID", latest.wifi_ssid),
      textContext("Radio", latest.wifi_radio ?? latest.wifi_state),
      textContext("Channel", latest.wifi_channel),
    ]);
  }
  if (stream === "host.wifi.latency") {
    return compactContext([textContext("Host", latest.wifi_latency_host), textContext("Probe", latest.probe_status)]);
  }
  if (stream === "host.bluetooth") {
    return compactContext([
      textContext("Adapter", latest.bluetooth_adapter),
      textContext("Radio", latest.bluetooth_radio_state),
      textContext("Probe", latest.probe_status),
    ]);
  }
  return undefined;
}

function compactContext(parts: Array<string | undefined>): string | undefined {
  const values = parts.filter((part): part is string => Boolean(part));
  return values.length ? values.join(" · ") : undefined;
}

function textContext(label: string, value: unknown): string | undefined {
  if (typeof value === "number" && Number.isFinite(value)) {
    return `${label} ${value}`;
  }
  if (typeof value === "string" && value.trim()) {
    return `${label} ${value.trim()}`;
  }
  return undefined;
}

function normalizedText(...values: unknown[]): string | undefined {
  return firstText(...values)?.toLowerCase();
}

function titleCase(value: string): string {
  return value
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/\bphy\b/gi, "PHY")
    .replace(/\brx\b/gi, "RX")
    .replace(/\btx\b/gi, "TX")
    .replace(/\bms\b/gi, "ms")
    .replace(/\bmbps\b/gi, "Mb/s")
    .split(/[\s_.:-]+/)
    .filter(Boolean)
    .map(formatTitleWord)
    .join(" ");
}

const FIELD_LABELS: Record<string, string> = {
  acceleration_x: "Acceleration X",
  acceleration_y: "Acceleration Y",
  acceleration_z: "Acceleration Z",
  battery_percent: "Battery",
  bluetooth_adapter_count: "Bluetooth Adapters",
  bluetooth_connected_device_count: "Connected Devices",
  bluetooth_present_device_count: "Present Devices",
  bluetooth_recv_mb_per_second: "Bluetooth RX Throughput",
  bluetooth_sent_mb_per_second: "Bluetooth TX Throughput",
  cpu_load_percent: "CPU Load",
  distance_m: "Distance",
  horizontal_error_m: "Horizontal Error",
  network_count: "Networks",
  position_x_m: "Position X",
  position_y_m: "Position Y",
  position_z_m: "Position Z",
  sequence: "Sequence",
  solution_state: "Solution State",
  speed_m_per_s: "Speed",
  system_uptime_s: "System Uptime",
  velocity_x_m_per_s: "Velocity X",
  velocity_y_m_per_s: "Velocity Y",
  wifi_channel: "WiFi Channel",
  wifi_latency_host: "WiFi Latency Host",
  wifi_latency_ms: "WiFi Latency",
  wifi_network_congestion_score: "WiFi Congestion Index",
  wifi_rx_mbps: "WiFi RX Throughput",
  wifi_rx_phy_mbps: "WiFi RX PHY Rate",
  wifi_tx_mbps: "WiFi TX Throughput",
  wifi_signal_percent: "WiFi Signal",
  wifi_tx_phy_mbps: "WiFi TX PHY Rate",
};
