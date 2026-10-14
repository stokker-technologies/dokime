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

import type { Value as SerializedValue } from "@/lib/document-contracts";
import { isDatetimeValue } from "@/lib/record-display";
import type { Wire } from "@/lib/rpc/canonical";
import type { ElapsedUnit, JsonObject, RecordScope, Severity, ValueFormat } from "@/lib/types";
import type {
  Row107 as NativeElapsed,
  Row19 as NativeFormat,
  Row4 as NativeSeverity,
  Row16 as NativeValueType,
} from "../../../.generated/dokime/v1/CatalogMessages/GetRecordSchema/messages_pb";
import type {
  Row6 as NativeMetadata,
  Row11 as NativeTimestamp,
  Row19 as NativeValue,
} from "../../../.generated/dokime/v1/CatalogMessages/GetSuite/messages_pb";
import type { Row4 as NativeScope } from "../../../.generated/dokime/v1/ExecutionMessages/GetExecution/messages_pb";

type Value = Wire<NativeValue>;
type Metadata = Wire<NativeMetadata>;
type Timestamp = Wire<NativeTimestamp>;
type EvidenceScope = Wire<NativeScope>;
type ValueType = Wire<NativeValueType>;
type NativeValueFormat = Wire<NativeFormat>;
type NativeElapsedUnit = Wire<NativeElapsed>;

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
  const { nodes } = value;
  if (!nodes.length) throw new Error("Native value has no root node.");
  const members = new Set<number>();
  function rebuild(index: number, depth: number): unknown {
    if (depth > 64) throw new Error("Native value exceeds 64 levels.");
    const kind = nodes[index]?.kind?.HarmosVariant;
    if (!kind?.case) throw new Error(`Native value node ${index} has no kind.`);
    function child(member: number): unknown {
      if (!Number.isInteger(member) || member <= index || member >= nodes.length)
        throw new Error("Native value member index is out of order or range.");
      if (members.has(member)) throw new Error("Native value node has multiple owners.");
      members.add(member);
      return rebuild(member, depth + 1);
    }
    switch (kind.case) {
      case "Null":
        return null;
      case "Bool":
      case "Text":
        return kind.value.field0;
      case "Number":
        return new NativeNumberValue(kind.value.field0);
      case "Integer": {
        const integer = kind.value.field0;
        const number = Number(integer);
        return Number.isSafeInteger(number) ? number : new NativeIntegerValue(integer);
      }
      case "Unsigned":
        return new NativeIntegerValue(kind.value.field0, true);
      case "Datetime": {
        const timestamp = timestampIso(kind.value.field0);
        if (!timestamp) throw new Error("Native value timestamp is invalid.");
        return { $datetime: timestamp };
      }
      case "Array": {
        if (!kind.value.field0) throw new Error("Native value array has no members.");
        return kind.value.field0.items.map(child);
      }
      case "Object": {
        if (!kind.value.field0) throw new Error("Native value object has no fields.");
        const entries = new Map<string, unknown>();
        for (const field of kind.value.field0.fields) {
          if (entries.has(field.key)) throw new Error("Native value object repeats a key.");
          entries.set(field.key, child(field.value));
        }
        return nativeObjectValue(Object.fromEntries(entries));
      }
    }
  }
  const result = rebuild(0, 0);
  if (members.size !== value.nodes.length - 1) throw new Error("Native value has unowned nodes.");
  return result;
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

export function nativeValueObjectToJson(value?: Value): JsonObject {
  if (!value) return {};
  if (value.nodes[0]?.kind?.HarmosVariant.case !== "Object")
    throw new Error("Native object value has no object root.");
  const converted = nativeValueToJson(value);
  if (isNativeObjectJson(converted)) return converted.fields;
  if (typeof converted !== "object" || converted === null || Array.isArray(converted))
    throw new Error("Native object value is invalid.");
  return Object.fromEntries(Object.entries(converted));
}

