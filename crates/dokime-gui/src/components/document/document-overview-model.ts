/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Imports === */

import { parseTimestampMs } from "@/lib/time-format";
import type { TelemetryStream } from "@/lib/types";
import { isRecord } from "@/lib/utils";

/* === DocumentOverview model: traceability/telemetry/progress formatting helpers === */

export function telemetryContextGroups(streams: TelemetryStream[]): Array<{
  title: string;
  description: string;
  streams: TelemetryStream[];
}> {
  const groups = [
    {
      title: "Tracking",
      description: "Target position, distance, stability, and marker data.",
      match: (stream: TelemetryStream) => stream.stream.includes("tracking"),
    },
    {
      title: "Host System",
      description: "CPU, memory, battery, and host process context.",
      match: (stream: TelemetryStream) => stream.stream === "host.system",
    },
    {
      title: "WiFi",
      description: "Link state, stream, signal, throughput, and scan occupancy.",
      match: (stream: TelemetryStream) => stream.stream.includes("wifi"),
    },
    {
      title: "Bluetooth",
      description: "Bluetooth adapters, devices, and link counters.",
      match: (stream: TelemetryStream) => stream.stream.includes("bluetooth"),
    },
  ];
  const matched = new Set<string>();
  const result = groups
    .map((group) => {
      const groupStreams = streams.filter((stream) => group.match(stream));
      groupStreams.forEach((stream) => {
        matched.add(stream.stream);
      });
      return { ...group, streams: groupStreams };
    })
    .filter((group) => group.streams.length)
    .map(({ title, description, streams }) => ({ title, description, streams }));
  const advanced = streams.filter((stream) => !matched.has(stream.stream));
  if (advanced.length) {
    result.push({
      title: "Advanced Raw",
      description: "Additional telemetry streams retained for diagnostics.",
      streams: advanced,
    });
  }
  return result;
}

export function usefulTelemetryEntries(
  stream: TelemetryStream,
  latest: Record<string, unknown>,
): Array<[string, unknown]> {
  const preferred = preferredTelemetryFields(stream.stream);
  const entries = preferred
    .filter((key) => latest[key] !== null && latest[key] !== undefined && latest[key] !== "")
    .map((key) => [key, latest[key]] as [string, unknown]);
  if (entries.length) {
    return entries.slice(0, 8);
  }

  return Object.entries(latest)
    .filter(
      ([key, value]) =>
        value !== null &&
        value !== undefined &&
        key !== "sample_index" &&
        key !== "timestamp" &&
        !key.endsWith("_id"),
    )
    .slice(0, 6);
}

export function preferredTelemetryFields(stream: string): string[] {
  if (stream === "host.system") {
    return [
      "cpu_load_percent",
      "memory_pressure_percent",
      "battery_percent",
      "battery_state",
      "battery_power_plugged",
    ];
  }
  if (stream === "host.wifi") {
    return [
      "wifi_state",
      "wifi_ssid",
      "wifi_channel",
      "wifi_band",
      "wifi_signal_percent",
      "wifi_network_congestion_score",
    ];
  }
  if (stream === "host.wifi.latency") {
    return ["probe_status", "wifi_latency_host", "wifi_latency_ms", "probe_error"];
  }
  if (stream === "host.wifi.channel_occupancy") {
    return [
      "scan_status",
      "active_wifi_channel",
      "active_wifi_band",
      "wifi_network_congestion_score",
      "wifi_channel",
      "wifi_band",
      "network_count",
      "scan_network_count",
      "scan_error",
    ];
  }
  if (stream.includes("bluetooth")) {
    return [
      "probe_status",
      "bluetooth_adapter_count",
      "bluetooth_connected_device_count",
      "bluetooth_present_device_count",
      "bluetooth_device_names",
    ];
  }
  if (stream.includes("tracking")) {
    return [
      "distance_m",
      "x_m",
      "y_m",
      "target_x_m",
      "target_y_m",
      "horizontal_angle_deg",
      "vertical_angle_deg",
    ];
  }
  return [];
}

export function telemetryFieldLabel(stream: TelemetryStream, key: string): string {
  return stream.spec?.columns.find((column) => column.name === key)?.label ?? key;
}

export function latestStreamSample(stream: TelemetryStream): Record<string, unknown> {
  const latest = stream.summary.latest_sample;
  if (isRecord(latest)) {
    return latest;
  }
  return stream.samples.at(-1) ?? {};
}

export function relativeTime(timestamp: string): string {
  const then = parseTimestampMs(timestamp);
  if (then === null) {
    return "-";
  }
  const diffSeconds = Math.max(0, Math.floor((Date.now() - then) / 1000));
  if (diffSeconds < 45) {
    return "just now";
  }
  const minutes = Math.floor(diffSeconds / 60);
  if (minutes < 60) {
    return `${minutes} min ago`;
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${hours} hr ago`;
  }
  const days = Math.floor(hours / 24);
  return `${days} d ago`;
}
