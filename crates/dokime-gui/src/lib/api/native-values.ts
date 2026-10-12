/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* Shared native value conversion, read limits, and mapping helpers. */

/* === External Imports === */

import { create, type DescEnumValue } from "@bufbuild/protobuf";
import type { GenEnum } from "@bufbuild/protobuf/codegenv2";
import type { Timestamp } from "@bufbuild/protobuf/wkt";

/* === Local Imports === */

import { isDatetimeValue } from "@/lib/record-display";
import type { Value as SerializedValue } from "@/lib/document-contracts";
import {
  type EvidenceScope,
  EvidenceScopeSchema,
  type Metadata,
  Severity as NativeSeverity,
  SeveritySchema,
} from "@/lib/rpc/generated/dokime/v1/common_pb";
import {
  ElapsedUnit as NativeElapsedUnit,
  type ValueFormat as NativeValueFormat,
  ValueType,
} from "@/lib/rpc/generated/dokime/v1/schema_pb";
import {
  NullValue,
  type Value,
  type ValueObject,
  ValueSchema,
} from "@/lib/rpc/generated/dokime/v1/value_pb";
import type { ElapsedUnit, JsonObject, RecordScope, Severity, ValueFormat } from "@/lib/types";

/* === Constants === */

/** The native planning contract rejects selections above this bound. */
export const NATIVE_CASE_LIMIT = 1_000;
export const NATIVE_PAGE_SIZE = 100;
export const NATIVE_MAX_PAGES = 100;
/** Keep aggregate reads below the bridge's in-flight call limit. */
export const NATIVE_MAX_CONCURRENT_READS = 4;
/** Bound lifecycle polling after an asynchronous stop request. */
export const NATIVE_MAX_SETTLE_ATTEMPTS = 120;

/* === Errors === */

export class NativeApiUnsupportedError extends Error {
  readonly operation: string;
  readonly requiredRpc: string;

  constructor(operation: string, requiredRpc: string) {
    super(`${operation} is not available on the native backend yet (${requiredRpc}).`);
    this.name = "NativeApiUnsupportedError";
    this.operation = operation;
    this.requiredRpc = requiredRpc;
  }
}

export class NativeApiLimitError extends Error {
  readonly operation: string;

  constructor(operation: string, detail: string) {
    super(`${operation} exceeded the native bounded read contract: ${detail}`);
    this.name = "NativeApiLimitError";
    this.operation = operation;
  }
}

/* === Value conversion === */

/**
 * A JSON-facing integer that keeps its signed 64-bit identity for a later
 * read-modify-write while still rendering as its decimal text.
 *
 * The tagged JSON form is intentional: drafts and selection keys can cross a
 * JSON.stringify/JSON.parse boundary without turning the integer into text.
 */
export class NativeIntegerValue {
  readonly __dokime_native_integer__ = true;
  readonly decimal: string;

  constructor(
    value: bigint,
    readonly unsigned = false,
  ) {
    this.decimal = value.toString();
  }

  toString(): string {
    return this.decimal;
  }

  toJSON(): NativeIntegerJson {
    return {
      __dokime_native_integer__: true,
      decimal: this.decimal,
      unsigned: this.unsigned,
    };
  }
}

export type NativeIntegerJson = {
  readonly __dokime_native_integer__: true;
  readonly decimal: string;
  readonly unsigned?: boolean;
};

class NativeObjectValue {
  readonly __dokime_native_object__ = true;

  constructor(readonly fields: JsonObject) {}
}

function isNativeObjectJson(value: unknown): value is NativeObjectValue {
  return (
    typeof value === "object" &&
    value !== null &&
    (value as NativeObjectValue).__dokime_native_object__ === true &&
    typeof (value as NativeObjectValue).fields === "object" &&
    (value as NativeObjectValue).fields !== null
  );
}

export class NativeNumberValue {
  readonly __dokime_native_number__ = true;
  readonly decimal: string;

  constructor(value: number) {
    this.decimal = Number.isNaN(value)
      ? "nan"
      : value === Infinity
        ? "inf"
        : value === -Infinity
          ? "-inf"
          : Object.is(value, -0)
            ? "-0"
            : String(value);
  }

