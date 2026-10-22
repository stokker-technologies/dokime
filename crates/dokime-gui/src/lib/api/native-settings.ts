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

import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/rpc/generated/client";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type {
  HostCardBlock,
  RuntimePluginSettings as LegacyPluginRuntimeSettings,
  RuntimeSettingsCard,
  RuntimeSettingsFrame,
  RuntimeSettingsSection,
  SettingsEntry,
} from "@/lib/types/view-contracts";

import { jsonToNativeValue, nativeValueObjectToJson, safeNumber } from "./native-values";

/* === Types === */

type SettingsFrame = Wire<Awaited<ReturnType<typeof rpc.settingsGetSettings>>>;
type SettingsSection = SettingsFrame["sections"][number];
type SettingsCard = SettingsFrame["cards"][number];
type SettingsCardState = SettingsCard["state"];
type HostCardTone = NonNullable<
  Extract<NonNullable<NonNullable<SettingsCard["body"]>["blocks"][number]["kind"]>["HarmosVariant"], { case: "Status" }>["value"]["field0"]
>["tone"];
type PluginRuntimeSettings = Wire<Awaited<ReturnType<typeof rpc.settingsGetPluginSettings>>>;
type SettingContribution = NonNullable<PluginRuntimeSettings["settings"]>["contributions"][number];
type NativeSecretReference = NonNullable<
  Extract<NonNullable<SettingContribution["kind"]>["HarmosVariant"], { case: "SecretReference" }>["value"]["field0"]
>["default"];
type SecretBackendKind = NonNullable<NonNullable<PluginRuntimeSettings["secretBindings"]>["bindings"][number]["value"]>["backend"];
type SettingValues = LegacyPluginRuntimeSettings["values"];

/* === Settings frame === */

export async function fetchNativeSettingsFrame(session: SessionIdentity): Promise<RuntimeSettingsFrame> {
  const frame = await rpc.settingsGetSettings(session, {});
  return {
    sections: frame.sections.map(mapSettingsSection),
    cards: frame.cards.map(mapSettingsCard),
  };
}

/* === Plugin Settings === */

export type NativePluginRuntimeSettings = PluginRuntimeSettings;

export function mapNativePluginRuntimeSettings(settings: NativePluginRuntimeSettings): LegacyPluginRuntimeSettings {
  return {
    settings: {
      contributions: settings.settings?.contributions.map(mapSettingContribution) ?? [],
    },
    values: nativeValueObjectToJson(settings.values) as SettingValues,
    discovered: nativeValueObjectToJson(settings.discovered) as SettingValues,
    secret_bindings: {
      bindings: Object.fromEntries(
        (settings.secretBindings?.bindings ?? []).map(({ key, value }) => {
          if (!value) throw new Error(`Native settings omitted secret binding ${key}.`);
          return [
            key,
            {
              secret_id: value.secretId,
              backend: mapSecretBackend(value.backend),
              configured: value.configured,
            },
          ];
        }),
      ),
      missing_required: settings.secretBindings?.missingRequired ?? [],
    },
  };
}

export async function getNativePluginSettings(session: SessionIdentity, pluginId: string) {
  return mapNativePluginRuntimeSettings(await rpc.settingsGetPluginSettings(session, { pluginId }));
}

export async function updateNativePluginSettings(session: SessionIdentity, pluginId: string, values: SettingValues) {
  return mapNativePluginRuntimeSettings(
    await rpc.settingsUpdatePluginSettings(session, {
      pluginId,
      values: jsonToNativeValue(values),
    }),
  );
}

export async function updateNativePluginSecret(session: SessionIdentity, pluginId: string, secretId: string, secret: string) {
  return mapNativePluginRuntimeSettings(await rpc.settingsUpdatePluginSecret(session, { pluginId, secretId, secret }));
}

/* === Mapping === */

