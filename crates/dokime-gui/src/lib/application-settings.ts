/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Application settings owned by the connected backend: appearance preferences
 * and the data root. Nothing is saved or read without a connected session.
 */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

// Local Imports
import { useNativeSessionOptional } from "@/app/native-session/NativeSessionProvider";
import * as rpc from "@/lib/rpc/generated/client";
import {
  type AppearancePreference,
  type ApplicationSettings,
  Density as WireDensity,
  ThemeMode as WireThemeMode,
} from "@/lib/rpc/generated/dokime/v1/settings_pb";
import type { SessionIdentity } from "@/lib/rpc/transport";

/* === Types === */

export type ThemeMode = "system" | "light" | "dark";
export type Density = "comfortable" | "compact" | "touch";
export type Appearance = {
  themeMode: ThemeMode;
  themeFamily: string | null;
  density: Density;
};
export type ApplicationSettingsView = {
  appearance: Appearance;
  applicationRoot: string;
  configuredDataRoot: string | null;
  defaultDataRoot: string;
  effectiveDataRoot: string;
  restartRequired: boolean;
};

/* === Constants === */

const THEME_MODES: Record<ThemeMode, WireThemeMode> = {
  system: WireThemeMode.SYSTEM,
  light: WireThemeMode.LIGHT,
  dark: WireThemeMode.DARK,
};
const DENSITIES: Record<Density, WireDensity> = {
  comfortable: WireDensity.COMFORTABLE,
  compact: WireDensity.COMPACT,
  touch: WireDensity.TOUCH,
};

/* Saves are serialized so independent controls cannot overwrite each other. */
let pendingSave: Promise<unknown> = Promise.resolve();

/* === Public Functions === */

export const applicationSettingsKey = (generation: string) =>
  ["settings", "application", generation] as const;

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

/** A null directory restores the application default. */
export function useSaveDataRoot() {
  const session = useNativeSessionOptional();
  const cache = useSavedSettings();
  return useMutation({
    mutationFn: (dataRoot: string | null) =>
      serialized(session?.identity, async (identity) =>
        settingsView(
          await rpc.settingsUpdateDataRoot(identity, { dataRoot: dataRoot ?? undefined }),
        ),
      ),
    onSuccess: cache,
  });
}

export function applicationSettingsError(error: unknown): string {
  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof error.message === "string"
  )
    return error.message;
  return String(error);
}

/* === Private Functions === */

const DISCONNECTED = "Settings are available once the backend is connected";

function serialized<T>(
  identity: SessionIdentity | undefined,
  write: (identity: SessionIdentity) => Promise<T>,
): Promise<T> {
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
    if (session?.identity.isCurrent())
      client.setQueryData(applicationSettingsKey(session.identity.generation), view);
  };
}

function settingsView(wire: ApplicationSettings): ApplicationSettingsView {
  return {
    appearance: appearance(wire.appearance),
    applicationRoot: wire.applicationRoot,
    configuredDataRoot: wire.configuredDataRoot ?? null,
    defaultDataRoot: wire.defaultDataRoot,
    effectiveDataRoot: wire.effectiveDataRoot,
    restartRequired: wire.restartRequired,
  };
}

function appearance(wire: AppearancePreference | undefined): Appearance {
  return {
    themeMode: fromWire(THEME_MODES, wire?.themeMode, "system"),
    themeFamily: wire?.themeFamily ?? null,
    density: fromWire(DENSITIES, wire?.density, "comfortable"),
  };
}

function appearancePreference(value: Appearance): AppearancePreference {
  return {
    themeMode: THEME_MODES[value.themeMode],
    themeFamily: value.themeFamily ?? undefined,
    density: DENSITIES[value.density],
  } as AppearancePreference;
}

function fromWire<K extends string, V>(table: Record<K, V>, wire: V | undefined, fallback: K): K {
  return (Object.keys(table) as K[]).find((key) => table[key] === wire) ?? fallback;
}

/* === Entry Point === */
