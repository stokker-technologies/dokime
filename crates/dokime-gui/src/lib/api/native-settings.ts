/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native settings adapters for the original settings surfaces. */

/* === External Imports === */

/* === Local Imports === */

import type { SettingValue } from "@host/model";
import { NativeIntegerValue } from "./native-values";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/host/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { RuntimePluginSettings, SettingsEntry } from "@/lib/types/view-contracts";

/* === Types === */

import { required } from "@/lib/rpc/canonical";

type PluginRuntimeSettings = Wire<Awaited<ReturnType<typeof rpc.pluginsSettingsList>>>;
type SettingContribution = PluginRuntimeSettings["declarations"][number];
type SettingValues = RuntimePluginSettings["values"];

/* === Settings frame === */

export async function fetchNativeSettingsFrame(session: SessionIdentity) {
  const response = await rpc.settingsSchema(session, {});
  return required(response.schema, "Get Settings Schema");
}

/* === Plugin Settings === */

export type NativePluginRuntimeSettings = PluginRuntimeSettings;

export function mapNativePluginRuntimeSettings(
  settings: NativePluginRuntimeSettings,
  requirements: Wire<Awaited<ReturnType<typeof rpc.pluginsSecretsList>>>,
): RuntimePluginSettings {
  const secrets = requirements.secrets.map((requirement) => {
    const declaration = required(requirement.declaration, "List Plugin Secret Declaration");
    return {
      identifier: required(declaration.metadata, "List Plugin Secret Metadata").identifier,
      available: requirement.available,
    };
  });
  return {
    settings: {
      contributions: [
        ...settings.declarations.map(mapSettingContribution),
        ...requirements.secrets.map((requirement): SettingsEntry => {
          const declaration = required(requirement.declaration, "List Plugin Secret Declaration");
          const metadata = required(declaration.metadata, "List Plugin Secret Metadata");
          return {
            kind: "secret",
            id: metadata.identifier,
            title: metadata.label,
            description: metadata.description ?? null,
            required: declaration.required,
          };
        }),
      ],
    },
    values: Object.fromEntries(Object.entries(settings.value?.values ?? {}).map(([name, value]) => [name, settingValue(value)])),
    secrets,
  };
}

export async function getNativePluginSettings(session: SessionIdentity, pluginId: string) {
  const [settings, requirements] = await Promise.all([
    rpc.pluginsSettingsList(session, { name: pluginId }),
    rpc.pluginsSecretsList(session, { name: pluginId }),
  ]);
  return mapNativePluginRuntimeSettings(settings, requirements);
}

export async function updateNativePluginSettings(session: SessionIdentity, pluginId: string, values: SettingValues) {
  const current = await rpc.pluginsSettingsList(session, { name: pluginId });
  const entries = current.declarations.map((declaration) => {
    const defaultValue = required(declaration.default, "Set Plugin Setting Default");
    const value = values[declaration.name];
    return [declaration.name, writeSetting(defaultValue, value)] as const;
  });
  await rpc.pluginsSettingsSet(session, {
    name: pluginId,
    expectedRevision: current.revision,
    requestId: crypto.randomUUID(),
    value: { values: Object.fromEntries(entries) },
  });
  return getNativePluginSettings(session, pluginId);
}

function settingValue(setting: Wire<SettingValue>): SettingValues[string] {
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
      return `${nanoseconds < 0n ? "-" : ""}${absolute / 1_000_000_000n}.${String(absolute % 1_000_000_000n).padStart(9, "0")}`;
    }
    default:
      throw new Error("Read Plugin Setting: Missing Typed Value");
  }
}

function writeSetting(defaultValue: Wire<SettingValue>, value: unknown): Wire<SettingValue> {
  switch (defaultValue.value.case) {
    case "flag":
      if (typeof value === "boolean") return { value: { case: "flag", value } };
      break;
    case "text":
      if (typeof value === "string") return { value: { case: "text", value } };
      break;
    case "number":
      if (typeof value === "number" && Number.isFinite(value)) return { value: { case: "number", value } };
      break;
    case "integer": {
      const decimal =
        value instanceof NativeIntegerValue
          ? value.decimal
          : typeof value === "number" && Number.isSafeInteger(value)
            ? String(value)
            : typeof value === "string"
              ? value
              : "";
      if (/^-?\d+$/.test(decimal)) {
        const integer = BigInt(decimal);
        if (integer >= -(1n << 63n) && integer < 1n << 63n) return { value: { case: "integer", value: integer } };
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
      return { value: { case: "duration", value: { seconds, nanos } } };
    }
    default:
      break;
  }
  throw new Error("Set Plugin Setting: Value Does Not Match Its Declared Type");
}

/* === Mapping === */

function mapSettingContribution(contribution: SettingContribution): SettingsEntry {
  const common = { id: contribution.name, title: contribution.title, description: contribution.description || null };
  const value = required(contribution.default, "Read Plugin Setting Default");
  switch (value.value.case) {
    case "flag":
      return { ...common, kind: "boolean", default: value.value.value };
    case "text":
      return { ...common, kind: "text_area", default: value.value.value, required: false };
    case "number":
      return { ...common, kind: "number", default: value.value.value, min: null, max: null, step: null };
    case "integer":
      return { ...common, kind: "text_area", default: value.value.value.toString(), required: true };
    case "duration":
      return { ...common, kind: "text_area", default: String(settingValue(value)), required: true };
    default:
      throw new Error("Read Plugin Setting: Missing Typed Default");
  }
}
