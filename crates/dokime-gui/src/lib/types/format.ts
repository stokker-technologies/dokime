/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Value format types ===
   Mirrors `dokime_contract::core::format::ValueFormat` (crates/dokime-contract/src/core/format.rs).
   Five number kinds, three time kinds; `kind` is the serde tag. */

export type ElapsedUnit = "seconds" | "milliseconds" | "minutes";

export type ValueFormat =
  | { kind: "normal"; max_fraction_digits?: number | null }
  | { kind: "fixed"; decimals: number }
  | { kind: "scientific"; significant_digits: number }
  | { kind: "engineering"; significant_digits: number }
  | { kind: "si_prefix"; significant_digits: number }
  | { kind: "absolute_time"; pattern: string }
  | { kind: "elapsed_since_run"; unit: ElapsedUnit }
  | { kind: "elapsed_since_cycle"; unit: ElapsedUnit };