  toString(): string {
    return this.decimal;
  }
  valueOf(): number {
    return this.decimal === "nan"
      ? NaN
      : this.decimal === "inf"
        ? Infinity
        : this.decimal === "-inf"
          ? -Infinity
          : Number(this.decimal);
  }
}

function isNativeNumber(value: unknown): value is NativeNumberValue {
  return (
    typeof value === "object" &&
    value !== null &&
    (value as NativeNumberValue).__dokime_native_number__ === true &&
    typeof (value as NativeNumberValue).decimal === "string"
  );
}

function isNativeIntegerJson(value: unknown): value is NativeIntegerJson {
  return (
    typeof value === "object" &&
    value !== null &&
    !Array.isArray(value) &&
    (value as Record<string, unknown>).__dokime_native_integer__ === true &&
    typeof (value as Record<string, unknown>).decimal === "string"
  );
}

function nativeIntegerFromJson(value: NativeIntegerJson | NativeIntegerValue): NativeIntegerValue {
  const integer = BigInt(value.decimal);
  assertIntegerRange(integer, value.unsigned);
  return new NativeIntegerValue(integer, value.unsigned);
}

/** Rehydrate tagged native integers after a persisted JSON round-trip. */
export function reviveNativeValues(value: unknown): unknown {
  if (isNativeObjectJson(value)) {
    return new NativeObjectValue(
      Object.fromEntries(
        Object.entries(value.fields).map(([key, item]) => [key, reviveNativeValues(item)]),
      ),
    );
  }
  if (value instanceof NativeIntegerValue) {
    return value;
  }
  if (isNativeIntegerJson(value)) {
    return nativeIntegerFromJson(value);
  }
  if (isNativeNumber(value)) {
    return new NativeNumberValue(NativeNumberValue.prototype.valueOf.call(value));
  }
  if (Array.isArray(value)) {
    return value.map(reviveNativeValues);
  }
  if (typeof value === "object" && value !== null) {
    return Object.fromEntries(
      Object.entries(value).map(([name, item]) => [name, reviveNativeValues(item)]),
    );
  }
  return value;
}

/** Convert a protobuf value to the JSON-shaped values used by the old UI. */
export function nativeValueToJson(value?: Value): unknown {
  if (!value) return undefined;
  switch (value.kind.case) {
    case "nullValue":
      return null;
    case "boolValue":
    case "textValue":
      return value.kind.value;
    case "numberValue":
      return new NativeNumberValue(value.kind.value);
    case "integerValue": {
      const number = Number(value.kind.value);
      return Number.isSafeInteger(number) ? number : new NativeIntegerValue(value.kind.value);
    }
    case "unsignedValue":
      return new NativeIntegerValue(value.kind.value, true);
    case "arrayValue":
      return value.kind.value.values.map(nativeValueToJson);
    case "objectValue": {
      const fields = nativeValueObjectToJson(value.kind.value);
      return nativeObjectValue(fields);
    }
    case "datetimeValue":
      // Preserve the datetime kind using the UI's existing datetime marker.
      return { $datetime: value.kind.value };
    default:
      return undefined;
  }
}

/** Keep authored objects distinct from the reserved scalar projection markers. */
export function nativeObjectValue(fields: JsonObject): JsonObject | NativeObjectValue {
  return isDatetimeValue(fields) ||
    isNativeIntegerJson(fields) ||
    isNativeNumber(fields) ||
    isNativeObjectJson(fields)
    ? new NativeObjectValue(fields)
    : fields;
}

/** Decode the portable Harmos value representation used by stored declarations. */
export function serializedValueToJson(value: SerializedValue): unknown {
  return serializedValue(value, 0);
}

export function serializedValueObjectToJson(values: Record<string, SerializedValue>): JsonObject {
  return Object.fromEntries(
    Object.entries(values).map(([name, value]) => [name, serializedValue(value, 0)]),
  );
}

