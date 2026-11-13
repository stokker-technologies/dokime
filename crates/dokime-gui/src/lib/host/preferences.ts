/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Host Preferences Projection
 */

/* === Imports === */

import { create, fromBinary, toBinary } from "@bufbuild/protobuf";
import type { ApplicationSettings, SettingsScope, PreferencesSnapshot, PreferencesUpdate } from "@generated/application-settings";
import {
  Density,
  ThemeMode,
  ApplicationSettingsSchema,
  ApplicationAppearanceSchema,
  ClientPreferencesSchema,
  PluginPreferencesSchema,
  ReportDefaultsSchema,
  TelemetryPreferencesSchema,
} from "@host/dokime/client/v1/settings_pb.js";
import { Downsample } from "@host/dokime/v1/plots_pb.js";
import { SettingsGetResponseSchema, SettingsSetRequestSchema } from "@host/dokime/client/v1/settings_pb.js";
import { invoke } from "../desktop/commands";
import { safeNumber } from "../api/native-values";

/* === Re-Exports === */

/* === Constants === */

const themeModes = { system: ThemeMode.SYSTEM, light: ThemeMode.LIGHT, dark: ThemeMode.DARK };
const densities = { comfortable: Density.COMFORTABLE, compact: Density.COMPACT, touch: Density.TOUCH };
const downsampling = {
  unspecified: Downsample.UNSPECIFIED,
  raw: Downsample.RAW,
  min_max: Downsample.MIN_MAX,
  lttb: Downsample.LTTB,
  auto: Downsample.AUTO,
};

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export async function readPreferences(scope: SettingsScope): Promise<PreferencesSnapshot> {
  return snapshot(await invoke<ArrayBuffer>("desktop_settings_read", { scope }));
}

export async function savePreferences(scope: SettingsScope, update: PreferencesUpdate): Promise<PreferencesSnapshot> {
  const settings = update.settings;
  const request = create(SettingsSetRequestSchema, {
    expectedRevision: new Uint8Array(update.expected_revision),
    requestId: crypto.randomUUID(),
    value: create(ApplicationSettingsSchema, {
      appearance: create(ApplicationAppearanceSchema, {
        themeMode: themeModes[settings.appearance.theme_mode],
        themeFamily: settings.appearance.theme_family ?? undefined,
        density: densities[settings.appearance.density],
      }),
      client: create(ClientPreferencesSchema, {
        connectTimeoutMs: BigInt(settings.client.connect_timeout_ms),
        requestTimeoutMs: BigInt(settings.client.request_timeout_ms),
      }),
      plugins: create(PluginPreferencesSchema, {
        startupTimeoutS: settings.plugins.startup_timeout_s,
        stopTimeoutS: settings.plugins.stop_timeout_s,
      }),
      report: create(ReportDefaultsSchema, {
        plotDownsample: downsampling[settings.report.plot_downsample],
        plotPoints: settings.report.plot_points,
        sourceBytes: settings.report.source_bytes,
        tableRows: settings.report.table_rows,
        valueDepth: settings.report.value_depth,
      }),
      telemetry: create(TelemetryPreferencesSchema, { intervalMs: BigInt(settings.telemetry.interval_ms) }),
    }),
  });
  return snapshot(
    await invoke<ArrayBuffer>("desktop_settings_update", { scope, update: [...toBinary(SettingsSetRequestSchema, request)] }),
  );
}

/* === Private Functions === */

function snapshot(bytes: ArrayBuffer): PreferencesSnapshot {
  const reply = fromBinary(SettingsGetResponseSchema, new Uint8Array(bytes));
  const settings = reply.value;
  if (!settings?.appearance || !settings.client || !settings.listener || !settings.plugins || !settings.report || !settings.telemetry)
    throw new Error("Read Preferences: Host Omitted a Settings Section; Reconnect and Retry");
  const themeMode = Object.entries(themeModes).find(([, value]) => value === settings.appearance?.themeMode)?.[0] as
    | ApplicationSettings["appearance"]["theme_mode"]
    | undefined;
  const density = Object.entries(densities).find(([, value]) => value === settings.appearance?.density)?.[0] as
    | ApplicationSettings["appearance"]["density"]
    | undefined;
  const plotDownsample = Object.entries(downsampling).find(([, value]) => value === settings.report?.plotDownsample)?.[0] as
    | ApplicationSettings["report"]["plot_downsample"]
    | undefined;
  if (!themeMode || !density || !plotDownsample)
    throw new Error("Read Preferences: Host Returned an Unsupported Preference; Update the Desktop");
  return {
    revision: [...reply.revision],
    settings: {
      appearance: { theme_mode: themeMode, theme_family: settings.appearance.themeFamily ?? null, density },
      client: {
        connect_timeout_ms: safeNumber(settings.client.connectTimeoutMs, "Connect Timeout"),
        request_timeout_ms: safeNumber(settings.client.requestTimeoutMs, "Request Timeout"),
      },
      listener: {
        address: settings.listener.address,
        enabled: settings.listener.enabled,
        tls_enabled: settings.listener.tlsEnabled,
        authentication_enabled: settings.listener.authenticationEnabled,
        certificate_names: settings.listener.certificateNames,
      },
      plugins: { startup_timeout_s: settings.plugins.startupTimeoutS, stop_timeout_s: settings.plugins.stopTimeoutS },
      report: {
        plot_points: settings.report.plotPoints,
        plot_downsample: plotDownsample,
        source_bytes: settings.report.sourceBytes,
        table_rows: settings.report.tableRows,
        value_depth: settings.report.valueDepth,
      },
      telemetry: { interval_ms: safeNumber(settings.telemetry.intervalMs, "Telemetry Interval") },
    },
  };
}

/* === Entry Point === */
