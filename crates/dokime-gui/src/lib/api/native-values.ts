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
import type { JsonObject } from "@/lib/types";

type Timestamp = { seconds: bigint; nanos: number };

/* === Constants === */

const DOCUMENTS_VALUE_DEPTH = 128;
const GUI_CASE_ITEMS = 1_000;
const GUI_CONCURRENT_READS = 4;
const GUI_MAXIMUM_PAGES = 100;
const GUI_NUMBER_TEXT_CHARS = 1_024;
const GUI_PAGE_ITEMS = 500;
const GUI_SMALL_PAGE_ITEMS = 100;

/** The native planning contract rejects selections above this bound. */
export const NATIVE_CASE_LIMIT = GUI_CASE_ITEMS;
/** Local Page Budget For Cursor-Paged Lists. */
export const NATIVE_PAGE_SIZE = GUI_PAGE_ITEMS;
/** The annotation and plugin lists keep their own cap of 100 items per page. */
export const NATIVE_SMALL_PAGE_SIZE = GUI_SMALL_PAGE_ITEMS;
export const NATIVE_MAX_PAGES = GUI_MAXIMUM_PAGES;
/** Keep aggregate reads below the bridge's in-flight call limit. */
export const NATIVE_MAX_CONCURRENT_READS = GUI_CONCURRENT_READS;

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

/** Exact Arithmetic Results Never Pass Through A JavaScript Number. */
export class NativeExactValue {
  readonly __dokime_native_exact__ = true;
  readonly decimal: string;

  constructor(
    readonly negative: boolean,
    readonly numerator: string,
    readonly denominator: string,
  ) {
    let n = BigInt(numerator);
    let d = BigInt(denominator);
    if (n < 0n || d <= 0n) throw new Error("Read Exact Value: Supply Nonnegative Magnitudes and a Positive Denominator");
    let a = n;
    let b = d;
    while (b !== 0n) {
      const next = a % b;
      a = b;
      b = next;
    }
    if (a > 0n) {
      n /= a;
      d /= a;
    }
    let remainder = n % d;
    let fractional = "";
    let finite = d;
    while (finite % 2n === 0n) finite /= 2n;
    while (finite % 5n === 0n) finite /= 5n;
    if (finite === 1n) {
      while (remainder !== 0n) {
        remainder *= 10n;
        fractional += (remainder / d).toString();
        remainder %= d;
      }
      this.decimal = `${negative && n !== 0n ? "-" : ""}${n / d}${fractional ? `.${fractional}` : ""}`;
    } else {
      this.decimal = `${negative ? "-" : ""}${n}/${d}`;
    }
  }

  toString(): string {
    return this.decimal;
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
    return this.decimal === "nan" ? NaN : this.decimal === "inf" ? Infinity : this.decimal === "-inf" ? -Infinity : Number(this.decimal);
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
    return new NativeObjectValue(Object.fromEntries(Object.entries(value.fields).map(([key, item]) => [key, reviveNativeValues(item)])));
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
    return Object.fromEntries(Object.entries(value).map(([name, item]) => [name, reviveNativeValues(item)]));
  }
  return value;
}

/** Keep authored objects distinct from the reserved scalar projection markers. */
export function nativeObjectValue(fields: JsonObject): JsonObject | NativeObjectValue {
  return isDatetimeValue(fields) || isNativeIntegerJson(fields) || isNativeNumber(fields) || isNativeObjectJson(fields)
    ? new NativeObjectValue(fields)
    : fields;
}

/** Decode the portable Harmos value representation used by stored declarations. */
export function serializedValueToJson(value: SerializedValue): unknown {
  return serializedValue(value, 0);
}

export function serializedValueObjectToJson(values: Record<string, SerializedValue>): JsonObject {
  return Object.fromEntries(Object.entries(values).map(([name, value]) => [name, serializedValue(value, 0)]));
}

function serializedValue(value: SerializedValue, depth: number): unknown {
  if (depth > DOCUMENTS_VALUE_DEPTH) throw new Error("Stored value nesting exceeds the portable value limit.");
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
      if (integer < minimum || integer > maximum) throw new Error("Stored integer exceeds its declared range.");
      const number = Number(integer);
      return !unsigned && Number.isSafeInteger(number) ? number : new NativeIntegerValue(integer, unsigned);
    }
    case "number": {
      const text = value.value;
      const special = ["nan", "inf", "-inf"].includes(text);
      if (text.length > GUI_NUMBER_TEXT_CHARS || (!special && !/^-?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?$/.test(text))) {
        throw new Error("Invalid stored number.");
      }
      const number = text === "nan" ? NaN : text === "inf" ? Infinity : text === "-inf" ? -Infinity : Number(text);
      if (!special && !Number.isFinite(number)) throw new Error("Stored finite number exceeds its declared range.");
      return new NativeNumberValue(number);
    }
    case "datetime":
      return { $datetime: value.value };
    case "array":
      return value.value.map((item) => serializedValue(item, depth + 1));
    case "object":
      return nativeObjectValue(
        Object.fromEntries(Object.entries(value.value).map(([name, item]) => [name, serializedValue(item, depth + 1)])),
      );
    default:
      throw new Error("Unknown portable stored value kind.");
  }
}

/* === Mapping and Bounded Reads === */

export function timestampIso(timestamp?: Timestamp): string | null {
  if (!timestamp || !Number.isInteger(timestamp.nanos) || timestamp.nanos < 0 || timestamp.nanos >= 1_000_000_000) return null;
  const milliseconds = Number(timestamp.seconds) * 1_000 + Math.floor(timestamp.nanos / 1_000_000);
  const date = new Date(milliseconds);
  if (!Number.isFinite(date.valueOf())) return null;
  const iso = date.toISOString();
  return timestamp.nanos % 1_000_000 === 0 ? iso : `${iso.slice(0, iso.lastIndexOf("."))}.${String(timestamp.nanos).padStart(9, "0")}Z`;
}

export function durationSeconds(start?: Timestamp, finish?: Timestamp): number | null {
  const started = timestampIso(start);
  const ended = timestampIso(finish);
  if (!started || !ended) return null;
  return Math.max(0, (Date.parse(ended) - Date.parse(started)) / 1_000);
}

export function safeNumber(value: bigint, label: string): number {
  const number = Number(value);
  if (!Number.isSafeInteger(number) || number < 0) throw new NativeApiLimitError(label, String(value));
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
  return Array.from(new Uint8Array(digest), (value) => value.toString(16).padStart(2, "0")).join("");
}

/** Run independent native reads with a bridge-safe, deterministic concurrency bound. */
export async function mapBounded<T, R>(values: readonly T[], operation: (value: T, index: number) => Promise<R>): Promise<R[]> {
  const result = new Array<R>(values.length);
  let next = 0;
  async function worker() {
    while (true) {
      const index = next++;
      if (index >= values.length) return;
      result[index] = await operation(values[index], index);
    }
  }
  await Promise.all(Array.from({ length: Math.min(NATIVE_MAX_CONCURRENT_READS, values.length) }, () => worker()));
  return result;
}