function serializedValue(value: SerializedValue, depth: number): unknown {
  if (depth > 128) throw new Error("Stored value nesting exceeds the portable value limit.");
  switch (value.type) {
    case "null":
      return null;
    case "bool":
    case "text":
      return value.value;
    case "integer":
    case "unsigned": {
      const text = value.value;
      if (text.length > 20 || !/^-?\d+$/.test(text)) throw new Error("Invalid stored integer.");
      const integer = BigInt(text);
      const unsigned = value.type === "unsigned";
      const minimum = unsigned ? 0n : -9223372036854775808n;
      const maximum = unsigned ? 18446744073709551615n : 9223372036854775807n;
      if (integer < minimum || integer > maximum)
        throw new Error("Stored integer exceeds its declared range.");
      const number = Number(integer);
      return !unsigned && Number.isSafeInteger(number)
        ? number
        : new NativeIntegerValue(integer, unsigned);
    }
    case "number": {
      const text = value.value;
      const special = ["nan", "inf", "-inf"].includes(text);
      if (
        text.length > 1024 ||
        (!special && !/^-?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?$/.test(text))
      ) {
        throw new Error("Invalid stored number.");
      }
      const number =
        text === "nan"
          ? NaN
          : text === "inf"
            ? Infinity
            : text === "-inf"
              ? -Infinity
              : Number(text);
      if (!special && !Number.isFinite(number))
        throw new Error("Stored finite number exceeds its declared range.");
      return new NativeNumberValue(number);
    }
    case "datetime":
      return { $datetime: value.value };
    case "array":
      return value.value.map((item) => serializedValue(item, depth + 1));
    case "object":
      return nativeObjectValue(
        Object.fromEntries(
          Object.entries(value.value).map(([name, item]) => [
            name,
            serializedValue(item, depth + 1),
          ]),
        ),
      );
    default:
      throw new Error("Unknown portable stored value kind.");
  }
}

export function nativeValueObjectToJson(value?: ValueObject): JsonObject {
  return Object.fromEntries(
    Object.entries(value?.fields ?? {}).flatMap(([name, item]) => {
      const converted = nativeValueToJson(item);
      return converted === undefined ? [] : [[name, converted]];
    }),
  );
}

/** Convert a JSON-shaped value into the lossless native Value representation. */
export function jsonToNativeValue(value: unknown): Value {
  if (value === null)
    return create(ValueSchema, { kind: { case: "nullValue", value: NullValue.NULL } });
  if (typeof value === "boolean")
    return create(ValueSchema, { kind: { case: "boolValue", value } });
  if (typeof value === "string") return create(ValueSchema, { kind: { case: "textValue", value } });
  if (value instanceof NativeIntegerValue || isNativeIntegerJson(value)) {
    const integer = nativeIntegerFromJson(value);
    return create(ValueSchema, {
      kind: {
        case: integer.unsigned ? "unsignedValue" : "integerValue",
        value: BigInt(integer.decimal),
      },
    });
  }
  if (isNativeNumber(value)) {
    return create(ValueSchema, {
      kind: { case: "numberValue", value: NativeNumberValue.prototype.valueOf.call(value) },
    });
  }
  if (typeof value === "bigint") {
    const unsigned = value >= 1n << 63n;
    assertIntegerRange(value, unsigned);
    return create(ValueSchema, {
      kind: { case: unsigned ? "unsignedValue" : "integerValue", value },
    });
  }
  if (typeof value === "number") {
    if (Number.isInteger(value)) {
      if (!Number.isSafeInteger(value))
        throw new Error("Native integer values must be safe JavaScript integers.");
      const integer = BigInt(value);
      assertIntegerRange(integer);
      return create(ValueSchema, { kind: { case: "integerValue", value: integer } });
    }
    return create(ValueSchema, { kind: { case: "numberValue", value } });
  }
  if (Array.isArray(value)) {
    return create(ValueSchema, {
      kind: {
        case: "arrayValue",
        value: { values: value.map(jsonToNativeValue) },
      },
    });
  }
  if (isNativeObjectJson(value)) {
    return create(ValueSchema, {
      kind: {
        case: "objectValue",
        value: {
          fields: Object.fromEntries(
            Object.entries(value.fields).map(([key, item]) => [key, jsonToNativeValue(item)]),
          ),
        },
      },
    });
  }
  if (isDatetimeValue(value)) {
    return create(ValueSchema, { kind: { case: "datetimeValue", value: value.$datetime } });
  }
  if (typeof value === "object" && value !== null) {
    return create(ValueSchema, {
      kind: {
        case: "objectValue",
        value: {
          fields: Object.fromEntries(
            Object.entries(value).map(([name, item]) => [name, jsonToNativeValue(item)]),
          ),
        },
      },
    });
  }
  throw new Error("Native values cannot contain undefined or functions.");
}

