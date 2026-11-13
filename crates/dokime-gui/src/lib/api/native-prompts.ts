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

/* === Local Imports === */

import type { Wire } from "@/lib/rpc/canonical";
import * as rpc from "@/lib/host/client";
import { PromptKind } from "@host/dokime/v1/client_types_pb.js";
import { PromptFieldKind, PromptActionEmphasis, StreamMatchOperator, type EvidenceCondition } from "@host/dokime/v1/host_pb.js";
import { Severity } from "@host/dokime/v1/execution_pb.js";
import { recordScopeFromNative, jsonToNativeValue, nativeValuesToJson, nativeValueToJson, valueTypeName } from "@/lib/host/values";
import type { SessionIdentity } from "@/lib/rpc/transport";
import type { CommandReceipt, PromptRequest } from "@/lib/types";
import { numericValue } from "@/lib/utils";
import { mapBounded, NativeApiLimitError, nowIso, timestampIso } from "./native-values";

/* === Types === */

type PromptView = Wire<Awaited<ReturnType<typeof rpc.promptsGetPrompt>>>;
type PromptInstance = NonNullable<PromptView["prompt"]>;
type NativePromptResponse = NonNullable<PromptView["response"]>;
type PromptDefinition = NonNullable<PromptInstance["definition"]>;
/* === Public API === */

export async function fetchNativePendingPrompts(
  session: SessionIdentity,
  options: { cycleId?: string | null; executionId?: string | null } = {},
): Promise<PromptRequest[]> {
  const reply = await rpc.promptsListPending(session, {
    ...(options.cycleId ? { cycleId: options.cycleId } : {}),
    ...(options.executionId ? { executionId: options.executionId } : {}),
  });
  const identities = reply.prompts;
  const views = await mapBounded(identities, (identity) => rpc.promptsGetPrompt(session, { promptId: identity.promptId }));
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
    values: { values: Object.fromEntries(Object.entries(values).map(([key, value]) => [key, jsonToNativeValue(value)])) },
    accepted: !["abort", "cancel"].includes(action),
    context: { correlationId: crypto.randomUUID() },
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
  if (!createdAt) throw new Error(`Native prompt ${prompt.promptId} has no valid creation timestamp.`);
  return {
    correlation_id: prompt.promptId,
    template_id: definition.template || prompt.scope?.templateId || "",
    prompt_type: definition.metadata?.identifier ?? "",
    kind: mapPromptKind(prompt.kind),
    level:
      definition.level === Severity.ERROR
        ? "error"
        : definition.level === Severity.WARNING
          ? "warning"
          : definition.level === Severity.DEBUG
            ? "debug"
            : "info",
    title: definition.metadata?.label || definition.metadata?.identifier || prompt.promptId,
    message: definition.message ?? null,
    fields: definition.fields.map(mapPromptField),
    actions: definition.actions.length
      ? definition.actions.map((action) => ({
          id: action.identifier,
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
    kind: mapPromptFieldKind(field.kind, field.type),
    multiple: field.multiple,
    required: field.required,
    default: nativeValueToJson(field.defaultValue),
    choices: field.choices.map((choice) => String(nativeValueToJson(choice))),
    validation: field.validation
      ? {
          min_length: field.validation.minLength === undefined ? null : safeNativeNumber(field.validation.minLength),
          max_length: field.validation.maxLength === undefined ? null : safeNativeNumber(field.validation.maxLength),
          pattern: field.validation.pattern ?? null,
          minimum: nativeValueToNumber(field.validation.minimum),
          maximum: nativeValueToNumber(field.validation.maximum),
          min_items: field.validation.minItems === undefined ? null : safeNativeNumber(field.validation.minItems),
          max_items: field.validation.maxItems === undefined ? null : safeNativeNumber(field.validation.maxItems),
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
    case PromptFieldKind.INTEGER:
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
  return emphasis === PromptActionEmphasis.DANGER ? "danger" : emphasis === PromptActionEmphasis.SECONDARY ? "secondary" : "primary";
}

function mapPromptResponse(
  response: NativePromptResponse,
  scope?: Parameters<typeof recordScopeFromNative>[0],
): NonNullable<PromptRequest["response"]> {
  const respondedAt = timestampIso(response.submittedAt);
  if (!respondedAt) throw new Error(`Native prompt ${response.promptId} has no response timestamp.`);
  return {
    correlation_id: response.promptId,
    action: response.action,
    values: nativeValuesToJson(response.values?.values),
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
  const evidenceCondition = definition.condition;
  if (!evidenceCondition) return null;
  const condition = evidenceCondition.kind;
  if (!condition?.case) return null;
  if (condition.case === "record") {
    return { condition: { kind: "record", record_type: condition.value.kind, details: nativeValuesToJson(condition.value.fields) } };
  }
  if (condition.case === "stream") {
    const stream = condition.value;
    const operator = mapTelemetryOperator(stream.operator);
    const target = nativeValueToNumber(stream.target);
    if (operator && target !== null)
      return {
        condition: { kind: "telemetry", stream: stream.stream, field: stream.field, operator, target, tolerance: stream.tolerance },
      };
  }
  return { condition: { kind: "host", label: conditionLabel(evidenceCondition) } };
}

function conditionLabel(condition: Wire<EvidenceCondition>, depth = 0): string {
  if (depth > 64) throw new Error("Read Prompt Condition: Nesting Exceeds the View Budget");
  const kind = condition.kind;
  switch (kind.case) {
    case "record":
      return `Record ${kind.value.kind}`;
    case "stream":
      return kind.value.operator === StreamMatchOperator.EXISTS
        ? `${kind.value.stream}.${kind.value.field} Exists`
        : `${kind.value.stream}.${kind.value.field} ${StreamMatchOperator[kind.value.operator]} ${String(nativeValueToJson(kind.value.target))}`;
    case "all":
      return `All of (${kind.value.conditions.map((item) => conditionLabel(item, depth + 1)).join("; ")})`;
    case "any":
      return `Any of (${kind.value.conditions.map((item) => conditionLabel(item, depth + 1)).join("; ")})`;
    default:
      throw new Error("Read Prompt Condition: Missing Kind");
  }
}

function mapTelemetryOperator(operator: StreamMatchOperator): "within" | "eq" | "gte" | "lte" | "gt" | "lt" | null {
  switch (operator) {
    case StreamMatchOperator.WITHIN:
      return "within";
    case StreamMatchOperator.EQ:
      return "eq";
    case StreamMatchOperator.GE:
      return "gte";
    case StreamMatchOperator.LE:
      return "lte";
    case StreamMatchOperator.GT:
      return "gt";
    case StreamMatchOperator.LT:
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
