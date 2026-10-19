/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Local Imports === */

import { NativeIntegerValue, NativeNumberValue, parseNativeInteger } from "@/lib/api/native-values";
import { numericValue } from "@/lib/utils";

import type { JsonObject, PromptField, PromptRequest } from "./types";

/* === Types === */

export type PromptValue = string | boolean | string[];
export type PromptValues = Record<string, PromptValue>;

/* === Public Functions === */

export function promptFieldDefault(field: PromptField): PromptValue {
  if (field.multiple) {
    if (Array.isArray(field.default)) {
      return field.default.map((item) => String(item));
    }
    if (field.default === undefined || field.default === null) {
      return field.required === false ? [] : [""];
    }
    return [String(field.default)];
  }
  if (field.kind === "boolean") {
    return Boolean(field.default);
  }
  if (field.default === undefined || field.default === null) {
    return "";
  }
  return String(field.default);
}

export function promptDefaults(prompt: PromptRequest | undefined): PromptValues {
  if (!prompt) {
    return {};
  }
  return Object.fromEntries(prompt.fields.map((field) => [field.name, promptFieldDefault(field)]));
}

export function promptResponseValues(prompt: PromptRequest, values: PromptValues): JsonObject {
  return Object.fromEntries(
    prompt.fields
      .filter((field) => !(field.required === false && !field.multiple && (values[field.name] ?? promptFieldDefault(field)) === ""))
      .map((field) => {
        const value = values[field.name] ?? promptFieldDefault(field);
        if (field.multiple) {
          const items = Array.isArray(value) ? value : [String(value)];
          return [field.name, items.map((item) => coercePromptItem(field, item))];
        }
        if (field.kind === "boolean") {
          return [field.name, Boolean(value)];
        }
        return [field.name, coercePromptItem(field, String(value))];
      }),
  );
}

function coercePromptItem(field: PromptField, value: string) {
  if (field.kind === "int") {
    try {
      return parseNativeInteger(value);
    } catch {
      return null;
    }
  }
  if (field.kind === "float") {
    const parsed = /^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?$/.test(value.trim()) ? Number(value) : Number.NaN;
    return Number.isFinite(parsed) ? new NativeNumberValue(parsed) : null;
  }
  return value;
}

/** Immediate feedback; the host validates the submitted values again. */
export function promptFieldError(field: PromptField, value: PromptValue | undefined): string | null {
  if (field.readonly) return null;
  const current = value ?? promptFieldDefault(field);
  const items = field.multiple ? (Array.isArray(current) ? current : [String(current)]) : [current];
  const rules = field.validation ?? {};
  if (field.multiple) {
    const minimum = rules.min_items ?? (field.required === false ? 0 : 1);
    if (items.length < minimum) return `Choose at least ${minimum} values.`;
    if (rules.max_items != null && items.length > rules.max_items) return `Choose at most ${rules.max_items} values.`;
    if (field.kind === "choice" && new Set(items).size !== items.length) return "Choose each value only once.";
  }
  for (const item of items) {
    if (typeof item === "boolean") continue;
    const text = String(item);
    if (field.required !== false && text.trim() === "") return "Enter a value.";
    if (field.required === false && text === "") continue;
    if (field.kind === "float" || field.kind === "int") {
      const converted = coercePromptItem(field, text);
      const number = numericValue(converted);
      if (converted === null)
        return field.kind === "int" ? "Enter a whole number that can be represented exactly." : "Enter a finite number.";
      if (converted instanceof NativeIntegerValue) {
        const integer = BigInt(converted.decimal);
        if (rules.minimum != null && integer < BigInt(Math.ceil(rules.minimum))) return `Enter at least ${rules.minimum}.`;
        if (rules.maximum != null && integer > BigInt(Math.floor(rules.maximum))) return `Enter at most ${rules.maximum}.`;
      }
      if (rules.minimum != null && number !== null && number < rules.minimum) return `Enter at least ${rules.minimum}.`;
      if (rules.maximum != null && number !== null && number > rules.maximum) return `Enter at most ${rules.maximum}.`;
    } else {
      const length = Array.from(text).length;
      if (rules.min_length != null && length < rules.min_length) return `Enter at least ${rules.min_length} characters.`;
      if (rules.max_length != null && length > rules.max_length) return `Enter at most ${rules.max_length} characters.`;
      if (field.kind === "choice" && !field.choices?.includes(text)) return "Choose an offered value.";
      // Rust and JavaScript differ on Unicode classes and set operations.
      // Only give local pattern errors for their common ASCII subset.
      if (rules.pattern && !/[\\]|&&|--|~~|\[:/.test(rules.pattern) && /^[\x20-\x7e]*$/.test(text)) {
        try {
          if (!new RegExp(rules.pattern, "u").test(text)) return "Use the requested format.";
        } catch {
          // Rust supports patterns unavailable in JavaScript. The host checks these.
        }
      }
    }
  }
  return null;
}
