/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

/* === Local Imports === */

import { fetchNativeSettingsFrame } from "@/lib/api/native-settings";
import type { SessionIdentity } from "@/lib/rpc/transport";

/* === Query Keys === */

export const settingsQueryKeys = {
  all: ["settings"] as const,
  frame: (generation?: string) =>
    generation ? ([...settingsQueryKeys.all, "frame", generation] as const) : ([...settingsQueryKeys.all, "frame"] as const),
};

/* === Hooks === */

export function settingsFrameQueryOptions(identity: SessionIdentity | undefined, ownerKey?: string, enabled = true) {
  return {
    enabled: enabled && identity !== undefined,
    queryFn: () => {
      if (!identity) {
        throw new Error("Connect to a backend before reading its settings.");
      }
      return fetchNativeSettingsFrame(identity);
    },
    queryKey: ownerKey ? [...settingsQueryKeys.frame(identity?.generation), ownerKey] : settingsQueryKeys.frame(identity?.generation),
  };
}
