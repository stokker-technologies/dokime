/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Native prompt adapters for the original routed UI. */

/* === External Imports === */

import { create } from "@bufbuild/protobuf";

/* === Local Imports === */

import * as rpc from "@/lib/rpc/generated/client";
import type { Cursor } from "@/lib/rpc/generated/dokime/v1/common_pb";
import { MutationContextSchema } from "@/lib/rpc/generated/dokime/v1/execution_pb";
import {
  type PromptResponse as NativePromptResponse,
  type PendingPromptIdentity,
  type PromptActionEmphasis,
  PromptActionEmphasisSchema,
  type PromptDefinition,
  PromptFieldKind,
  type PromptInstance,
  PromptKind,
  TelemetryMatchOperator,
} from "@/lib/rpc/generated/dokime/v1/prompts_pb";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { CommandReceipt, PromptRequest } from "@/lib/types";
import { numericValue } from "@/lib/utils";
import {
  enumTail,
  jsonToNativeValue,
  mapBounded,
  NATIVE_MAX_PAGES,
  NATIVE_PAGE_SIZE,
  NativeApiLimitError,
  nativeValueObjectToJson,
  nativeValueToJson,
  nowIso,
  recordScopeFromNative,
  timestampIso,
  valueTypeName,
} from "./native-values";

/* === Public API === */

export async function fetchNativePendingPrompts(
  session: SessionIdentity,
  options: { cycleId?: string | null; executionId?: string | null } = {},
): Promise<PromptRequest[]> {
  const identities: PendingPromptIdentity[] = [];
  let afterPromptId = "";
  let cursor: Cursor | undefined;
  for (let page = 0; page < NATIVE_MAX_PAGES; page++) {
    const reply = await rpc.promptsListPending(session, {
      limit: NATIVE_PAGE_SIZE,
      afterPromptId,
      cursor,
      ...(options.cycleId ? { cycleId: options.cycleId } : {}),
      ...(options.executionId ? { executionId: options.executionId } : {}),
    });
    identities.push(...reply.prompts);
    const next = reply.nextAfterPromptId;
    if (!next) break;
    if (next === afterPromptId) {
      throw new NativeApiLimitError("Prompts.ListPending", `cursor did not advance at ${next}`);
    }
    afterPromptId = next;
    cursor = reply.cursor;
    if (page === NATIVE_MAX_PAGES - 1) {
      throw new NativeApiLimitError("Prompts.ListPending", `more than ${NATIVE_MAX_PAGES} pages`);
    }
  }
  const views = await mapBounded(identities, (identity) =>
    rpc.promptsGetPrompt(session, { promptId: identity.promptId }),
  );
  return views.map((view, index) => {
    if (!view.prompt) {
      throw new Error(`Native prompt ${identities[index]?.promptId ?? "unknown"} has no instance.`);
    }
    return mapNativePrompt(view.prompt, view.response);
  });
}

export async function respondNativePrompt(
  session: SessionIdentity,
  correlationId: string,
  action: string,
  values: Record<string, unknown>,
): Promise<CommandReceipt> {
  const receipt = await rpc.promptsRespond(session, {
    promptId: correlationId,
    action,
    values: {
      fields: Object.fromEntries(
        Object.entries(values).map(([name, value]) => [name, jsonToNativeValue(value)]),
      ),
    },
    accepted: !["abort", "cancel"].includes(action),
    context: create(MutationContextSchema, { correlationId: crypto.randomUUID() }),
  });
  const acceptedAt = timestampIso(receipt.acceptedAt) ?? nowIso();
  return {
    command_id: receipt.commandId,
    kind: "prompt.response",
    target_id: receipt.promptId,
    status: "applied",
    accepted_at: acceptedAt,
    updated_at: acceptedAt,
    result: {
      committed_revision: String(receipt.committedRevision),
      observed_cursor: receipt.observedCursor
        ? {
            epoch: receipt.observedCursor.epoch,
            stream: receipt.observedCursor.stream,
            position: String(receipt.observedCursor.position),
          }
        : null,
    },
  };
}

/* === Mapping === */

function mapNativePrompt(prompt: PromptInstance, response?: NativePromptResponse): PromptRequest {
  const definition = prompt.definition;
  if (!definition) throw new Error(`Native prompt ${prompt.promptId} has no definition.`);
  const createdAt = timestampIso(prompt.createdAt);
  if (!createdAt)
    throw new Error(`Native prompt ${prompt.promptId} has no valid creation timestamp.`);
  return {
    correlation_id: prompt.promptId,
    template_id: definition.templateIdentifier ?? prompt.scope?.templateId ?? "",
    prompt_type: definition.promptType,
    schema_version: definition.schemaVersion,
    kind: mapPromptKind(prompt.kind),
    title: definition.title,
    message: definition.message ?? null,
    fields: definition.fields.map(mapPromptField),
    actions: definition.actions.length
      ? definition.actions.map((action) => ({
          id: action.id,
          label: action.label,
          emphasis: mapActionEmphasis(action.emphasis),
          requires_condition: action.requiresCondition,
        }))
      : [{ id: "submit", label: "Submit", emphasis: "primary" as const }],
    context: {},
    webhook_context: {},
    verification: mapVerification(definition),
    status: response ? "answered" : "pending",
    response: response ? mapPromptResponse(response, prompt.scope) : null,
    response_source: response ? mapResponseSource(response.responder) : null,
    expires_at: null,
    answered_at: response ? timestampIso(response.submittedAt) : null,
    created_at: createdAt,
    scope: recordScopeFromNative(prompt.scope),
  };
}