/* === Mapping and bounded reads === */

export function metadataJson(metadata?: Metadata): JsonObject {
  return metadata
    ? {
        identifier: metadata.identifier,
        label: metadata.label,
        ...(metadata.description !== undefined ? { description: metadata.description } : {}),
        tags: metadata.tags,
        attributes: Object.fromEntries(
          Object.entries(metadata.attributes).map(([key, value]) => [
            key,
            nativeValueToJson(value),
          ]),
        ),
        created_at: timestampIso(metadata.createdAt),
        updated_at: timestampIso(metadata.updatedAt),
      }
    : {};
}

/** Preserve every ownership field at the native projection boundary. */
export function nativeEvidenceScope(scope?: RecordScope | null): EvidenceScope | undefined {
  if (!scope) return undefined;
  return create(EvidenceScopeSchema, {
    sessionId: scope.session_id ?? undefined,
    cycleId: scope.test_cycle_id ?? undefined,
    caseId: scope.test_case_id ?? undefined,
    executionId: scope.test_execution_id ?? undefined,
    stepId: scope.step_id ?? undefined,
    criterionId: scope.criterion_id ?? undefined,
    templateId: scope.template_id ?? undefined,
    suiteId: scope.suite_id ?? undefined,
  });
}

export function recordScopeFromNative(scope?: EvidenceScope): RecordScope | null {
  if (!scope) return null;
  return {
    session_id: scope.sessionId ?? null,
    test_cycle_id: scope.cycleId ?? null,
    test_case_id: scope.caseId ?? null,
    test_execution_id: scope.executionId ?? null,
    step_id: scope.stepId ?? null,
    criterion_id: scope.criterionId ?? null,
    template_id: scope.templateId ?? null,
    suite_id: scope.suiteId ?? null,
  };
}

/* Map a generated proto enum value to the lowercase of its member name.
 *
 * The native adapters used to spell every member out by hand, so a member added
 * to the proto silently fell through to whichever label the `default` arm held.
 * `UNSPECIFIED` and any number this generation does not know still resolve to
 * `fallback`; `overrides` names the members whose label is not the lowercase
 * member name, including several members collapsing onto one label.
 *
 * The member name comes from the enum descriptor rather than `enumToJson`,
 * which returns the prefixed proto name (`SEVERITY_WARN`) and throws on a
 * number outside the descriptor.
 */
export function enumTail<Label extends string, Value extends number>(
  schema: GenEnum<Value>,
  value: Value | undefined,
  fallback: Label,
  overrides?: Partial<Record<Value, Label>>,
): Label {
  if (value === undefined) return fallback;
  const override = overrides?.[value];
  if (override !== undefined) return override;
  const member: DescEnumValue | undefined = schema.value[value];
  if (!member || member.number === 0) return fallback;
  return member.localName.toLowerCase() as Label;
}

/** Severity renames `WARN` and collapses the two verbose levels onto `debug`. */
export function mapNativeSeverity(value?: NativeSeverity): Severity {
  return enumTail(SeveritySchema, value, "info", {
    [NativeSeverity.TRACE]: "debug",
    [NativeSeverity.DEBUG]: "debug",
    [NativeSeverity.WARN]: "warning",
  });
}

export function valueTypeName(valueType: ValueType): string {
  switch (valueType) {
    case ValueType.BOOLEAN:
      return "boolean";
    case ValueType.INTEGER:
      return "integer";
    case ValueType.NUMBER:
      return "number";
    case ValueType.TEXT:
      return "string";
    case ValueType.FILE:
      return "path";
    case ValueType.OBJECT:
      return "object";
    case ValueType.ARRAY:
      return "array";
    default:
      return "unknown";
  }
}

