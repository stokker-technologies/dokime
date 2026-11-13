// Dokime — SPDX-License-Identifier: Apache-2.0

/* === Types === */

/** Derive Structural Fields From Canonical Generated Messages Across Method Namespaces. */
export type Wire<T> = T extends Uint8Array
  ? T
  : T extends readonly (infer Item)[]
    ? Wire<Item>[]
    : T extends object
      ? { [Key in keyof T as Key extends "$typeName" | "$unknown" ? never : Key]: Wire<T[Key]> }
      : T;

/** Require a Value Promised by a Successful Resource Read. */
export function required<T>(value: T | undefined | null, action: string): T {
  if (value === undefined || value === null) throw new Error(`${action}: Host Omitted Its Value`);
  return value;
}

/** Compare Opaque Read Cuts Without Reinterpreting Their Positions. */
export function cutKey(value: unknown): string {
  return JSON.stringify(value, (key, item: unknown) =>
    key.startsWith("$") ? undefined : typeof item === "bigint" ? item.toString() : item,
  );
}
