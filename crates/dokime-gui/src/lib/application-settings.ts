/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Application settings owned by the connected backend: appearance preferences,
 * the plot defaults reports and live plots start from, and the data root.
 * Nothing is saved or read without a connected session.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

// Local Imports
import { useNativeSessionOptional } from "@/app/native-session/context";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";

type ApplicationSettings = Wire<Awaited<ReturnType<typeof rpc.settingsReadApplicationSettings>>>;
type AppearancePreference = NonNullable<ApplicationSettings["appearance"]>;
type ReportPreference = NonNullable<ApplicationSettings["report"]>;

import type { SessionIdentity } from "@/lib/rpc/transport";
import type { DocumentDownsampleTechnique } from "@/lib/types/plot";
import { DEFAULT_PLOT_MAX_POINTS } from "@/plots/reduce";

/* === Types === */

export type ThemeMode = "system" | "light" | "dark";
export type Density = "comfortable" | "compact" | "touch";
export type Appearance = {
  themeMode: ThemeMode;
  themeFamily: string | null;
  density: Density;
};
/** What a plot draws with when neither its author nor its operator chose. */
export type ReportDefaults = {
  plotPoints: number;
  plotDownsample: DocumentDownsampleTechnique;
};
export type ApplicationSettingsView = {
  appearance: Appearance;
  report: ReportDefaults;
  applicationRoot: string;
  configuredDataRoot: string | null;
  defaultDataRoot: string;
  effectiveDataRoot: string;
  restartRequired: boolean;
};

/* === Constants === */

const THEME_MODES = {
  system: "System",
  light: "Light",
  dark: "Dark",
} as const;
const DENSITIES = {
  comfortable: "Comfortable",
  compact: "Compact",
  touch: "Touch",
} as const;

const TECHNIQUES = {
  auto: "Auto",
  lttb: "Lttb",
  min_max: "MinMax",
  raw: "Raw",
} as const;

/** The defaults before the backend answers, or without one. */
export const BUILT_IN_REPORT_DEFAULTS: ReportDefaults = {
  plotPoints: DEFAULT_PLOT_MAX_POINTS,
  plotDownsample: "auto",
};

/* Saves are serialized so independent controls cannot overwrite each other. */
let pendingSave: Promise<unknown> = Promise.resolve();

/* === Public Functions === */

export const applicationSettingsKey = (generation: string) => ["settings", "application", generation] as const;

export function useApplicationSettings() {
  const session = useNativeSessionOptional();
  return useQuery({
    queryKey: applicationSettingsKey(session?.identity.generation ?? ""),
    queryFn: async () => {
      if (!session) throw new Error(DISCONNECTED);
      return settingsView(await rpc.settingsReadApplicationSettings(session.identity, {}));
    },
    enabled: session !== null,
    retry: false,
  });
}

/** Read, merge and write the patch so a partial change keeps the other fields. */
export function useSaveAppearance() {
  const session = useNativeSessionOptional();
  const cache = useSavedSettings();
  return useMutation({
    mutationFn: (patch: Partial<Appearance>) =>
      serialized(session?.identity, async (identity) => {
        const current = settingsView(await rpc.settingsReadApplicationSettings(identity, {}));
        return settingsView(
          await rpc.settingsUpdateAppearance(identity, {
            appearance: appearancePreference({ ...current.appearance, ...patch }),
          }),
        );
      }),
    onSuccess: cache,
  });
}

/** The plot defaults in force: the backend's once read, the built-in ones until then. */
export function useReportDefaults(): ReportDefaults {
  return useApplicationSettings().data?.report ?? BUILT_IN_REPORT_DEFAULTS;
}

export function useSaveReportDefaults() {
  const session = useNativeSessionOptional();
  const cache = useSavedSettings();
  return useMutation({
    mutationFn: (patch: Partial<ReportDefaults>) =>
      serialized(session?.identity, async (identity) => {
        const current = settingsView(await rpc.settingsReadApplicationSettings(identity, {}));
        const next = { ...current.report, ...patch };
        return settingsView(
          await rpc.settingsUpdateReportDefaults(identity, {
            report: {
              plotPoints: next.plotPoints,
              plotDownsample: {
                HarmosVariant: { case: TECHNIQUES[next.plotDownsample], value: {} },
              },
            },
          }),
        );
      }),
    onSuccess: cache,
  });
}

/** A null directory restores the application default. */
export function useSaveDataRoot() {
  const session = useNativeSessionOptional();
  const cache = useSavedSettings();
  return useMutation({
    mutationFn: (dataRoot: string | null) =>
      serialized(session?.identity, async (identity) =>
        settingsView(await rpc.settingsUpdateDataRoot(identity, { dataRoot: dataRoot ?? undefined })),
      ),
    onSuccess: cache,
  });
}

export function applicationSettingsError(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error && typeof error.message === "string") return error.message;
  return String(error);
}

/* === Private Functions === */

const DISCONNECTED = "Settings are available once the backend is connected";

function serialized<T>(identity: SessionIdentity | undefined, write: (identity: SessionIdentity) => Promise<T>): Promise<T> {
  const saved = pendingSave
    .catch(() => undefined)
    .then(() => {
      if (!identity) throw new Error(DISCONNECTED);
      if (!identity.isCurrent()) throw new Error("Backend connection changed");
      return write(identity);
    });
  pendingSave = saved;
  return saved;
}

function useSavedSettings() {
  const client = useQueryClient();
  const session = useNativeSessionOptional();
  return (view: ApplicationSettingsView) => {
    if (session?.identity.isCurrent()) client.setQueryData(applicationSettingsKey(session.identity.generation), view);
  };
}

function settingsView(wire: ApplicationSettings): ApplicationSettingsView {
  return {
    appearance: appearance(wire.appearance),
    report: reportDefaults(wire.report),
    applicationRoot: wire.applicationRoot,
    configuredDataRoot: wire.configuredDataRoot ?? null,
    defaultDataRoot: wire.defaultDataRoot,
    effectiveDataRoot: wire.effectiveDataRoot,
    restartRequired: wire.restartRequired,
  };
}

function appearance(wire: AppearancePreference | undefined): Appearance {
  return {
    themeMode: fromWire(THEME_MODES, wire?.themeMode?.HarmosVariant.case, "system"),
    themeFamily: wire?.themeFamily ?? null,
    density: fromWire(DENSITIES, wire?.density?.HarmosVariant.case, "comfortable"),
  };
}

function reportDefaults(wire: ReportPreference | undefined): ReportDefaults {
  return {
    plotPoints: wire?.plotPoints || BUILT_IN_REPORT_DEFAULTS.plotPoints,
    plotDownsample: fromWire(TECHNIQUES, wire?.plotDownsample?.HarmosVariant.case, BUILT_IN_REPORT_DEFAULTS.plotDownsample),
  };
}

function appearancePreference(value: Appearance): AppearancePreference {
  return {
    themeMode: { HarmosVariant: { case: THEME_MODES[value.themeMode], value: {} } },
    themeFamily: value.themeFamily ?? undefined,
    density: { HarmosVariant: { case: DENSITIES[value.density], value: {} } },
  };
}

function fromWire<K extends string, V>(table: Record<K, V>, wire: V | undefined, fallback: K): K {
  return (Object.keys(table) as K[]).find((key) => table[key] === wire) ?? fallback;
}

/* === Entry Point === */
