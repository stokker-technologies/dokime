/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The settings sections and the runtime frame behind them.
 *
 * The sidebar lists the sections and the surface renders the one that is
 * selected, so both read this one hook and agree on what exists, on whether
 * the runtime frame has been read, and on what to say when it could not be.
 * Plugin-contributed settings are not sections here: each plugin has its own
 * page under Settings › Plugins, so the navigation stays the fixed list.
 */

/* === External Imports === */

/* === Local Imports === */

import type { RuntimeSettingsFrame } from "@/features/settings/settings-contracts";
import { useSettingsFrameQuery } from "@/features/settings/settings-queries";
import { SETTINGS_SECTIONS, type SettingsSection } from "@/features/settings/settings-sections";

/* === Types === */

export type SettingsSectionsState = {
  /** The message to show when the frame could not be read; null otherwise. */
  error: string | null;
  frame: RuntimeSettingsFrame | null;
  /** True only while the first read is in flight. */
  loading: boolean;
  retry: () => void;
  /** The fixed navigation; plugins contribute pages, not sections. */
  sections: SettingsSection[];
};

/* === Public Functions === */

export function useSettingsSections(enabled = true): SettingsSectionsState {
  const query = useSettingsFrameQuery(undefined, enabled);
  const frame = query.data ?? null;

  return {
    error:
      query.error instanceof Error
        ? query.error.message
        : query.isError
          ? "Settings could not be loaded."
          : null,
    frame,
    loading: enabled && query.isPending,
    retry: () => void query.refetch(),
    sections: SETTINGS_SECTIONS,
  };
}
