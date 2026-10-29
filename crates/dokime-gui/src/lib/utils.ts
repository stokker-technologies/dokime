/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export type KeyedValue<T> = { key: string; value: T };

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/** Narrow an unknown payload to a plain object, excluding arrays and null. */
export function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** Narrow away the absent and non-finite readings a payload may carry. */
export function isNumber(value: number | null | undefined): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

/**
 * Pair each value with a stable React key built from its own content, suffixing
 * the repeats so a list of identical items (two blank lines, the same token
 * twice) keeps one key per position without falling back to the index.
 */
export function occurrenceKeys<T>(values: T[], signature: (value: T) => string): KeyedValue<T>[] {
  const counts = new Map<string, number>();
  return values.map((value) => {
    const baseKey = signature(value);
    const count = counts.get(baseKey) ?? 0;
    counts.set(baseKey, count + 1);
    return { key: count === 0 ? baseKey : `${baseKey}\u001f${count + 1}`, value };
  });
}

/** Project lossless native numeric values for display or numeric calculations only. */
export function numericValue(value: unknown): number | null {
  if (typeof value === "number") return value;
  if (!isRecord(value) || typeof value.decimal !== "string") return null;
  if (value.__dokime_native_number__ === true) {
    return value.decimal === "nan"
      ? NaN
      : value.decimal === "inf"
        ? Infinity
        : value.decimal === "-inf"
          ? -Infinity
          : Number(value.decimal);
  }
  if (value.__dokime_native_integer__ === true) {
    const number = Number(value.decimal);
    return Number.isSafeInteger(number) ? number : null;
  }
  return null;
}
