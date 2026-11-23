/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Installed-plugin contracts. */

/* === Imports === */

import type { SettingDeclaration } from "@generated/host/model";
import type { Wire } from "@/lib/rpc/canonical";
import type { SecretSetting, Value } from "@/lib/types/settings";

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

export type SettingsEntry = { kind: "setting"; declaration: Wire<SettingDeclaration> } | ({ kind: "secret" } & SecretSetting);

export type Settings = { contributions?: SettingsEntry[] };

export type RuntimePluginSettings = {
  settings: Settings;
  values: Record<string, Value>;
  secrets: { identifier: string; available: boolean }[];
};

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
