// Dokime — SPDX-License-Identifier: Apache-2.0

/* === Imports === */

import type { SettingDeclaration, SettingValue } from "@host/model";
import type { RuntimePluginSettings, SettingValues } from "@/features/settings/installed-plugins/installed-plugin-contracts";
import * as rpc from "@/lib/host/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import { required, type Wire } from "@dokime/plots/lib/rpc/canonical";
import { failureOf } from "@dokime/plots/lib/notification-details";

/* === Public Functions === */

export async function fetchNativeSettingsFrame(session: SessionIdentity) {
  const response = await rpc.settingsSchema(session, {});
  return required(response.schema, "Get Settings Schema");
}

export function mapNativeRuntimePluginSettings(
  settings: Wire<Awaited<ReturnType<typeof rpc.pluginsSettingsList>>>,
  requirements: Wire<Awaited<ReturnType<typeof rpc.pluginsSecretsList>>>,
): RuntimePluginSettings {
  return {
    settings: {
      contributions: [
        ...settings.declarations.map((declaration) => ({ kind: "setting" as const, declaration })),
        ...requirements.secrets.map((requirement) => {
          const declaration = required(requirement.declaration, "List Plugin Secret Declaration");
          const metadata = required(declaration.metadata, "List Plugin Secret Metadata");
          return { kind: "secret" as const, id: metadata.identifier, title: metadata.label, description: metadata.description, required: declaration.required };
        }),
      ],
    },
    values: Object.fromEntries(Object.entries(settings.value?.values ?? {}).map(([name, value]) => [name, readPluginSetting(value)])),
    secrets: requirements.secrets.map((requirement) => {
      const declaration = required(requirement.declaration, "List Plugin Secret Declaration");
      const metadata = required(declaration.metadata, "List Plugin Secret Metadata");
      return { identifier: metadata.identifier, available: requirement.available };
    }),
  };
}

export async function getNativePluginSettings(session: SessionIdentity, pluginId: string) {
  const [settings, requirements] = await Promise.all([
    rpc.pluginsSettingsList(session, { name: pluginId }),
    rpc.pluginsSecretsList(session, { name: pluginId }),
  ]);
  return mapNativeRuntimePluginSettings(settings, requirements);
}

/** Merge Only Edited Fields Into the Latest Complete Value at Its Issued Revision. */
export async function updateNativePluginSettings(session: SessionIdentity, pluginId: string, values: SettingValues) {
  try {
    requireCurrentSession(session);
    const current = await rpc.pluginsSettingsList(session, { name: pluginId });
    requireCurrentSession(session);
    const entries = current.declarations.map((declaration) => {
      const value = Object.hasOwn(values, declaration.name)
        ? values[declaration.name]
        : readPluginSetting(current.value?.values[declaration.name] ?? required(declaration.default, "Read Plugin Setting Default"));
      return [declaration.name, parsePluginSetting(declaration, value)] as const;
    });
    await rpc.pluginsSettingsSet(session, {
      name: pluginId,
      expectedRevision: current.revision,
      requestId: crypto.randomUUID(),
      value: { values: Object.fromEntries(entries) },
    });
    requireCurrentSession(session);
    return getNativePluginSettings(session, pluginId);
  } catch (error) {
    const failure = failureOf(error, "Settings Could Not Be Saved");
    throw { ...failure, action: failure.action || "Reload Plugin Settings, Review the Values and Save Again" };
  }
}

export function readPluginSetting(setting: Wire<SettingValue>): boolean | number | string {
  const value = setting.value;
  switch (value.case) {
    case "integer":
      return value.value.toString();
    case "flag":
    case "text":
    case "number":
      return value.value;
    case "duration": {
      const nanoseconds = value.value.seconds * 1_000_000_000n + BigInt(value.value.nanos);
      const absolute = nanoseconds < 0n ? -nanoseconds : nanoseconds;
      const fraction = String(absolute % 1_000_000_000n).padStart(9, "0").replace(/0+$/, "");
      return `${nanoseconds < 0n ? "-" : ""}${absolute / 1_000_000_000n}${fraction ? `.${fraction}` : ""}`;
    }
    default:
      throw new Error("Read Plugin Setting: Reload the Plugin to Obtain Its Declared Value");
  }
}

/** Parse and Validate With the Same Declaration Used to Render the Field. */
export function parsePluginSetting(declaration: Wire<SettingDeclaration>, value: unknown): Wire<SettingValue> {
  const kind = required(declaration.default, "Read Plugin Setting Default").value.case;
  const parsed = writeSetting(kind, value);
  for (const [bound, direction] of [[declaration.minimum, "minimum"], [declaration.maximum, "maximum"]] as const) {
    if (!bound) continue;
    const current = comparableSetting(parsed);
    const limit = comparableSetting(bound);
    if (bound.value.case !== kind || current === undefined || limit === undefined) {
      throw new Error("Validate Plugin Setting: Reload the Plugin to Obtain Valid Bounds");
    }
    if (direction === "minimum" ? current < limit : current > limit) {
      throw new Error(`Enter ${declaration.title || declaration.name} ${direction === "minimum" ? "at least" : "at most"} ${readPluginSetting(bound)}${kind === "duration" ? " s" : ""}.`);
    }
  }
  return parsed;
}

/* === Private Functions === */

function requireCurrentSession(session: SessionIdentity) {
  if (!session.isCurrent()) throw new Error("The Connection Changed. Reopen Plugin Settings on the Current Host Before Saving.");
}

function comparableSetting(setting: Wire<SettingValue>) {
  const value = setting.value;
  switch (value.case) {
    case "integer":
    case "number":
      return value.value;
    case "duration":
      return value.value.seconds * 1_000_000_000n + BigInt(value.value.nanos);
    default:
      return undefined;
  }
}

function writeSetting(kind: Wire<SettingValue>["value"]["case"], value: unknown): Wire<SettingValue> {
  switch (kind) {
    case "flag":
      if (typeof value === "boolean") {
        return { value: { case: "flag", value } };
      }
      break;
    case "text":
      if (typeof value === "string") {
        return { value: { case: "text", value } };
      }
      break;
    case "number": {
      const number = typeof value === "string" && /^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?$/i.test(value) ? Number(value) : value;
      if (typeof number === "number" && Number.isFinite(number)) {
        return { value: { case: "number", value: number } };
      }
      break;
    }
    case "integer": {
      if (typeof value !== "string" || !/^-?\d+$/.test(value)) break;
      const integer = BigInt(value);
      if (integer >= -(1n << 63n) && integer < 1n << 63n) {
        return { value: { case: "integer", value: integer } };
      }
      break;
    }
    case "duration": {
      if (typeof value !== "string") break;
      const match = /^(-?)(\d+)(?:\.(\d{1,9}))?$/.exec(value);
      if (!match) break;
      const sign = match[1] ? -1n : 1n;
      const seconds = sign * BigInt(match[2]);
      const nanos = Number(sign * BigInt((match[3] ?? "").padEnd(9, "0")));
      if (seconds >= -315_576_000_000n && seconds <= 315_576_000_000n) {
        return { value: { case: "duration", value: { seconds, nanos } } };
      }
      break;
    }
    default:
      break;
  }
  throw new Error(`Enter a Valid ${kind === "duration" ? "Duration in Seconds" : kind === "integer" ? "64-Bit Integer" : kind || "Setting Value"}.`);
}
