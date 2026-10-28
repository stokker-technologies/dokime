/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Installed-plugin contracts. */

/**
 * Presentation record for a code-defined plugin admitted by the native
 * runtime. This stays at the settings boundary instead of importing the
 * retired HTTP runtime-query payload types.
 */
export type RuntimeInstalledPluginRecord = {
  plugin_id: string;
  owner:
    | { kind: "host" }
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

import type { RuntimePluginSettings as PluginRuntimeSettings } from "@/lib/types/view-contracts";

export type {
  PluginRuntimeSecretBindings,
  RedactedSecretBinding,
  RuntimePluginSettings as PluginRuntimeSettings,
  SecretBackendKind,
  SecretReference,
  Settings,
  SettingsEntry,
} from "@/lib/types/view-contracts";

export type InstalledPluginRuntimeKind = RuntimeInstalledPluginRecord["runtime"];
export type SettingValues = PluginRuntimeSettings["values"];

/**
 * Composite plugin detail: the code-defined plugin record and its current
 * host-owned settings and secret readiness.
 */
export type InstalledPluginDetail = RuntimeInstalledPluginRecord & PluginRuntimeSettings;

export const RUNTIME_LABELS: Record<InstalledPluginRuntimeKind, string> = {
  code: "Code-defined",
};
