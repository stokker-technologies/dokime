/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Parameter overrides the desktop shell persists, mirroring the types beside
   the parameter commands in `src-tauri/src/app`. Application settings live in
   `@/lib/application-settings`. */

/* === Imports === */

import { createPreferencesCommands } from "@generated/native-preferences";
import { invoke } from "./commands";

/* === Public Constants === */

export const preferencesCommands = createPreferencesCommands(invoke);

/* === Types === */

/** Stable Template ID, Then Stable Parameter ID, Then Edited Raw Input. */
export type ParameterOverride = string | { raw: string; valueType: string; unit: string | null };
export type ParameterOverrides = Record<string, Record<string, ParameterOverride>>;