function mapSettingsSection(section: SettingsSection): RuntimeSettingsSection {
  const owner =
    section.owner?.HarmosVariant.case === "App"
      ? { kind: "app" as const }
      : section.owner?.HarmosVariant.case === "InstalledPluginId"
        ? { kind: "installed_plugin" as const, plugin_id: section.owner.HarmosVariant.value.field0 }
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
  const variant = block.kind?.HarmosVariant;
  switch (variant?.case) {
    case "Text": {
      const entry = variant.value.field0;
      return { type: "text", text: entry };
    }
    case "Status": {
      const entry = variant.value.field0;
      if (!entry) throw new Error("Native settings omitted its status payload.");
      return {
        type: "status",
        label: entry.label,
        tone: mapHostCardTone(entry.tone),
        detail: entry.detail ?? null,
      };
    }
    case "KeyValueList": {
      const entry = variant.value.field0;
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
    case "Metrics": {
      const entry = variant.value.field0;
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
  const variant = contribution.kind?.HarmosVariant;
  switch (variant?.case) {
    case "Boolean": {
      const entry = variant.value.field0;
      if (!entry) throw new Error("Native settings omitted its boolean payload.");
      return {
        kind: "boolean",
        id: entry.id,
        title: entry.title,
        description: entry.description ?? null,
        default: entry.default,
      };
    }
    case "TextArea": {
      const entry = variant.value.field0;
      if (!entry) throw new Error("Native settings omitted its text area payload.");
      return {
        kind: "text_area",
        id: entry.id,
        title: entry.title,
        description: entry.description ?? null,
        default: entry.default,
        required: entry.required,
      };
    }
    case "Path": {
      const entry = variant.value.field0;
      if (!entry) throw new Error("Native settings omitted its path payload.");
      return {
        kind: "path",
        id: entry.id,
        title: entry.title,
        description: entry.description ?? null,
        default: entry.default,
        required: entry.required,
      };
    }
    case "Number": {
      const entry = variant.value.field0;
      if (!entry) throw new Error("Native settings omitted its number payload.");
      return {
        kind: "number",
        id: entry.id,
        title: entry.title,
        description: entry.description ?? null,
        default: entry.default,
        min: entry.min ?? null,
        max: entry.max ?? null,
        step: entry.step ?? null,
      };
    }
    case "MultipleChoice": {
      const entry = variant.value.field0;
      if (!entry) throw new Error("Native settings omitted its choice payload.");
      return {
        kind: "multiple_choice",
        id: entry.id,
        title: entry.title,
        description: entry.description ?? null,
        default: entry.default,
        choices: entry.choices.map((choice) => ({ value: choice.value, title: choice.title })),
        provider: entry.provider ?? null,
      };
    }
    case "SecretReference": {
      const entry = variant.value.field0;
      if (!entry) throw new Error("Native settings omitted its secret reference payload.");
      return {
        kind: "secret_reference",
        id: entry.id,
        title: entry.title,
        description: entry.description ?? null,
        default: mapSecretReference(entry.default),
        required: entry.required,
      };
    }
    default:
      throw new Error("Native settings returned a contribution without a kind.");
  }
}

function mapSecretReference(
  reference?: NativeSecretReference,
): { kind: "environment"; variable: string } | { kind: "stored_secret"; secret_id: string } | null {
  if (!reference) return null;
  if (reference.kind?.HarmosVariant.case === "Environment")
    return { kind: "environment", variable: reference.kind.HarmosVariant.value.field0 ?? "" };
  if (reference.kind?.HarmosVariant.case === "StoredSecret")
    return {
      kind: "stored_secret",
      secret_id: reference.kind.HarmosVariant.value.field0 ?? "",
    };
  throw new Error("Native settings returned a secret reference without a kind.");
}

function mapRefreshPolicy(policy: NonNullable<SettingsCard["refresh"]>): RuntimeSettingsCard["refresh"] {
  const variant = policy.policy?.HarmosVariant;
  switch (variant?.case) {
    case "Manual":
      return { kind: "manual" };
    case "Cached": {
      const cached = variant.value.field0;
      if (!cached) throw new Error("Native settings omitted its cached refresh policy.");
      return {
        kind: "cached",
        max_age_ms: safeNumber(cached.maxAgeMs, "Settings cache age"),
      };
    }
    case "OnDemand":
      return { kind: "on_demand" };
    default:
      throw new Error("Native settings returned a refresh policy without a kind.");
  }
}

function mapHostCardTone(tone: HostCardTone): "neutral" | "info" | "success" | "warning" | "danger" {
  switch (tone?.HarmosVariant.case) {
    case "Neutral":
      return "neutral";
    case "Info":
      return "info";
    case "Success":
      return "success";
    case "Warning":
      return "warning";
    case "Danger":
      return "danger";
    default:
      return "neutral";
  }
}

function mapSettingsCardState(state: SettingsCardState): RuntimeSettingsCard["state"] {
  switch (state?.HarmosVariant.case) {
    case "Ready":
      return "ready";
    case "Disabled":
      return "disabled";
    case "Disconnected":
      return "disconnected";
    case "Stale":
      return "stale";
    case "Error":
      return "error";
    case "PermissionDenied":
      return "permission_denied";
    default:
      throw new Error("Native settings returned an unknown card state.");
  }
}

function mapSecretBackend(backend: SecretBackendKind): "os_keyring" | "memory" | "local_file" {
  switch (backend?.HarmosVariant.case) {
    case "OsKeyring":
      return "os_keyring";
    case "Memory":
      return "memory";
    case "LocalFile":
      return "local_file";
    default:
      throw new Error("Native settings returned an unknown secret backend.");
  }
}