function mapPromptField(field: PromptDefinition["fields"][number]) {
  const metadata = field.metadata;
  if (!metadata?.identifier) throw new Error("Native prompt field has no identifier.");
  return {
    name: metadata.identifier,
    label: metadata.label || metadata.identifier,
    kind: mapPromptFieldKind(field.kind, field.valueType),
    multiple: field.multiple,
    required: field.required,
    default: nativeValueToJson(field.defaultValue),
    choices: field.choices.map((choice) => String(choice)),
    validation: field.validation
      ? {
          min_length:
            field.validation.minLength === undefined
              ? null
              : safeNativeNumber(field.validation.minLength),
          max_length:
            field.validation.maxLength === undefined
              ? null
              : safeNativeNumber(field.validation.maxLength),
          pattern: field.validation.pattern ?? null,
          minimum: nativeValueToNumber(field.validation.minimum),
          maximum: nativeValueToNumber(field.validation.maximum),
          min_items:
            field.validation.minItems === undefined
              ? null
              : safeNativeNumber(field.validation.minItems),
          max_items:
            field.validation.maxItems === undefined
              ? null
              : safeNativeNumber(field.validation.maxItems),
        }
      : undefined,
  };
}

function mapPromptFieldKind(
  kind: PromptFieldKind,
  valueType: Parameters<typeof valueTypeName>[0],
): "text" | "float" | "int" | "choice" | "boolean" {
  switch (kind) {
    case PromptFieldKind.FLOAT:
      return "float";
    case PromptFieldKind.INT:
      return "int";
    case PromptFieldKind.CHOICE:
      return "choice";
    case PromptFieldKind.BOOLEAN:
      return "boolean";
    case PromptFieldKind.TEXT:
      return "text";
    default:
      if (valueTypeName(valueType) === "integer") return "int";
      if (valueTypeName(valueType) === "number") return "float";
      if (valueTypeName(valueType) === "boolean") return "boolean";
      return "text";
  }
}

function mapPromptKind(kind: PromptKind): PromptRequest["kind"] {
  switch (kind) {
    case PromptKind.CONFIRMATION:
      return "confirmation";
    case PromptKind.FORM:
      return "form";
    case PromptKind.CHOICE:
      return "choice";
    case PromptKind.STATUS:
      return "status";
    case PromptKind.VERIFICATION:
      return "verification";
    default:
      throw new Error("Native prompt has an unspecified kind.");
  }
}

function mapActionEmphasis(emphasis: PromptActionEmphasis): "primary" | "secondary" | "danger" {
  return enumTail(PromptActionEmphasisSchema, emphasis, "primary");
}

function mapPromptResponse(
  response: NativePromptResponse,
  scope?: Parameters<typeof recordScopeFromNative>[0],
): NonNullable<PromptRequest["response"]> {
  const respondedAt = timestampIso(response.submittedAt);
  if (!respondedAt)
    throw new Error(`Native prompt ${response.promptId} has no response timestamp.`);
  return {
    correlation_id: response.promptId,
    action: response.action,
    values: nativeValueObjectToJson(response.values),
    source: mapResponseSource(response.responder),
    responded_at: respondedAt,
    scope: recordScopeFromNative(scope),
  };
}

function mapResponseSource(value: string): "gui" | "api" | "webhook" | "verification" {
  if (value === "api" || value === "webhook" || value === "verification") return value;
  return "gui";
}

function mapVerification(definition: PromptDefinition): PromptRequest["verification"] {
  const condition = definition.condition;
  if (!condition) return null;
  if (condition.kind.case === "record") {
    return {
      condition: {
        kind: "record",
        record_type: condition.kind.value.recordType,
        details: nativeValueObjectToJson(condition.kind.value.matchFields),
      },
    };
  }
  if (condition.kind.case === "telemetry") {
    const telemetry = condition.kind.value;
    const operator = mapTelemetryOperator(telemetry.op);
    if (!operator || !telemetry.target) return null;
    const target = nativeValueToNumber(telemetry.target);
    if (target === null) return null;
    return {
      condition: {
        kind: "telemetry",
        stream: telemetry.stream,
        field: telemetry.field,
        operator,
        target,
        tolerance: telemetry.tolerance ?? null,
      },
    };
  }
  return null;
}

function mapTelemetryOperator(
  operator: TelemetryMatchOperator,
): "within" | "eq" | "gte" | "lte" | "gt" | "lt" | null {
  switch (operator) {
    case TelemetryMatchOperator.WITHIN:
      return "within";
    case TelemetryMatchOperator.EQ:
      return "eq";
    case TelemetryMatchOperator.GTE:
      return "gte";
    case TelemetryMatchOperator.LTE:
      return "lte";
    case TelemetryMatchOperator.GT:
      return "gt";
    case TelemetryMatchOperator.LT:
      return "lt";
    default:
      return null;
  }
}

function nativeValueToNumber(value?: Parameters<typeof nativeValueToJson>[0]): number | null {
  const converted = nativeValueToJson(value);
  return numericValue(converted);
}

function safeNativeNumber(value: bigint): number {
  const converted = Number(value);
  if (!Number.isSafeInteger(converted) || converted < 0) {
    throw new NativeApiLimitError("Prompts.GetPrompt", String(value));
  }
  return converted;
}
