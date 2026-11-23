/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/**
 * Scoped Settings State
 */

/* === Imports === */

import type { ApplicationSettings, SettingsScope } from "@generated/typescript/application-settings";
import { type QueryClient, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useNativeSessionOptional } from "@/app/native-session/context";
import type { ConnectedSession } from "@/app/native-session/session";
import { preferencesCommands } from "@/lib/desktop/settings";
import { withNativeListener, saveNativeListenerPreferences, type ScopedPreferencesSnapshot } from "@/lib/api/native-remote";
import { useUiStore } from "@/stores/ui-store";

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Re-Exports === */

/* === Constants === */

/* === Types === */

/* === Classes === */

/* === Public Functions === */

export function preferencesKey(scope: SettingsScope, generation?: string) {
  return ["settings", "preferences", scope, scope === "local" ? "local" : (generation ?? "offline")] as const;
}

export function usePreferences(scope: SettingsScope) {
  return useQuery(preferencesQuery(scope, useNativeSessionOptional()));
}

export function preferencesQuery(scope: SettingsScope, session: ConnectedSession | null) {
  return {
    queryKey: preferencesKey(scope, session?.identity.generation),
    queryFn: async () => {
      const snapshot = await preferencesCommands.desktopSettingsRead(scope);
      return scope === "backend" && session ? withNativeListener(session.identity, snapshot) : snapshot;
    },
    retry: false,
  };
}

export function useSavePreferences(scope: SettingsScope) {
  const session = useNativeSessionOptional();
  const client = useQueryClient();
  return useMutation(preferencesMutation(scope, session, client));
}

export function preferencesMutation(scope: SettingsScope, session: ConnectedSession | null, client: QueryClient) {
  return {
    mutationFn: ({ previous, settings }: { previous: ScopedPreferencesSnapshot; settings: ApplicationSettings }) => {
      if (scope === "backend" && session && JSON.stringify(previous.settings.listener) !== JSON.stringify(settings.listener)) {
        return saveNativeListenerPreferences(session.identity, previous, settings);
      }
      return preferencesCommands.desktopSettingsUpdate(scope, { expected_revision: previous.revision, settings });
    },
    onSuccess: (snapshot: ScopedPreferencesSnapshot) => {
      client.setQueryData(preferencesKey(scope, session?.identity.generation), snapshot);
      if (scope === "local") {
        const appearance = snapshot.settings.appearance;
        useUiStore.getState().setUiState({ density: appearance.density, themeFamily: appearance.theme_family });
      }
      void client.invalidateQueries({ queryKey: ["settings", "application"] });
      void client.invalidateQueries({ queryKey: ["settings", "remote-listener"] });
      void client.invalidateQueries({ queryKey: ["settings", "firewall"] });
      // A Local Backend Resolves Both Scopes To The Same Revision.
      void client.invalidateQueries({ queryKey: ["settings", "preferences"] });
    },
  };
}

/* === Private Functions === */

/* === Entry Point === */
