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

import * as v1 from "@host/dokime/v1/client_types_pb.js";
import { jsonToNativeValue, nativeValuesToJson } from "@/lib/host/values";
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

type SettingsFrame = Wire<Awaited<ReturnType<typeof rpc.settingsGetSettings>>>;
type SettingsSection = SettingsFrame["sections"][number];
type SettingsCard = SettingsFrame["cards"][number];
type SettingsCardState = SettingsCard["state"];
type HostCardTone = v1.HostCardTone;

type PluginRuntimeSettings = Wire<Awaited<ReturnType<typeof rpc.settingsGetPluginSettings>>>;
type SettingContribution = NonNullable<PluginRuntimeSettings["settings"]>["settings"][number];
type SettingValues = PluginSettingsView["values"];

/* === Settings frame === */

export async function fetchNativeSettingsFrame(session: SessionIdentity): Promise<RuntimeSettingsFrame> {
  const frame = await rpc.settingsGetSettings(session, {});
  const paths = frame.bootstrapPaths;
  return {
    bootstrap_paths: paths ? { application: paths.application, storage: paths.storage, plugins: paths.plugins, logs: paths.logs } : null,
    sections: frame.sections.map(mapSettingsSection),
    cards: frame.cards.map(mapSettingsCard),
  };
}

/* === Plugin Settings === */

export type NativePluginRuntimeSettings = PluginRuntimeSettings;

export function mapNativePluginRuntimeSettings(settings: NativePluginRuntimeSettings): PluginSettingsView {
  return {
    settings: {
      contributions: settings.settings?.settings.map(mapSettingContribution) ?? [],
    },
    values: nativeValuesToJson(settings.values?.values) as SettingValues,
    discovered: nativeValuesToJson(settings.discovered?.values) as SettingValues,
    secrets: settings.secrets.map((secret) => ({ identifier: secret.identifier, available: secret.available })),
    restart: undefined,
    unapplied: undefined,
  };
}

export async function getNativePluginSettings(session: SessionIdentity, pluginId: string) {
  return mapNativePluginRuntimeSettings(await rpc.settingsGetPluginSettings(session, { pluginId }));
}

export async function updateNativePluginSettings(session: SessionIdentity, pluginId: string, values: SettingValues) {
  return mapNativePluginRuntimeSettings(
    await rpc.settingsUpdatePluginSettings(session, {
      pluginId,
      values: { values: Object.fromEntries(Object.entries(values).map(([key, value]) => [key, jsonToNativeValue(value)])) },
    }),
  );
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
  const metadata = contribution.metadata;
  if (!metadata?.identifier) throw new Error("Read Plugin Settings: Missing Setting Identity");
  const common = { id: metadata.identifier, title: metadata.label, description: metadata.description ?? null };
  const variant = contribution.kind;
  switch (variant.case) {
    case "boolean":
      return { ...common, kind: "boolean", default: variant.value.defaultValue };
    case "text":
      return { ...common, kind: "text_area", default: variant.value.defaultValue, required: variant.value.required };
    case "path":
      return { ...common, kind: "path", default: variant.value.defaultValue, required: variant.value.required };
    case "number":
      return {
        ...common,
        kind: "number",
        default: variant.value.defaultValue,
        min: variant.value.min ?? null,
        max: variant.value.max ?? null,
        step: variant.value.step ?? null,
      };
    case "choice":
      return {
        ...common,
        kind: "multiple_choice",
        default: variant.value.defaultValue,
        choices: variant.value.choices.map((choice) => ({ value: choice.value, title: choice.label })),
        provider: variant.value.provider ?? null,
      };
    case "secret":
      return { ...common, kind: "secret", required: variant.value.required };
    default:
      throw new Error("Read Plugin Settings: Missing Setting Kind");
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
    case v1.HostCardTone.NEUTRAL:
      return "neutral";
    case v1.HostCardTone.INFO:
      return "info";
    case v1.HostCardTone.SUCCESS:
      return "success";
    case v1.HostCardTone.WARNING:
      return "warning";
    case v1.HostCardTone.DANGER:
      return "danger";
    default:
      return "neutral";
  }
}

function mapSettingsCardState(state: SettingsCardState): RuntimeSettingsCard["state"] {
  switch (state) {
    case v1.SettingsCardState.READY:
      return "ready";
    case v1.SettingsCardState.DISABLED:
      return "disabled";
    case v1.SettingsCardState.DISCONNECTED:
      return "disconnected";
    case v1.SettingsCardState.STALE:
      return "stale";
    case v1.SettingsCardState.ERROR:
      return "error";
    case v1.SettingsCardState.PERMISSION_DENIED:
      return "permission_denied";
    default:
      throw new Error("Native settings returned an unknown card state.");
  }
}
