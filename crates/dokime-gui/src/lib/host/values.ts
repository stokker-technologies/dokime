/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Canonical Host Values at the View Boundary.

/* === External Imports === */

import {
  ElapsedUnit as HostElapsedUnit,
  type ValueFormat as HostValueFormat,
  type EvidenceScope,
} from "@host/dokime/v1/client_types_pb.js";
import { ValueType, type Metadata, type Value } from "@host/dokime/v1/values_pb.js";

/* === Local Imports === */

import { assertIntegerRange, NativeIntegerValue, NativeNumberValue, timestampIso } from "@/lib/api/native-values";
import { isDatetimeValue } from "@/lib/record-display";
import type { Wire } from "@/lib/rpc/canonical";
import type { JsonObject, RecordScope, ValueFormat, ElapsedUnit } from "@/lib/types";

/* === Constants === */

const VALUE_DEPTH = 64;

/* === Public Functions === */

export function nativeValueToJson(value?: Wire<Value>): unknown {
  return readValue(value, 0);
}

export function nativeValuesToJson(values: Record<string, Wire<Value>> = {}): JsonObject {
  return Object.fromEntries(Object.entries(values).map(([key, value]) => [key, nativeValueToJson(value)]));
}

export function jsonToNativeValue(value: unknown): Wire<Value> {
  return writeValue(value, 0, new Set());
}

function writeValue(value: unknown, depth: number, containers: Set<object>): Wire<Value> {
  if (depth > VALUE_DEPTH) throw new Error("Write Host Value: Nesting Exceeds the View Budget");
  if (value === null) return { kind: { case: "null", value: {} } };
  if (typeof value === "boolean") return { kind: { case: "boolean", value } };
  if (typeof value === "string") return { kind: { case: "text", value } };
  if (typeof value === "object" && value !== null && "__dokime_native_integer__" in value && value.__dokime_native_integer__ === true) {
    if (!("decimal" in value) || typeof value.decimal !== "string") throw new Error("Write Host Value: Invalid Integer Text");
    const unsigned = "unsigned" in value && value.unsigned === true;
    value = new NativeIntegerValue(BigInt(value.decimal), unsigned);
  }
  if (value instanceof NativeIntegerValue) {
    const integer = BigInt(value.decimal);
    assertIntegerRange(integer, value.unsigned);
    return { kind: value.unsigned ? { case: "unsignedInteger", value: integer } : { case: "integer", value: integer } };
  }
  if (typeof value === "object" && value !== null && "__dokime_native_number__" in value && value.__dokime_native_number__ === true) {
    const number = NativeNumberValue.prototype.valueOf.call(value);
    return { kind: { case: "number", value: number } };
  }
  if (typeof value === "bigint") {
    const unsigned = value >= 1n << 63n;
    assertIntegerRange(value, unsigned);
    return { kind: unsigned ? { case: "unsignedInteger", value } : { case: "integer", value } };
  }
  if (typeof value === "number") {
    if (!Number.isInteger(value) || Object.is(value, -0)) return { kind: { case: "number", value } };
    if (!Number.isSafeInteger(value)) throw new Error("Write Host Value: Supply Exact Integer Text for Large Integers");
    const integer = BigInt(value);
    assertIntegerRange(integer);
    return { kind: { case: "integer", value: integer } };
  }
  if (value instanceof Uint8Array) return { kind: { case: "data", value } };
  if (isDatetimeValue(value)) {
    const milliseconds = Date.parse(value.$datetime);
    if (!Number.isFinite(milliseconds)) throw new Error("Write Host Value: Invalid Timestamp");
    const match = /\.(\d{1,9})Z$/.exec(value.$datetime);
    const nanos = match ? Number(match[1].padEnd(9, "0")) : (((milliseconds % 1000) + 1000) % 1000) * 1000000;
    return { kind: { case: "datetime", value: { seconds: BigInt(Math.floor(milliseconds / 1000)), nanos } } };
  }
  if (Array.isArray(value)) {
    if (containers.has(value)) throw new Error("Write Host Value: Lists Cannot Contain Cycles");
    containers.add(value);
    const values = value.map((item) => writeValue(item, depth + 1, containers));
    containers.delete(value);
    return { kind: { case: "list", value: { values } } };
  }
  throw new Error("Write Host Value: Use Scalars or Lists; Named Values Belong to Their Owning Map");
}

