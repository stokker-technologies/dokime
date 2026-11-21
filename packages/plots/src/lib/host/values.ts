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

import type { EvidenceScope, RecordScope } from "@host/model";
import { ValueType, type Value } from "@host/model";
import type { Metadata } from "@host/model";

/* === Local Imports === */

import { assertIntegerRange, NativeExactValue, NativeIntegerValue, NativeNumberValue, timestampIso } from "../api/native-values";
import { isDatetimeValue } from "../record-display";
import type { Wire } from "../rpc/canonical";
import type { JsonObject } from "../types";

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
  if (value instanceof NativeExactValue) {
    return { kind: { case: "exact", value: { negative: value.negative, numerator: value.numerator, denominator: value.denominator } } };
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
    case "exact":
      return new NativeExactValue(kind.value.negative, kind.value.numerator, kind.value.denominator);
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

export function nativeRecordScope(scope?: Wire<EvidenceScope> | null): Wire<RecordScope> | undefined {
  if (scope?.cycleId) return { kind: { case: "cycle", value: { cycleId: scope.cycleId, executionId: scope.executionId ?? undefined } } };
  if (scope?.executionId) return { kind: { case: "execution", value: scope.executionId } };
  return undefined;
}
