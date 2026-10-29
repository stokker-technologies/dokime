/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { Activity, CircleUser, Info, type LucideIcon, Network, PackagePlus, Palette } from "lucide-react";

export const SETTINGS_SECTIONS: SettingsSection[] = [
  { id: "backends", label: "Backends", description: "Saved backend connections", icon: Network, group: "primary" },
  {
    id: "remote",
    label: "Remote Access",
    description: "Listener, pairing files and paired clients",
    icon: Network,
    group: "primary",
  },
  {
    id: "appearance",
    label: "Appearance",
    description: "Theme, density, and brand identity",
    icon: Palette,
    group: "primary",
  },
  { id: "organization", label: "Organization", description: "Organization identity and pack", icon: Palette, group: "primary" },
  {
    id: "plugins",
    label: "Plugins",
    description: "Install plugins and configure their programs",
    icon: PackagePlus,
    group: "primary",
  },
  {
    id: "secrets",
    label: "Secrets",
    description: "Shared named secrets on the connected host",
    icon: CircleUser,
    group: "primary",
  },
  {
    id: "diagnostics",
    label: "Diagnostics",
    description: "Server health, lanes, and run folder",
    icon: Activity,
    group: "utility",
  },
  {
    id: "about",
    label: "About",
    description: "Version and licenses",
    icon: Info,
    group: "utility",
  },
];

export const DEFAULT_SETTINGS_SECTION: SettingsSectionId = "backends";

export type SettingsSectionId = string;

export type SettingsSectionGroup = "primary" | "utility";

export type SettingsSection = {
  id: SettingsSectionId;
  label: string;
  description: string;
  icon: LucideIcon;
  group: SettingsSectionGroup;
};