function readValue(value: Wire<Value> | undefined, depth: number): unknown {
  if (!value) return undefined;
  if (depth > VALUE_DEPTH) throw new Error("Read Host Value: Nesting Exceeds the View Budget");
  const kind = value.kind;
  switch (kind.case) {
    case "null":
      return null;
    case "boolean":
    case "text":
      return kind.value;
    case "integer": {
      const number = Number(kind.value);
      return Number.isSafeInteger(number) ? number : new NativeIntegerValue(kind.value);
    }
    case "unsignedInteger":
      return new NativeIntegerValue(kind.value, true);
    case "number":
      return new NativeNumberValue(kind.value);
    case "data":
      return kind.value;
    case "list":
      return kind.value.values.map((item) => readValue(item, depth + 1));
    case "datetime": {
      const timestamp = timestampIso(kind.value);
      if (!timestamp) throw new Error("Read Host Value: Invalid Timestamp");
      return { $datetime: timestamp };
    }
    default:
      throw new Error("Read Host Value: Missing Kind");
  }
}

export function metadataJson(metadata?: Wire<Metadata>): JsonObject {
  if (!metadata) return {};
  return {
    identifier: metadata.identifier,
    label: metadata.label,
    description: metadata.description,
    tags: metadata.tags,
    attributes: Object.fromEntries(Object.entries(metadata.attributes).map(([key, value]) => [key, nativeValueToJson(value)])),
    created_at: timestampIso(metadata.createdAt),
    updated_at: timestampIso(metadata.updatedAt),
  };
}

export function valueTypeName(type: ValueType): string {
  switch (type) {
    case ValueType.BOOLEAN:
      return "boolean";
    case ValueType.INTEGER:
      return "integer";
    case ValueType.UNSIGNED_INTEGER:
      return "unsigned";
    case ValueType.NUMBER:
      return "number";
    case ValueType.TEXT:
      return "string";
    case ValueType.BYTES:
      return "bytes";
    case ValueType.LIST:
      return "array";
    case ValueType.DATETIME:
      return "datetime";
    default:
      return "unknown";
  }
}

export function nativeEvidenceScope(scope?: RecordScope | null): Wire<EvidenceScope> | undefined {
  if (!scope) return undefined;
  const origin = scope.origin;
  return {
    sessionId: scope.session_id ?? undefined,
    cycleId: scope.test_cycle_id ?? undefined,
    caseId: scope.test_case_id ?? undefined,
    executionId: scope.test_execution_id ?? undefined,
    stepId: scope.step_id ?? undefined,
    criterionId: scope.criterion_id ?? undefined,
    templateId: scope.template_id ?? undefined,
    suiteId: scope.suite_id ?? undefined,
    origin: origin
      ? {
          kind: {
            kind: origin.kind === "step_attempt" ? { case: "stepAttemptId", value: origin.attemptId } : { case: origin.kind, value: true },
          },
        }
      : undefined,
  };
}

export function recordScopeFromNative(scope?: Wire<EvidenceScope>): RecordScope | null {
  if (!scope) return null;
  const origin = scope.origin?.kind?.kind;
  return {
    session_id: scope.sessionId ?? null,
    test_cycle_id: scope.cycleId ?? null,
    test_case_id: scope.caseId ?? null,
    test_execution_id: scope.executionId ?? null,
    step_id: scope.stepId ?? null,
    criterion_id: scope.criterionId ?? null,
    template_id: scope.templateId ?? null,
    suite_id: scope.suiteId ?? null,
    ...(origin?.case
      ? { origin: origin.case === "stepAttemptId" ? { kind: "step_attempt", attemptId: origin.value } : { kind: origin.case } }
      : {}),
  };
}

export function mapNativeValueFormat(format?: Wire<HostValueFormat>): ValueFormat | null {
  const kind = format?.kind?.kind;
  switch (kind?.case) {
    case "normal":
      return { kind: "normal", max_fraction_digits: kind.value.maxFractionDigits ?? null };
    case "fixed":
      return { kind: "fixed", decimals: kind.value.decimals };
    case "scientific":
      return { kind: "scientific", significant_digits: kind.value.significantDigits };
    case "engineering":
      return { kind: "engineering", significant_digits: kind.value.significantDigits };
    case "siPrefix":
      return { kind: "si_prefix", significant_digits: kind.value.significantDigits };
    case "absoluteTime":
      return { kind: "absolute_time", pattern: kind.value.pattern };
    case "elapsedSinceRun":
      return { kind: "elapsed_since_run", unit: elapsedUnit(kind.value.unit) };
    case "elapsedSinceCycle":
      return { kind: "elapsed_since_cycle", unit: elapsedUnit(kind.value.unit) };
    default:
      return null;
  }
}

function elapsedUnit(unit: HostElapsedUnit): ElapsedUnit {
  return unit === HostElapsedUnit.MILLISECONDS ? "milliseconds" : unit === HostElapsedUnit.MINUTES ? "minutes" : "seconds";
}
