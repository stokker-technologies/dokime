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

import { HostCardTone, SettingsCardState as HostSettingsCardState } from "@host/model";
import type { SettingValue } from "@host/model";
import { NativeIntegerValue } from "./native-values";
import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/host/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type {
  HostCardBlock,
  RuntimePluginSettings as PluginSettingsView,
  RuntimeSettingsCard,
  RuntimeSettingsFrame,
  RuntimeSettingsSection,
  SettingsEntry,
} from "@/lib/types/view-contracts";

import { safeNumber } from "./native-values";

/* === Types === */

type SettingsFrame = Wire<NonNullable<Awaited<ReturnType<typeof rpc.settingsSchema>>["schema"]>>;
import { required } from "@/lib/rpc/canonical";
type SettingsSection = SettingsFrame["sections"][number];
type SettingsCard = SettingsFrame["cards"][number];
type SettingsCardState = SettingsCard["state"];

type PluginRuntimeSettings = Wire<Awaited<ReturnType<typeof rpc.pluginsSettingsList>>>;
type SettingContribution = PluginRuntimeSettings["declarations"][number];
type SettingValues = PluginSettingsView["values"];

/* === Settings frame === */

export async function fetchNativeSettingsFrame(session: SessionIdentity): Promise<RuntimeSettingsFrame> {
  const frame = required((await rpc.settingsSchema(session, {})).schema, "Get Settings Schema");
  const paths = frame.bootstrapPaths;
  return {
    bootstrap_paths: paths ? { application: paths.application, storage: paths.storage, plugins: paths.plugins, logs: paths.logs } : null,
    sections: frame.sections.map(mapSettingsSection),
    cards: frame.cards.map(mapSettingsCard),
  };
}

/* === Plugin Settings === */

export type NativePluginRuntimeSettings = PluginRuntimeSettings;

export function mapNativePluginRuntimeSettings(
  settings: NativePluginRuntimeSettings,
  requirements: Wire<Awaited<ReturnType<typeof rpc.pluginsSecretsList>>>,
): PluginSettingsView {
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

function mapSettingsSection(section: SettingsSection): RuntimeSettingsSection {
  const owner =
    section.owner?.kind.case === "app"
      ? { kind: "app" as const }
      : section.owner?.kind.case === "installedPluginId"
        ? { kind: "installed_plugin" as const, plugin_id: section.owner.kind.value }
        : null;
  if (!owner) throw new Error(`Native settings section ${section.sectionId} has no owner.`);
  return {
    section_id: section.sectionId,
    label: section.label,
    description: section.description ?? null,
    owner,
    order: section.order,
  };
}

function mapSettingsCard(card: SettingsCard): RuntimeSettingsCard {
  return {
    plugin_id: card.pluginId,
    card_id: card.cardId,
    section_id: card.sectionId,
    label: card.label,
    description: card.description ?? null,
    body: { blocks: card.body?.blocks.map(mapHostCardBlock) ?? [] },
    refresh: card.refresh ? mapRefreshPolicy(card.refresh) : undefined,
    state: mapSettingsCardState(card.state),
    order: card.order,
  };
}

function mapHostCardBlock(block: NonNullable<SettingsCard["body"]>["blocks"][number]): HostCardBlock {
  const variant = block.kind?.kind;
  switch (variant?.case) {
    case "text": {
      const entry = variant.value;
      return { type: "text", text: entry };
    }
    case "status": {
      const entry = variant.value;
      if (!entry) throw new Error("Native settings omitted its status payload.");
      return {
        type: "status",
        label: entry.label,
        tone: mapHostCardTone(entry.tone),
        detail: entry.detail ?? null,
      };
    }
    case "keyValueList": {
      const entry = variant.value;
      if (!entry) throw new Error("Native settings omitted its keyValueList payload.");
      return {
        type: "key_value_list",
        items: entry.items.map((item) => ({
          label: item.label,
          value: item.value,
          detail: item.detail ?? null,
        })),
      };
    }
    case "metrics": {
      const entry = variant.value;
      if (!entry) throw new Error("Native settings omitted its metrics payload.");
      return {
        type: "metrics",
        items: entry.items.map((item) => ({
          label: item.label,
          value: item.value,
          unit: item.unit ?? null,
          trend: item.trend ?? null,
        })),
      };
    }
    default:
      throw new Error("Native settings returned a card block without a kind.");
  }
}

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

function mapRefreshPolicy(policy: NonNullable<SettingsCard["refresh"]>): RuntimeSettingsCard["refresh"] {
  const variant = policy.policy?.kind;
  switch (variant?.case) {
    case "manual":
      return { kind: "manual" };
    case "cached": {
      const cached = variant.value;
      if (!cached) throw new Error("Native settings omitted its cached refresh policy.");
      return {
        kind: "cached",
        max_age_ms: safeNumber(cached.maxAgeMs, "Settings cache age"),
      };
    }
    case "onDemand":
      return { kind: "on_demand" };
    default:
      throw new Error("Native settings returned a refresh policy without a kind.");
  }
}

function mapHostCardTone(tone: HostCardTone): "neutral" | "info" | "success" | "warning" | "danger" {
  switch (tone) {
    case HostCardTone.NEUTRAL:
      return "neutral";
    case HostCardTone.INFO:
      return "info";
    case HostCardTone.SUCCESS:
      return "success";
    case HostCardTone.WARNING:
      return "warning";
    case HostCardTone.DANGER:
      return "danger";
    default:
      return "neutral";
  }
}

function mapSettingsCardState(state: SettingsCardState): RuntimeSettingsCard["state"] {
  switch (state) {
    case HostSettingsCardState.READY:
      return "ready";
    case HostSettingsCardState.DISABLED:
      return "disabled";
    case HostSettingsCardState.DISCONNECTED:
      return "disconnected";
    case HostSettingsCardState.STALE:
      return "stale";
    case HostSettingsCardState.ERROR:
      return "error";
    case HostSettingsCardState.PERMISSION_DENIED:
      return "permission_denied";
    default:
      throw new Error("Native settings returned an unknown card state.");
  }
}
