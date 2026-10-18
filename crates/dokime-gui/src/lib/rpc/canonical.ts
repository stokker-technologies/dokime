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
