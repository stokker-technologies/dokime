/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === External Imports === */

import { Activity, CircleUser, Info, type LucideIcon, Network, PackagePlus, Palette, Settings } from "lucide-react";

/* === Constants === */

export const SETTINGS_SECTIONS: SettingsSection[] = [
  {
    id: "general",
    label: "General",
    description: "Execution storage and restart",
    icon: Settings,
    group: "primary",
  },
  {
    id: "remote",
    label: "Remote Access",
    description: "Listener, access key and public certificate",
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
  {
    id: "identity",
    label: "Identity",
    description: "Host and installed plugin identity",
    icon: CircleUser,
    group: "primary",
  },
  {
    id: "plugins",
    label: "Plugins",
    description: "Install plugins and configure their programs",
    icon: PackagePlus,
    group: "primary",
  },
  {
    id: "account-updates",
    label: "Account & Updates",
    description: "Sign-in and desktop updates",
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

export const DEFAULT_SETTINGS_SECTION: SettingsSectionId = "general";

/* === Types === */

export type SettingsSectionId = string;

export type SettingsSectionGroup = "primary" | "utility";

export type SettingsSection = {
  id: SettingsSectionId;
  label: string;
  description: string;
  icon: LucideIcon;
  group: SettingsSectionGroup;
};
