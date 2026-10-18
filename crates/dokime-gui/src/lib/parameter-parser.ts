/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { NativeIntegerValue, NativeNumberValue, parseNativeInteger, reviveNativeValues } from "@/lib/api/native-values";
import { numericValue } from "@/lib/utils";

/* === Types === */

import type { ParameterDefinition, ParameterListDefinition } from "./types";

export type ParseResult = { ok: true; values: unknown[] } | { ok: false; error: string };

const NUMERIC_TYPES = new Set(["int", "integer", "float", "number"]);
const BOOL_TYPES = new Set(["bool", "boolean"]);

const RANGE_HARD_CAP = 10_000;

/* === Public === */

/**
 * Parse the user-typed parameter input into a list of values.
 *
 * Supported syntaxes:
 * - Empty string -> zero values (template default applies)
 * - Comma-separated literal list: `1, 2, 3` or `alpha, beta`
 * - Inclusive arithmetic range (numeric only): `start..stop` (step 1)
 *   or `start..stop:step`
 * - `linspace(start, stop, n)` - n linearly spaced points, inclusive
 * - `arange(start, stop[, step])` - half-open arithmetic range,
 *   exclusive of `stop` (matches numpy)
 *
 * Booleans accept `true`, `false`, `1`, `0`, `yes`, `no`. Strings pass
 * through unchanged after trimming.
 */
export function parseParameterInput(raw: string, definition: ParameterDefinition): ParseResult {
  const valueType = (definition.value_type ?? "str").toLowerCase();
  const trimmed = raw.trim();
  if (!trimmed) {
    return { ok: true, values: [] };
  }

  // Each candidate is a complete list. Expanded parameters carry several
  // candidates, one per chip once the chip's expression has been expanded;
  // single parameters carry exactly one list whatever their chips expand to.
  if (definition.list) {
    if ((definition.expansion?.kind ?? "single") === "single") {
      return boundedResult(parseListInput(trimmed, definition.list), definition);
    }
    let candidates: unknown;
    try {
      candidates = reviveNativeValues(JSON.parse(trimmed));
    } catch {
      return { ok: false, error: "Enter a list of candidate lists." };
    }
    if (!Array.isArray(candidates) || candidates.length === 0) {
      return { ok: false, error: "Add at least one candidate list." };
    }
    /* Bounds belong to the row, not to each candidate: `min_items = 2` asks
       the operator for two candidate lists, and checking it per candidate
       would reject every single-valued one. Element typing stays per
       candidate; the length check happens once, below. */
    const unbounded = { ...definition.list, max_items: null, min_items: null };
    const values: unknown[] = [];
    for (const [index, candidate] of candidates.entries()) {
      for (const entry of candidateLists(candidate, definition.list.element_type)) {
        const parsed = parseListInput(formatListRaw(entry), unbounded);
        if (!parsed.ok) return { ok: false, error: `List ${index + 1}: ${parsed.error}` };
        values.push(...parsed.values);
      }
    }
    const bounds = listBoundsError(values.length, definition.list);
    if (bounds) return { ok: false, error: bounds };
    return boundedResult({ ok: true, values }, definition);
  }

  if ((definition.expansion?.kind ?? "single") !== "single" && trimmed.startsWith("[")) {
    const parsed = parseListInput(trimmed, { element_type: valueType, min_items: 1 });
    return parsed.ok ? boundedResult({ ok: true, values: parsed.values[0] as unknown[] }, definition) : parsed;
  }

  if (NUMERIC_TYPES.has(valueType)) {
    const expansion = tryNumericExpansion(trimmed, valueType);
    if (expansion !== null) {
      return boundedResult(expansion, definition);
    }
  }

  // Expanded scalars accept comma-separated candidates; single text preserves commas.
  const parts = (definition.expansion?.kind ?? "single") === "single" ? [trimmed] : splitCommaPreservingDecimals(trimmed);
  const values: unknown[] = [];
  for (const part of parts) {
    const item = part.trim();
    if (!item) continue;
    const coerced = coerceSingle(item, valueType);
    if (!coerced.ok) {
      return coerced;
    }
    values.push(coerced.value);
  }
  if (values.length === 0) {
    return { ok: false, error: "No values parsed." };
  }
  return boundedResult({ ok: true, values }, definition);
}