/** Flatten a JSON-shaped value into the canonical native node graph. */
export function jsonToNativeValue(value: unknown): Value {
  const nodes: Value["nodes"] = [];
  const containers = new Set<object>();
  function push(value: unknown, depth: number): number {
    if (depth > 64) throw new Error("Native value exceeds 64 levels.");
    const index = nodes.length;
    nodes.push({});
    let variant: NonNullable<Value["nodes"][number]["kind"]>["HarmosVariant"];
    if (value === null) variant = { case: "Null", value: {} };
    else if (typeof value === "boolean") variant = { case: "Bool", value: { field0: value } };
    else if (typeof value === "string") variant = { case: "Text", value: { field0: value } };
    else if (value instanceof NativeIntegerValue || isNativeIntegerJson(value)) {
      const integer = nativeIntegerFromJson(value);
      variant = {
        case: integer.unsigned ? "Unsigned" : "Integer",
        value: { field0: BigInt(integer.decimal) },
      };
    } else if (isNativeNumber(value)) {
      variant = {
        case: "Number",
        value: { field0: NativeNumberValue.prototype.valueOf.call(value) },
      };
    } else if (typeof value === "bigint") {
      const unsigned = value >= 1n << 63n;
      assertIntegerRange(value, unsigned);
      variant = { case: unsigned ? "Unsigned" : "Integer", value: { field0: value } };
    } else if (typeof value === "number") {
      if (Number.isInteger(value) && !Object.is(value, -0)) {
        if (!Number.isSafeInteger(value))
          throw new Error("Native integers must be safe JavaScript integers.");
        const integer = BigInt(value);
        assertIntegerRange(integer);
        variant = { case: "Integer", value: { field0: integer } };
      } else variant = { case: "Number", value: { field0: value } };
    } else if (isDatetimeValue(value) && !isNativeObjectJson(value)) {
      const match = /^(.*?)(?:\.(\d{1,9}))?Z$/.exec(value.$datetime);
      const milliseconds = Date.parse(value.$datetime);
      if (!Number.isFinite(milliseconds)) throw new Error("Invalid native datetime.");
      const seconds = BigInt(Math.floor(milliseconds / 1000));
      const nanos = match?.[2]
        ? Number(match[2].padEnd(9, "0"))
        : (((milliseconds % 1000) + 1000) % 1000) * 1000000;
      variant = { case: "Datetime", value: { field0: { seconds, nanos } } };
    } else if (typeof value === "object" && value !== null) {
      if (containers.has(value)) throw new Error("Native values cannot contain cycles.");
      containers.add(value);
      if (Array.isArray(value))
        variant = {
          case: "Array",
          value: { field0: { items: value.map((item) => push(item, depth + 1)) } },
        };
      else {
        const fields = isNativeObjectJson(value) ? value.fields : value;
        const entries = Object.entries(fields).sort(([left], [right]) =>
          left < right ? -1 : left > right ? 1 : 0,
        );
        variant = {
          case: "Object",
          value: {
            field0: {
              fields: entries.map(([key, item]) => ({ key, value: push(item, depth + 1) })),
            },
          },
        };
      }
      containers.delete(value);
    } else throw new Error("Native values cannot contain undefined or functions.");
    nodes[index] = { kind: { HarmosVariant: variant } };
    return index;
  }
  push(value, 0);
  return { nodes };
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
          metadata.attributes.map(({ key, value }) => [key, nativeValueToJson(value)]),
        ),
        created_at: timestampIso(metadata.createdAt),
        updated_at: timestampIso(metadata.updatedAt),
      }
    : {};
}

/** Preserve every ownership field at the native projection boundary. */
export function nativeEvidenceScope(scope?: RecordScope | null): EvidenceScope | undefined {
  if (!scope) return undefined;
  return {
    sessionId: scope.session_id ?? undefined,
    cycleId: scope.test_cycle_id ?? undefined,
    caseId: scope.test_case_id ?? undefined,
    executionId: scope.test_execution_id ?? undefined,
    stepId: scope.step_id ?? undefined,
    criterionId: scope.criterion_id ?? undefined,
    templateId: scope.template_id ?? undefined,
    suiteId: scope.suite_id ?? undefined,
  };
}

export function recordScopeFromNative(
  scope?: Pick<
    EvidenceScope,
    | "sessionId"
    | "cycleId"
    | "caseId"
    | "executionId"
    | "stepId"
    | "criterionId"
    | "templateId"
    | "suiteId"
  >,
): RecordScope | null {
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

/** Map a canonical enum variant into a GUI label. */
export function enumTail<Label extends string>(
  _schema: unknown,
  value: { HarmosVariant: { case: string | undefined } } | undefined,
  fallback: Label,
  overrides?: Partial<Record<string, Label>>,
): Label {
  const member = value?.HarmosVariant.case;
  if (!member || member === "Unspecified") return fallback;
  return (
    overrides?.[member] ?? (member.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase() as Label)
  );
}

export function mapNativeSeverity(value?: Wire<NativeSeverity>): Severity {
  return enumTail(undefined, value, "info", { Trace: "debug", Debug: "debug", Warn: "warning" });
}

export function valueTypeName(valueType?: ValueType): string {
  switch (valueType?.HarmosVariant.case) {
    case "Boolean":
      return "boolean";
    case "Integer":
      return "integer";
    case "Number":
      return "number";
    case "Text":
      return "string";
    case "File":
      return "path";
    case "Object":
      return "object";
    case "Array":
      return "array";
    case "Datetime":
      return "datetime";
    default:
      return "unknown";
  }
}

export function mapNativeValueFormat(format?: NativeValueFormat): ValueFormat | null {
  const kind = format?.kind?.HarmosVariant;
  if (!kind?.case || !kind.value.field0) return null;
  switch (kind.case) {
    case "Normal":
      return { kind: "normal", max_fraction_digits: kind.value.field0.maxFractionDigits ?? null };
    case "Fixed":
      return { kind: "fixed", decimals: kind.value.field0.decimals };
    case "Scientific":
      return { kind: "scientific", significant_digits: kind.value.field0.significantDigits };
    case "Engineering":
      return { kind: "engineering", significant_digits: kind.value.field0.significantDigits };
    case "SiPrefix":
      return { kind: "si_prefix", significant_digits: kind.value.field0.significantDigits };
    case "AbsoluteTime":
      return { kind: "absolute_time", pattern: kind.value.field0.pattern };
    case "ElapsedSinceRun":
      return { kind: "elapsed_since_run", unit: mapNativeElapsedUnit(kind.value.field0.unit) };
    case "ElapsedSinceCycle":
      return { kind: "elapsed_since_cycle", unit: mapNativeElapsedUnit(kind.value.field0.unit) };
  }
}

function mapNativeElapsedUnit(unit?: NativeElapsedUnit): ElapsedUnit {
  switch (unit?.HarmosVariant.case) {
    case "Milliseconds":
      return "milliseconds";
    case "Minutes":
      return "minutes";
    default:
      return "seconds";
  }
}

export function timestampIso(timestamp?: Timestamp): string | null {
  if (
    !timestamp ||
    !Number.isInteger(timestamp.nanos) ||
    timestamp.nanos < 0 ||
    timestamp.nanos >= 1_000_000_000
  )
    return null;
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
