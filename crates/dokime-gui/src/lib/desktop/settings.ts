/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* The desktop's one settings file (`src-tauri/src/settings.rs`): remembered
   parameter overrides and the stream settings the built-in host starts with.
   Application settings live in `@/lib/application-settings`. */

/* === Imports === */

import { createPreferencesCommands } from "@generated/typescript/native-preferences";
import { invoke } from "./commands";
import { readPreferences, savePreferences } from "../host/preferences";

/* === Public Constants === */

export const preferencesCommands = {
  ...createPreferencesCommands(invoke),
  desktopSettingsRead: readPreferences,
  desktopSettingsUpdate: savePreferences,
};

/* === Types === */

/** Stable Template ID, Then Stable Parameter ID, Then Edited Raw Input. */
export type ParameterOverride = { raw: string; valueType: string; unit: string | null };
export type ParameterOverrides = Record<string, Record<string, ParameterOverride>>;

/** The Saved File; Keys Are Named as the Standalone Host's Configuration Names Them. */
export type SavedSettings = { interval_window_ms: number; interval_lag_windows: number; parameters?: ParameterOverrides };

/** Only the Keys Present Change; the Rest of the File Is Kept. */
export type SavedSettingsChange = Partial<SavedSettings>;
