/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Installed-plugin contracts. */

/* === Types === */

/**
 * Presentation record for a code-defined plugin admitted by the native
 * runtime. This stays at the settings boundary instead of importing the
 * retired HTTP runtime-query payload types.
 */
export type RuntimeInstalledPluginRecord = {
  plugin_id: string;
  owner:
    | { kind: "host" }
    | { kind: "remote"; url: string }
    | { kind: "embedded"; generation: string }
    | {
        kind: "bundle";
        package: string;
        path: string | null;
        installation: string | null;
        content_digest?: string;
      }
    | null;
  label: string;
  version: string;
  enabled: boolean;
  runtime: string;
  status: string;
  error: string | null;
};

import type { RuntimePluginSettings } from "@dokime/plots/lib/types/settings";

export type {
  RuntimePluginSettings,
  Settings,
  SettingsEntry,
} from "@dokime/plots/lib/types/settings";

export type InstalledPluginRuntimeKind = RuntimeInstalledPluginRecord["runtime"];
export type SettingValues = RuntimePluginSettings["values"];

/**
 * Composite plugin detail: the code-defined plugin record and its current
 * host-owned settings and secret readiness.
 */
export type InstalledPluginDetail = RuntimeInstalledPluginRecord & RuntimePluginSettings;

/* === Constants === */

export const RUNTIME_LABELS: Record<InstalledPluginRuntimeKind, string> = {
  code: "Code-Defined",
  managed: "Managed",
  remote: "Remote",
};
