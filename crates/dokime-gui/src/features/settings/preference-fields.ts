// Dokime — SPDX-License-Identifier: Apache-2.0
import type { ApplicationSettings } from "@generated/typescript/application-settings";

export type PreferencesSection = "appearance" | "remote";
type PreferenceGroup = keyof ApplicationSettings;
export type PreferenceField = {
  group: PreferenceGroup;
  key: string;
  label: string;
  help: string;
  advanced?: boolean;
  unit?: string;
  max?: number;
  options?: string[];
  kind?: "boolean" | "list";
};

export const PREFERENCE_FIELDS: PreferenceField[] = [
  {
    group: "appearance",
    key: "theme_mode",
    label: "Theme Mode",
    help: "Choose the desktop color mode.",
    options: ["system", "light", "dark"],
  },
  {
    group: "appearance",
    key: "theme_family",
    label: "Theme Family",
    help: "Use a theme family, or leave empty for the organization default.",
  },
  {
    group: "appearance",
    key: "density",
    label: "Density",
    help: "Set the space around desktop controls.",
    options: ["comfortable", "compact", "touch"],
  },
  {
    group: "report",
    key: "plot_points",
    label: "Report Points",
    help: "Maximum points in newly generated plots.",
    unit: "points",
    max: 100000,
  },
  {
    group: "report",
    key: "plot_downsample",
    label: "Report Downsampling",
    help: "Reduction method for newly generated plots.",
    options: ["auto", "raw", "min_max", "lttb"],
  },
  {
    group: "client",
    key: "connect_timeout_ms",
    label: "Connect Timeout",
    help: "Time allowed to establish a new connection.",
    unit: "ms",
    advanced: true,
  },
  {
    group: "client",
    key: "request_timeout_ms",
    label: "Request Timeout",
    help: "Time allowed for a new request.",
    unit: "ms",
    advanced: true,
  },
  {
    group: "telemetry",
    key: "interval_ms",
    label: "Telemetry Interval",
    help: "Delay between telemetry samples.",
    unit: "ms",
    advanced: true,
  },
  {
    group: "listener",
    key: "enabled",
    label: "Listener Enabled",
    help: "Allow remote clients after restarting the listener.",
    kind: "boolean",
  },
  { group: "listener", key: "address", label: "Listener Address", help: "Host and port to bind, for example 0.0.0.0:8765." },
  { group: "listener", key: "tls_enabled", label: "Listener TLS", help: "Encrypt traffic using this host’s certificate.", kind: "boolean" },
  {
    group: "listener",
    key: "authentication_enabled",
    label: "Listener Authentication",
    help: "Require an access key from remote clients.",
    kind: "boolean",
  },
  {
    group: "listener",
    key: "certificate_names",
    label: "Certificate Names",
    help: "DNS names or IP addresses separated by commas. Regenerate the certificate to apply.",
    kind: "list",
    advanced: true,
  },
];
