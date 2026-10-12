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

import * as rpc from "@/lib/rpc/generated/client";
import {
  CredentialBackendKind,
  HostCardTone,
  type SecretReference as NativeSecretReference,
  type SettingContribution,
  type SettingsCard,
  SettingsCardState,
  type SettingsSection,
} from "@/lib/rpc/generated/dokime/v1/settings_pb";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type {
  HostCardBlock,
  RuntimePluginSettings as LegacyPluginRuntimeSettings,
  RuntimeSettingsCard,
  RuntimeSettingsFrame,
  RuntimeSettingsSection,
  SettingsEntry,
} from "@/lib/types/view-contracts";

type SettingValues = LegacyPluginRuntimeSettings["values"];

import { jsonToNativeValue, nativeValueObjectToJson, safeNumber } from "./native-values";

/* === Settings frame === */

export async function fetchNativeSettingsFrame(
  session: SessionIdentity,
): Promise<RuntimeSettingsFrame> {
  const frame = await rpc.settingsGetSettings(session, {});
  return {
    sections: frame.sections.map(mapSettingsSection),
    cards: frame.cards.map(mapSettingsCard),
  };
}

/* === Plugin Settings === */

export type NativePluginRuntimeSettings = Awaited<ReturnType<typeof rpc.settingsGetPluginSettings>>;

export function mapNativePluginRuntimeSettings(
  settings: NativePluginRuntimeSettings,
): LegacyPluginRuntimeSettings {
  return {
    settings: {
      contributions: settings.settings?.contributions.map(mapSettingContribution) ?? [],
    },
    values: nativeValueObjectToJson(settings.values) as SettingValues,
    discovered: nativeValueObjectToJson(settings.discovered) as SettingValues,
    credential_bindings: {
      bindings: Object.fromEntries(
        Object.entries(settings.credentialBindings?.bindings ?? {}).map(([id, binding]) => [
          id,
          {
            credential_id: binding.credentialId,
            backend: mapCredentialBackend(binding.backend),
            configured: binding.configured,
          },
        ]),
      ),
      missing_required: settings.credentialBindings?.missingRequired ?? [],
    },
  };
}

export async function getNativePluginSettings(session: SessionIdentity, pluginId: string) {
  return mapNativePluginRuntimeSettings(await rpc.settingsGetPluginSettings(session, { pluginId }));
}

export async function updateNativePluginSettings(
  session: SessionIdentity,
  pluginId: string,
  values: SettingValues,
) {
  return mapNativePluginRuntimeSettings(
    await rpc.settingsUpdatePluginSettings(session, {
      pluginId,
      values: {
        fields: Object.fromEntries(
          Object.entries(values).map(([id, value]) => [id, jsonToNativeValue(value)]),
        ),
      },
    }),
  );
}

export async function updateNativePluginCredential(
  session: SessionIdentity,
  pluginId: string,
  credentialId: string,
  secret: string,
) {
  return mapNativePluginRuntimeSettings(
    await rpc.settingsUpdatePluginCredential(session, { pluginId, credentialId, secret }),
  );
}

/* === Mapping === */