/** Checks inclusive bounds for a scalar or every element of a numeric list. */
export function numericBoundsError(value: unknown, bounds: { min?: number | null; max?: number | null }): string | null {
  for (const original of Array.isArray(value) ? value : [value]) {
    if (original instanceof NativeIntegerValue) {
      const integer = BigInt(original.decimal);
      if (bounds.min != null && integer < BigInt(Math.ceil(bounds.min))) return `Must be at least ${bounds.min}; ${original} entered.`;
      if (bounds.max != null && integer > BigInt(Math.floor(bounds.max))) return `Must be at most ${bounds.max}; ${original} entered.`;
      continue;
    }
    const item = numericValue(original);
    if (typeof item !== "number") continue;
    if (bounds.min != null && item < bounds.min) return `Must be at least ${bounds.min}; ${item} entered.`;
    if (bounds.max != null && item > bounds.max) return `Must be at most ${bounds.max}; ${item} entered.`;
  }
  return null;
}

function boundedResult(result: ParseResult, definition: ParameterDefinition): ParseResult {
  if (!result.ok) return result;
  if ((definition.expansion?.kind ?? "single") === "single" && result.values.length > 1) {
    return { ok: false, error: "This parameter accepts one value." };
  }
  for (const value of result.values) {
    const error = numericBoundsError(value, definition);
    if (error) return { ok: false, error };
  }
  return result;
}

/**
 * Serialize a list editor's elements into the row's raw text.
 *
 * The raw form is JSON, not a comma-separated string: an element's own commas,
 * spaces, and quoting have to survive a round trip through the editor.
 */
export function formatListRaw(items: unknown[]): string {
  return JSON.stringify(items);
}

/**
 * Read a list row's raw text back into its elements, tolerating the text an
 * operator is mid-way through editing.
 */