/** Map a native `ValueFormat` oneof into the hand-written, snake_case-tagged mirror. */
export function mapNativeValueFormat(format?: NativeValueFormat): ValueFormat | null {
  if (!format) return null;
  switch (format.kind.case) {
    case "normal":
      return { kind: "normal", max_fraction_digits: format.kind.value.maxFractionDigits ?? null };
    case "fixed":
      return { kind: "fixed", decimals: format.kind.value.decimals };
    case "scientific":
      return { kind: "scientific", significant_digits: format.kind.value.significantDigits };
    case "engineering":
      return { kind: "engineering", significant_digits: format.kind.value.significantDigits };
    case "siPrefix":
      return { kind: "si_prefix", significant_digits: format.kind.value.significantDigits };
    case "absoluteTime":
      return { kind: "absolute_time", pattern: format.kind.value.pattern };
    case "elapsedSinceRun":
      return { kind: "elapsed_since_run", unit: mapNativeElapsedUnit(format.kind.value.unit) };
    case "elapsedSinceCycle":
      return { kind: "elapsed_since_cycle", unit: mapNativeElapsedUnit(format.kind.value.unit) };
    default:
      return null;
  }
}

// An unset oneof carries no unit; "seconds" is the same lenient default this
// module already uses for other cosmetic, non-structural enums (`axisScale`
// in native-plots.ts) — a stray unspecified value degrades the axis label,
// it does not take the plot down.
function mapNativeElapsedUnit(unit: NativeElapsedUnit): ElapsedUnit {
  switch (unit) {
    case NativeElapsedUnit.MILLISECONDS:
      return "milliseconds";
    case NativeElapsedUnit.MINUTES:
      return "minutes";
    default:
      return "seconds";
  }
}

export function timestampIso(timestamp?: Timestamp): string | null {
  if (!timestamp) return null;
  const milliseconds = Number(timestamp.seconds) * 1_000 + Math.floor(timestamp.nanos / 1_000_000);
  const date = new Date(milliseconds);
  if (!Number.isFinite(date.valueOf())) return null;
  const iso = date.toISOString();
  return timestamp.nanos % 1_000_000 === 0
    ? iso
    : `${iso.slice(0, iso.lastIndexOf("."))}.${String(timestamp.nanos).padStart(9, "0")}Z`;
}

export function durationSeconds(start?: Timestamp, finish?: Timestamp): number | null {
  const started = timestampIso(start);
  const ended = timestampIso(finish);
  if (!started || !ended) return null;
  return Math.max(0, (Date.parse(ended) - Date.parse(started)) / 1_000);
}

export function safeNumber(value: bigint, label: string): number {
  const number = Number(value);
  if (!Number.isSafeInteger(number) || number < 0)
    throw new NativeApiLimitError(label, String(value));
  return number;
}

export function assertIntegerRange(value: bigint, unsigned = false): void {
  if (unsigned ? value < 0n || value >= 1n << 64n : value < -(1n << 63n) || value >= 1n << 63n)
    throw new Error("Native integer values must fit their signed or unsigned 64-bit range.");
}

export function parseNativeInteger(text: string): number | NativeIntegerValue {
  if (!/^[+-]?\d+$/.test(text.trim())) throw new Error("Expected a whole number.");
  const value = BigInt(text);
  const unsigned = value >= 1n << 63n;
  assertIntegerRange(value, unsigned);
  const number = Number(value);
  return Number.isSafeInteger(number) ? number : new NativeIntegerValue(value, unsigned);
}

export function uniqueStrings(values: string[]): string[] {
  return [...new Set(values.map((value) => value.trim()).filter(Boolean))];
}

export function nowIso(): string {
  return new Date().toISOString();
}

/** Calculate the canonical lowercase digest used by native content streams. */
export async function sha256Hex(bytes: Uint8Array<ArrayBuffer>): Promise<string> {
  // `digest` honors the view's byteOffset/byteLength directly, so hashing
  // `bytes` in place is correct. A prior `bytes.slice()` here doubled peak
  // memory for every upload and download by copying content that is
  // already bounded at up to 256 MiB.
  const digest = await crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest), (value) => value.toString(16).padStart(2, "0")).join(
    "",
  );
}

/** Run independent native reads with a bridge-safe, deterministic concurrency bound. */
export async function mapBounded<T, R>(
  values: readonly T[],
  operation: (value: T, index: number) => Promise<R>,
): Promise<R[]> {
  const result = new Array<R>(values.length);
  let next = 0;
  async function worker() {
    while (true) {
      const index = next++;
      if (index >= values.length) return;
      result[index] = await operation(values[index], index);
    }
  }
  await Promise.all(
    Array.from({ length: Math.min(NATIVE_MAX_CONCURRENT_READS, values.length) }, () => worker()),
  );
  return result;
}
