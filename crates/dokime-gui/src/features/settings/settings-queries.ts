/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { useQuery } from "@tanstack/react-query";

/* === Local Imports === */

import { useNativeSessionOptional } from "@/app/native-session/context";
import { fetchNativeSettingsFrame } from "@/lib/api/native-settings";

/* === Query Keys === */

export const settingsQueryKeys = {
  all: ["settings"] as const,
  frame: (generation?: string) =>
    generation ? ([...settingsQueryKeys.all, "frame", generation] as const) : ([...settingsQueryKeys.all, "frame"] as const),
};

/* === Hooks === */

export function useSettingsFrameQuery(ownerKey?: string, enabled = true) {
  const session = useNativeSessionOptional();
  return useQuery({
    enabled: enabled && session !== null,
    queryFn: () => {
      if (!session) throw new Error("A connected backend is required.");
      return fetchNativeSettingsFrame(session.identity);
    },
    queryKey: ownerKey
      ? [...settingsQueryKeys.frame(session?.identity.generation), ownerKey]
      : settingsQueryKeys.frame(session?.identity.generation),
  });
}