export function parseListRaw(raw: string): unknown[] {
  const trimmed = raw.trim();
  if (!trimmed) {
    return [];
  }
  try {
    const parsed: unknown = reviveNativeValues(JSON.parse(trimmed));
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

/**
 * The values a chip's expression stands for, or `null` when the chip is not an
 * expression at all.
 *
 * An operator types `1..5` into a chip because they mean the five values it
 * generates, so the editor replaces that chip with five and the parser makes
 * five candidates of them. A comma list is deliberately not an expression
 * here: a chip holding `110, 230` is one list of two values, which is how the
 * registered default of a list parameter arrives and what it has to stay.
 * Anything unresolved, including a half-typed `1..`, keeps its chip so the
 * parser is the one that says what is wrong with it.
 */
export function expandChipValue(item: unknown, elementType: string): unknown[] | null {
  if (typeof item !== "string") {
    return null;
  }
  const normalized = elementType.toLowerCase();
  if (!NUMERIC_TYPES.has(normalized)) {
    return null;
  }
  const expansion = tryNumericExpansion(item.trim(), normalized);
  return expansion?.ok ? expansion.values : null;
}

/**
 * The bounds message for a list of `count` elements, or `null` when the length
 * is allowed. Shared by the editor and the parser so one wording reaches the
 * operator.
 */
export function listBoundsError(count: number, list: ParameterListDefinition): string | null {
  const min = list.min_items ?? null;
  const max = list.max_items ?? null;
  if (min !== null && count < min) {
    return `At least ${min} ${min === 1 ? "item" : "items"} required; ${count} entered.`;
  }
  if (max !== null && count > max) {
    return `At most ${max} ${max === 1 ? "item" : "items"} allowed; ${count} entered.`;
  }
  return null;
}

/**
 * One chip of a list row: a single value, and the candidate list it belongs
 * to.
 *
 * A chip never holds a comma. A registered default arrives as the author wrote
 * it — `[[100, 50, 10]]` is one ladder, one case — and rendering that as a
 * single chip reading "100,50,10" hid both the values and the fact that they
 * travel together. The values become chips; `candidate` is what keeps them one
 * case, so the preview count does not change when the display does.
 */
export type ListChip = { candidate: number; index: number; value: unknown };

/**
 * The candidate lists a row's raw text holds, one chip-ready array each.
 *
 * A single-valued list parameter is one candidate whatever its chips say; an
 * expanded one has a candidate per entry. Comma strings and expressions are
 * resolved here, so what the editor writes back never needs resolving again.
 */
export function listCandidates(raw: string, elementType: string, expanded: boolean): unknown[][] {
  const items = parseListRaw(raw);
  if (!expanded) {
    return [items.flatMap((item) => chipValues(item, elementType))];
  }
  return items.map((item) => chipValues(item, elementType));
}

/** The candidates as a flat chip list, in reading order. */
export function listChips(candidates: unknown[][]): ListChip[] {
  return candidates.flatMap((values, candidate) => values.map((value, index) => ({ candidate, index, value })));
}

/**
 * Serialize candidates back into a row's raw text.
 *
 * A candidate of one value collapses to that value, which is the shape a chip
 * the operator typed arrives in and the shape `candidateLists` reads back as
 * one candidate; a longer candidate stays an array, which is the shape a
 * registered default arrives in.
 */
export function formatListCandidates(candidates: unknown[][], expanded: boolean): string {
  if (!expanded) {
    return formatListRaw(candidates[0] ?? []);
  }
  return formatListRaw(candidates.filter((values) => values.length > 0).map((values) => (values.length === 1 ? values[0] : values)));
}

/** The default element for a newly added list row. */
export function emptyListItem(elementType: string): unknown {
  const normalized = elementType.toLowerCase();
  if (BOOL_TYPES.has(normalized)) {
    return false;
  }
  if (NUMERIC_TYPES.has(normalized)) {
    return 0;
  }
  return "";
}

/* === Private === */

/**
 * The candidate lists one chip of an expanded list parameter contributes.
 *
 * Three readings, and the chip's own shape decides which: an expression is the
 * values it generates and each of those is a case of its own, so `1..5` is
 * five candidates; a comma list is one list of several values; anything else
 * is one list of one value. A chip the operator never touched arrives as an
 * array, which is the registered default's own list and stays one candidate.
 */
/**
 * The values one stored entry shows as chips: an author's array, the parts of
 * a comma string, or the entry itself.
 */
function chipValues(item: unknown, elementType: string): unknown[] {
  if (Array.isArray(item)) {
    return item;
  }
  /* An expression keeps its own chip until it is committed: `linspace(0, 1,
     3)` holds commas that are its syntax, not its values, and expanding it
     while the operator is still typing would take the caret with it. */
  if (expandChipValue(item, elementType)) {
    return [item];
  }
  if (typeof item === "string" && item.includes(",")) {
    const parts = splitCommaPreservingDecimals(item)
      .map((part) => part.trim())
      .filter(Boolean);
    if (parts.length > 1) {
      return parts;
    }
  }
  return [item];
}

function candidateLists(candidate: unknown, elementType: string): unknown[][] {
  if (Array.isArray(candidate)) {
    return [candidate];
  }
  const expanded = expandChipValue(candidate, elementType);
  if (expanded) {
    return expanded.map((value) => [value]);
  }
  if (typeof candidate === "string" && candidate.includes(",")) {
    const parts = splitCommaPreservingDecimals(candidate)
      .map((part) => part.trim())
      .filter(Boolean);
    if (parts.length > 1) {
      return [parts];
    }
  }
  return [[candidate]];
}

function parseListInput(raw: string, list: ParameterListDefinition): ParseResult {
  let parsed: unknown;
  try {
    parsed = reviveNativeValues(JSON.parse(raw));
  } catch {
    return { ok: false, error: "List value is not readable." };
  }
  if (!Array.isArray(parsed)) {
    return { ok: false, error: "A list parameter needs a list of values." };
  }

  const elementType = list.element_type.toLowerCase();
  const items: unknown[] = [];
  for (const [index, item] of parsed.entries()) {
    /* A numeric list element may itself be an expression: `1..5` and
       `linspace(0, 1, 11)` stand for the values they generate, so one entry
       can contribute several items rather than failing as "not a number". */
    if (typeof item === "string" && NUMERIC_TYPES.has(elementType)) {
      const expansion = tryNumericExpansion(item.trim(), elementType);
      if (expansion) {
        if (!expansion.ok) {
          return { ok: false, error: `Item ${index + 1}: ${expansion.error}` };
        }
        items.push(...expansion.values);
        continue;
      }
    }
    const coerced = coerceListItem(item, elementType);
    if (!coerced.ok) {
      return { ok: false, error: `Item ${index + 1}: ${coerced.error}` };
    }
    items.push(coerced.value);
  }

  const bounds = listBoundsError(items.length, list);
  if (bounds) {
    return { ok: false, error: bounds };
  }

  // One array, so the parameter contributes one case rather than one per item.
  return { ok: true, values: [items] };
}

function coerceListItem(item: unknown, elementType: string): { ok: true; value: unknown } | { ok: false; error: string } {
  if (item === null || item === undefined) {
    return { ok: false, error: "value is missing." };
  }
  if (BOOL_TYPES.has(elementType)) {
    if (typeof item === "boolean") {
      return { ok: true, value: item };
    }
    return coerceSingle(String(item), elementType);
  }
  if (NUMERIC_TYPES.has(elementType)) {
    if (item instanceof NativeIntegerValue || item instanceof NativeNumberValue) return { ok: true, value: item };
    if (typeof item === "number") {
      if ((elementType === "int" || elementType === "integer") && !Number.isSafeInteger(item))
        return {
          ok: false,
          error: "Use exact integer text for integers outside the safe JavaScript range.",
        };
      return Number.isFinite(item)
        ? {
            ok: true,
            value: elementType === "int" || elementType === "integer" ? item : new NativeNumberValue(item),
          }
        : { ok: false, error: "value is not a valid number." };
    }
    const text = String(item).trim();
    // `Number("")` is 0; an empty field is an unfinished entry, not a zero.
    if (!text) {
      return { ok: false, error: "value is missing." };
    }
    return coerceSingle(text, elementType);
  }
  if (typeof item === "object") {
    return { ok: false, error: "value is not text." };
  }
  return { ok: true, value: String(item) };
}

function tryNumericExpansion(input: string, valueType: string): ParseResult | null {
  // linspace(start, stop, n)
  const linspace = input.match(/^linspace\(\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)\s*\)$/i);
  if (linspace) {
    const start = Number(linspace[1]);
    const stop = Number(linspace[2]);
    const n = Number(linspace[3]);
    if ((valueType === "int" || valueType === "integer") && ![start, stop].every(Number.isSafeInteger)) {
      return {
        ok: false,
        error: "Integer ranges require exact safe integer endpoints; enter wider integers individually.",
      };
    }
    if (!Number.isFinite(n) || !Number.isInteger(n) || n < 1) {
      return { ok: false, error: "linspace n must be a positive integer." };
    }
    if (n > RANGE_HARD_CAP) {
      return {
        ok: false,
        error: `linspace n=${n} exceeds the ${RANGE_HARD_CAP}-value cap.`,
      };
    }
    if (n === 1) {
      return { ok: true, values: [coerceNumber(start, valueType)] };
    }
    const step = (stop - start) / (n - 1);
    const values: unknown[] = [];
    for (let i = 0; i < n; i += 1) {
      values.push(coerceNumber(start + i * step, valueType));
    }
    return { ok: true, values };
  }

  // arange(start, stop[, step]) - half-open
  const arange = input.match(/^arange\(\s*(-?\d+(?:\.\d+)?)\s*,\s*(-?\d+(?:\.\d+)?)(?:\s*,\s*(-?\d+(?:\.\d+)?))?\s*\)$/i);
  if (arange) {
    const start = Number(arange[1]);
    const stop = Number(arange[2]);
    const step = arange[3] !== undefined ? Number(arange[3]) : 1;
    return expandRange({ start, stop, step, valueType, inclusive: false });
  }

  // start..stop[:step] - inclusive arithmetic range (Julia/MATLAB-ish)
  const range = input.match(/^(-?\d+(?:\.\d+)?)\s*\.\.\s*(-?\d+(?:\.\d+)?)(?:\s*:\s*(-?\d+(?:\.\d+)?))?$/);
  if (range) {
    const start = Number(range[1]);
    const stop = Number(range[2]);
    const step = range[3] !== undefined ? Number(range[3]) : 1;
    return expandRange({ start, stop, step, valueType, inclusive: true });
  }

  return null;
}

function expandRange(opts: { start: number; stop: number; step: number; valueType: string; inclusive: boolean }): ParseResult {
  const { start, stop, step, valueType, inclusive } = opts;
  if ((valueType === "int" || valueType === "integer") && ![start, stop, step].every(Number.isSafeInteger)) {
    return {
      ok: false,
      error: "Integer ranges require exact safe integer endpoints and steps; enter wider integers individually.",
    };
  }
  if (step === 0 || !Number.isFinite(step)) {
    return { ok: false, error: "Range step must be a non-zero finite number." };
  }
  if ((stop - start) * step < 0) {
    return {
      ok: false,
      error: "Range step direction does not move from start to stop.",
    };
  }

  // Compute count up front so we can fail fast on absurd sizes.
  const span = Math.abs(stop - start);
  const countEstimate = Math.floor(span / Math.abs(step)) + (inclusive ? 1 : 0);
  if (countEstimate > RANGE_HARD_CAP) {
    return {
      ok: false,
      error: `Range would generate ${countEstimate} values; cap is ${RANGE_HARD_CAP}.`,
    };
  }

  const values: unknown[] = [];
  // Use index-based iteration to avoid floating-point drift.
  for (let i = 0; i <= countEstimate + 1 && values.length <= RANGE_HARD_CAP; i += 1) {
    const candidate = start + step * i;
    if (step > 0) {
      if (inclusive ? candidate > stop + epsilon(stop) : candidate >= stop - epsilon(stop)) {
        break;
      }
    } else {
      if (inclusive ? candidate < stop - epsilon(stop) : candidate <= stop + epsilon(stop)) {
        break;
      }
    }
    values.push(coerceNumber(candidate, valueType));
  }
  if (values.length === 0) {
    return { ok: false, error: "Range produced no values." };
  }
  return { ok: true, values };
}

function epsilon(value: number): number {
  return Math.max(Math.abs(value), 1) * 1e-9;
}

function coerceNumber(value: number, valueType: string): number | NativeNumberValue {
  if (valueType === "int" || valueType === "integer") {
    return Math.round(value);
  }
  // Strip trailing FP noise to keep previews readable.
  return new NativeNumberValue(Number(value.toFixed(12)));
}

function coerceSingle(raw: string, valueType: string): { ok: true; value: unknown } | { ok: false; error: string } {
  if (NUMERIC_TYPES.has(valueType)) {
    if (valueType === "int" || valueType === "integer") {
      try {
        return { ok: true, value: parseNativeInteger(raw.trim().replace(/\.0+$/, "")) };
      } catch {
        return { ok: false, error: `"${raw}" is not a valid 64-bit integer.` };
      }
    }
    const parsed = Number(raw);
    if (!Number.isFinite(parsed)) {
      return { ok: false, error: `"${raw}" is not a valid number.` };
    }
    if ((valueType === "int" || valueType === "integer") && !Number.isInteger(parsed)) {
      // accept "10" "10.0" but reject "10.5" for int.
      if (Number(raw) !== Math.trunc(Number(raw))) {
        return { ok: false, error: `"${raw}" is not a valid integer.` };
      }
    }
    return { ok: true, value: new NativeNumberValue(parsed) };
  }
  if (BOOL_TYPES.has(valueType)) {
    const normalized = raw.trim().toLowerCase();
    if (normalized === "true" || normalized === "1" || normalized === "yes") {
      return { ok: true, value: true };
    }
    if (normalized === "false" || normalized === "0" || normalized === "no") {
      return { ok: true, value: false };
    }
    return {
      ok: false,
      error: `"${raw}" is not a valid boolean (use true/false).`,
    };
  }
  return { ok: true, value: raw };
}

function splitCommaPreservingDecimals(input: string): string[] {
  // Simple split on commas. Decimals use `.` per template defaults so
  // there is no ambiguity. Future improvement: respect quoted strings.
  return input.split(",");
}
