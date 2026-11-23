// Dokime — SPDX-License-Identifier: Apache-2.0
export type SettingsSectionId = string;
export const DEFAULT_SETTINGS_SECTION: SettingsSectionId = "backends";

export const SETTINGS_SECTIONS = [
  { id: "backends", label: "Backends" },
  { id: "remote", label: "Remote Access" },
  { id: "appearance", label: "Appearance" },
  { id: "plugins", label: "Plugins" },
  { id: "secrets", label: "Secret Storage" },
  { id: "engine", label: "Engine" },
  { id: "diagnostics", label: "Diagnostics" },
  { id: "about", label: "About" },
] as const;
