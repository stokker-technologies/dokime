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

import type { ApplicationSettings, SettingsScope, SettingsSnapshot } from "@generated/application-settings";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { connectionError } from "@/app/native-session/errors";
import { useNativeSessionOptional } from "@/app/native-session/context";
import { preferencesCommands } from "@/lib/desktop/settings";
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
  const session = useNativeSessionOptional();
  return useQuery({
    queryKey: preferencesKey(scope, session?.identity.generation),
    queryFn: () => preferencesCommands.desktopSettingsRead(scope),
    retry: false,
  });
}

export function useSavePreferences(scope: SettingsScope) {
  const session = useNativeSessionOptional();
  const client = useQueryClient();
  return useMutation({
    mutationFn: ({ previous, settings }: { previous: SettingsSnapshot; settings: ApplicationSettings }) =>
      preferencesCommands.desktopSettingsUpdate(scope, { expected_revision: previous.revision, settings }),
    onSuccess: (snapshot) => {
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
  });
}

export function preferencesError(error: unknown): string {
  if (error && typeof error === "object" && "kind" in error) {
    if (error.kind === "conflict") return "Settings changed elsewhere. Retry to review the latest settings before editing again.";
    if (error.kind === "disconnected") return "The backend is disconnected. Local preferences remain available.";
    if ("message" in error) return String(error.message);
  }
  return connectionError(error);
}

/* === Private Functions === */

/* === Entry Point === */