function mapSettingsSection(section: SettingsSection): RuntimeSettingsSection {
  const owner =
    section.owner.case === "app"
      ? { kind: "app" as const }
      : section.owner.case === "installedPluginId"
        ? { kind: "installed_plugin" as const, plugin_id: section.owner.value }
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

function mapHostCardBlock(
  block: NonNullable<SettingsCard["body"]>["blocks"][number],
): HostCardBlock {
  switch (block.kind.case) {
    case "text":
      return { type: "text", text: block.kind.value };
    case "status":
      return {
        type: "status",
        label: block.kind.value.label,
        tone: mapHostCardTone(block.kind.value.tone),
        detail: block.kind.value.detail ?? null,
      };
    case "keyValueList":
      return {
        type: "key_value_list",
        items: block.kind.value.items.map((item) => ({
          label: item.label,
          value: item.value,
          detail: item.detail ?? null,
        })),
      };
    case "metrics":
      return {
        type: "metrics",
        items: block.kind.value.items.map((item) => ({
          label: item.label,
          value: item.value,
          unit: item.unit ?? null,
          trend: item.trend ?? null,
        })),
      };
    default:
      throw new Error("Native settings returned a card block without a kind.");
  }
}

function mapSettingContribution(contribution: SettingContribution): SettingsEntry {
  switch (contribution.kind.case) {
    case "boolean":
      return {
        kind: "boolean",
        id: contribution.kind.value.id,
        title: contribution.kind.value.title,
        description: contribution.kind.value.description ?? null,
        default: contribution.kind.value.default,
      };
    case "textArea":
      return {
        kind: "text_area",
        id: contribution.kind.value.id,
        title: contribution.kind.value.title,
        description: contribution.kind.value.description ?? null,
        default: contribution.kind.value.default,
        required: contribution.kind.value.required,
      };
    case "path":
      return {
        kind: "path",
        id: contribution.kind.value.id,
        title: contribution.kind.value.title,
        description: contribution.kind.value.description ?? null,
        default: contribution.kind.value.default,
        required: contribution.kind.value.required,
      };
    case "number":
      return {
        kind: "number",
        id: contribution.kind.value.id,
        title: contribution.kind.value.title,
        description: contribution.kind.value.description ?? null,
        default: contribution.kind.value.default,
        min: contribution.kind.value.min ?? null,
        max: contribution.kind.value.max ?? null,
        step: contribution.kind.value.step ?? null,
      };
    case "multipleChoice":
      return {
        kind: "multiple_choice",
        id: contribution.kind.value.id,
        title: contribution.kind.value.title,
        description: contribution.kind.value.description ?? null,
        default: contribution.kind.value.default,
        choices: contribution.kind.value.choices.map((choice) => ({
          value: choice.value,
          title: choice.title,
        })),
        provider: contribution.kind.value.provider ?? null,
      };
    case "secretReference":
      return {
        kind: "secret_reference",
        id: contribution.kind.value.id,
        title: contribution.kind.value.title,
        description: contribution.kind.value.description ?? null,
        default: mapSecretReference(contribution.kind.value.default),
        required: contribution.kind.value.required,
      };
    default:
      throw new Error("Native settings returned a contribution without a kind.");
  }
}

function mapSecretReference(
  reference?: NativeSecretReference,
):
  | { kind: "environment"; variable: string }
  | { kind: "stored_credential"; credential_id: string }
  | null {
  if (!reference) return null;
  if (reference.kind.case === "environment")
    return { kind: "environment", variable: reference.kind.value ?? "" };
  if (reference.kind.case === "storedCredential")
    return { kind: "stored_credential", credential_id: reference.kind.value ?? "" };
  throw new Error("Native settings returned a secret reference without a kind.");
}

function mapRefreshPolicy(
  policy: NonNullable<SettingsCard["refresh"]>,
): RuntimeSettingsCard["refresh"] {
  switch (policy.policy.case) {
    case "manual":
      return { kind: "manual" };
    case "cached":
      return {
        kind: "cached",
        max_age_ms: safeNumber(policy.policy.value.maxAgeMs, "Settings cache age"),
      };
    case "onDemand":
      return { kind: "on_demand" };
    default:
      throw new Error("Native settings returned a refresh policy without a kind.");
  }
}

function mapHostCardTone(
  tone: HostCardTone,
): "neutral" | "info" | "success" | "warning" | "danger" {
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
    case SettingsCardState.READY:
      return "ready";
    case SettingsCardState.DISABLED:
      return "disabled";
    case SettingsCardState.DISCONNECTED:
      return "disconnected";
    case SettingsCardState.STALE:
      return "stale";
    case SettingsCardState.ERROR:
      return "error";
    case SettingsCardState.PERMISSION_DENIED:
      return "permission_denied";
    default:
      throw new Error("Native settings returned an unknown card state.");
  }
}

function mapCredentialBackend(
  backend: CredentialBackendKind,
): "os_keyring" | "memory" | "local_file" {
  switch (backend) {
    case CredentialBackendKind.OS_KEYRING:
      return "os_keyring";
    case CredentialBackendKind.MEMORY:
      return "memory";
    case CredentialBackendKind.LOCAL_FILE:
      return "local_file";
    default:
      throw new Error("Native settings returned an unknown credential backend.");
  }
}
