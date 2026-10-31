/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import type { DocumentReferenceLine, DocumentViewPlot, PlotPayload, TelemetryStream, ValueFormat } from "@/lib/types";
import { isRecord } from "@/lib/utils";
import { elapsedColumnUnit } from "@/plots/axis";
import { plotColor } from "@/plots/colors";
import { histogramAxes } from "@/plots/histogram-axes";
import type { PlotCardSpec } from "@/plots/types";

/* === Public Functions === */

/** The card spec of the plot at `position` within the card titled `title`. */
export function documentPlotSpec({
  parameters,
  payload,
  plot,
  plotPalette,
  position,
  telemetry,
  title,
}: {
  parameters?: Record<string, unknown> | null;
  payload: PlotPayload | null | undefined;
  plot: DocumentViewPlot;
  plotPalette?: readonly string[] | null;
  position: number;
  telemetry?: Record<string, TelemetryStream> | null;
  title: string;
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

  return {
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
    unit: plot.unit ?? commonUnit(series),
    colors: series.map((_, index) => plotColor(title, position + index, plotPalette)),
    referenceLines: resolveReferenceLines(plot.reference_lines ?? [], plot, telemetry, parameters),
    referenceBands: plot.reference_bands ?? [],
    stateLabel: state.label,
    stateDetail: state.detail,
    contextSummary: plotContextSummary({ stream: plot.stream, telemetry, context: payload?.context }),
    context: {
      ...(payload?.context ?? {}),
      ...(plotContext({ stream: plot.stream, plotType: plot.plot_type, telemetry }) ?? {}),
    },
    liveWindowS: payload ? (payload.window_s ?? null) : (plot.live_window_s ?? null),
    xFormat,
    xElapsedUnit: resolvedElapsedUnit(plot, xFormat),
    yFormat: plot.y_format ?? series[0]?.format ?? null,
    runStartedAt: epochSecondsOf(payload?.run_started_at),
    cycleStartedAt: epochSecondsOf(payload?.cycle_started_at),
    summaries: payload?.summaries ?? null,
    summaryPending: false,
  };
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

function resolveReferenceLines(
  lines: DocumentReferenceLine[],
  plot: DocumentViewPlot,
  telemetry: Record<string, TelemetryStream> | null | undefined,
  parameters: Record<string, unknown> | null | undefined,
): DocumentReferenceLine[] {
  return lines.map((line) => {
    if (Number.isFinite(line.value)) {
      return line;
    }
    const value = referenceLineSourceValue(line, plot, telemetry, parameters);
    if (value === null) {
      return line;
    }
    const sourceScale = numberValue(line.source_scale) ?? 1.0;
    return { ...line, value: value * sourceScale };
  });
}

function referenceLineSourceValue(
  line: DocumentReferenceLine,
  plot: DocumentViewPlot,
  telemetry: Record<string, TelemetryStream> | null | undefined,
  parameters: Record<string, unknown> | null | undefined,
): number | null {
  const source = line.source;
  if (!isRecord(source)) {
    return null;
  }
  const sourceRecord: Record<string, unknown> = source;

  const directKind = textValue(sourceRecord.kind);
  if (directKind === "static") {
    return numberValue(sourceRecord.value);
  }
  if (directKind === "telemetry_value") {
    const stream = textValue(sourceRecord.stream) ?? plot.stream;
    const field = textValue(sourceRecord.field);
    return field ? numberValue(latestSample(telemetry, stream)?.[field]) : null;
  }

  const nestedSource = sourceRecord.source;
  const expressionSource = isRecord(nestedSource) ? nestedSource : sourceRecord;
  const expressionType = textValue(expressionSource.type);
  if (expressionType === "telemetry") {
    const stream = textValue(expressionSource.stream) ?? plot.stream;
    const field = textValue(expressionSource.field);
    return field ? numberValue(latestSample(telemetry, stream)?.[field]) : null;
  }
  if (expressionType === "parameter") {
    const field = textValue(expressionSource.name);
    return field ? numberValue(parameters?.[field]) : null;
  }
  return null;
}

function plotContext({
  stream,
  plotType,
  telemetry,
}: {
  stream: string;
  plotType: DocumentViewPlot["plot_type"];
  telemetry?: Record<string, TelemetryStream> | null;
}): Record<string, unknown> | undefined {
  if (plotType !== "wifi_channel_occupancy") return undefined;
  const latest = latestSample(telemetry, stream);
  if (!latest) return undefined;
  return {
    active_wifi_channel: latest.active_wifi_channel,
    active_wifi_band: latest.active_wifi_band,
    active_wifi_ssid: latest.active_wifi_ssid,
  };
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
  const valueField = plotType === "xy_value_heatmap" || plotType === "heatmap" ? "z" : "y";
  const hasFinitePoint = cartesian
    ? cartesian.point_count > 0
    : series.some((item) => item.points.some((point) => Number.isFinite(point[valueField])));
  if (hasFinitePoint) {
    return {};
  }

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
    label: series.some((item) => item.raw_points > 0) ? "Unavailable" : "Waiting For Samples",
    detail: note ?? "No numeric samples are available in the selected window.",
  };
}

function latestSample(telemetry: Record<string, TelemetryStream> | null | undefined, stream: string) {
  const samples = telemetry?.[stream]?.samples ?? [];
  return samples[samples.length - 1];
}

function numberValue(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function textValue(value: unknown): string | null {
  return typeof value === "string" && value.trim() ? value.trim() : null;
}

function firstText(...values: unknown[]): string | undefined {
  for (const value of values) {
    if (typeof value === "string" && value.trim()) {
      return value.trim();
    }
  }
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
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
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
