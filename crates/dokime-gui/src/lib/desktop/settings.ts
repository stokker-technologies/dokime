/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Parameter overrides and interval settings the desktop shell persists, mirroring
   the types beside their commands in `src-tauri/src`. Application settings live in
   `@/lib/application-settings`. */

/* === Imports === */

import { createPreferencesCommands } from "@generated/native-preferences";
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

/** The Interval Window and Lag Bound the Built-In Host Starts With (`src-tauri/src/interval_settings.rs`). */
export type IntervalSettings = { interval_window_ms: number; interval_lag_windows: number };
